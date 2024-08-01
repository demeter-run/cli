use crate::api::{self, Instance, PortInfo};
use clap::Parser;
use colored::Colorize;
use miette::{bail, Context, IntoDiagnostic};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpStream, UnixStream},
};
use tracing::{debug, info, warn};

#[derive(Parser)]
pub struct Args {
    /// Instance of the port to connect (cardano-node kind)
    port: Option<String>,

    /// local path where the unix socket will be created
    #[arg(long)]
    socket: Option<PathBuf>,
}

pub async fn copy_bytes<T1, T2>(s1: T1, s2: T2) -> miette::Result<()>
where
    T1: AsyncRead + AsyncWrite + Unpin,
    T2: AsyncRead + AsyncWrite + Unpin,
{
    let (mut read_1, mut write_1) = tokio::io::split(s1);
    let (mut read_2, mut write_2) = tokio::io::split(s2);

    tokio::select! {
        res=tokio::io::copy(&mut read_1, &mut write_2)=>{
            match res {
                Ok(read) => debug!(read, "downstrem EOF"),
                Err(err) => bail!(err),
            }
        },
        res=tokio::io::copy(&mut read_2, &mut write_1)=>{
            match res {
                Ok(read) => debug!(read, "upstream EOF"),
                Err(err) => bail!(err),
            }
        }
    }

    warn!("connection ended");

    Ok(())
}

const DEFAULT_REMOTE_PORT: u16 = 9443;

async fn connect_remote<'a>(
    host: &str,
    port: u16,
) -> miette::Result<tokio_rustls::client::TlsStream<TcpStream>> {
    let remote = tokio::net::TcpStream::connect(format!("{host}:{port}"))
        .await
        .into_diagnostic()?;

    remote.set_nodelay(true).unwrap();

    {
        let sref = socket2::SockRef::from(&remote);
        sref.set_keepalive(true).unwrap();
    }

    // remote
    //     .set_linger(Some(Duration::from_secs(6000000)))
    //     .unwrap();

    let certs = rustls_native_certs::load_native_certs()
        .into_diagnostic()
        .context("error loading TLS certificates")?;

    let mut roots = tokio_rustls::rustls::RootCertStore::empty();

    for cert in certs {
        roots.add(cert).unwrap();
    }

    let config = tokio_rustls::rustls::ClientConfig::builder()
        .with_root_certificates(Arc::new(roots))
        .with_no_client_auth();

    let config = Arc::new(config);

    let domain = host
        .to_owned()
        .try_into()
        .into_diagnostic()
        .context("invalid DNS name")?;

    let connector = tokio_rustls::TlsConnector::from(config);

    let remote = connector
        .connect(domain, remote)
        .await
        .into_diagnostic()
        .context("couldn't connect to TLS server")?;

    Ok(remote)
}

fn define_socket_path(
    explicit: Option<PathBuf>,
    port: &PortInfo,
    dirs: &crate::dirs::Dirs,
    ctx: &crate::core::Context,
) -> miette::Result<PathBuf> {
    let default = dirs
        .ensure_tmp_dir(&ctx.project.namespace)?
        .join(format!("{}-{}.socket", port.network, port.version));

    let path = explicit.to_owned().unwrap_or(default);

    if path.exists() {
        bail!("path for the socket already exists");
    }

    Ok(path)
}

async fn spawn_new_connection(
    local: UnixStream,
    remote_host: &str,
    remote_port: u16,
    counter: Arc<Mutex<ClientCounter>>,
) -> miette::Result<()> {
    info!("new client connected to socket");

    let remote = connect_remote(&remote_host, remote_port).await?;
    info!("connected to remote endpoint");

    let copy_op = async move {
        counter.lock().map(|mut x| x.increase()).unwrap();

        // actual work
        let result = copy_bytes(local, remote).await;

        counter.lock().map(|mut x| x.decrease()).unwrap();

        result
    };

    tokio::spawn(copy_op);
    info!("proxy running");

    Ok(())
}

struct NodeOption(PortInfo);

impl std::fmt::Display for NodeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{} ({}, {})",
            self.0.kind, self.0.id, self.0.network, self.0.version
        )
    }
}

fn get_instance_parts(instance: &str) -> (String, String) {
    let parts: Vec<&str> = instance.split('/').collect();
    (parts[0].to_string(), parts[1].to_string())
}

async fn define_port(explicit: Option<String>, cli: &crate::Cli) -> miette::Result<PortInfo> {
    let available: Vec<_> = api::get::<Vec<PortInfo>>(cli, &format!("ports/{}", CARDANO_NODE_KIND))
        .await
        .into_diagnostic()?
        .into_iter()
        .map(NodeOption)
        .collect();

    if available.is_empty() {
        bail!("you don't have any cardano-node ports, run dmtrctl ports create");
    }

    if let Some(explicit) = explicit {
        let (kind, id) = get_instance_parts(&explicit);

        if kind != CARDANO_NODE_KIND {
            bail!("tunnels are only supported for cardano-node ports");
        }

        let explicit = available
            .iter()
            .find(|p| p.0.id == id)
            .ok_or(miette::miette!("can't find port"))?;

        return Ok(explicit.0.clone());
    }

    let selection = inquire::Select::new("select port", available)
        .prompt()
        .into_diagnostic()
        .context("selecting available port")?;

    Ok(selection.0)
}

struct ClientCounter {
    current: u32,
    total: u32,
    spinner: spinoff::Spinner,
}

impl ClientCounter {
    fn update_msg(&mut self) {
        self.spinner.update_text(format!(
            "total clients: {}, active clients {}",
            self.total, self.current
        ));
    }

    fn increase(&mut self) {
        self.current += 1;
        self.total += 1;
        self.update_msg();
    }

    fn decrease(&mut self) {
        self.current -= 1;
        self.update_msg();
    }

    fn stop(&mut self) {
        self.spinner
            .stop_with_message("stopped serving unix socket");
    }
}

const CARDANO_NODE_KIND: &str = "cardano-node";

// #[instrument("connect", skip_all)]
pub async fn run(args: Args, cli: &crate::Cli) -> miette::Result<()> {
    let ctx = cli
        .context
        .as_ref()
        .ok_or(miette::miette!("missing context"))?;

    let port_info = define_port(args.port, cli).await?;

    let hostname = match &port_info.instance {
        Instance::NodePort(x) => &x.authenticated_endpoint,
        _ => bail!("invalid port instance, only kind cardano-node support tunnels"),
    };

    let socket_path = define_socket_path(args.socket, &port_info, &cli.dirs, ctx)
        .context("error defining unix socket path")?;

    debug!(path = ?socket_path, "socket path defined");

    let server = tokio::net::UnixListener::bind(&socket_path)
        .into_diagnostic()
        .context("error creating unix socket listener")?;

    println!("🧦 unix socket created, you can connect at:");
    println!("{}", socket_path.to_string_lossy().bright_magenta());

    let spinner = spinoff::Spinner::new(
        spinoff::spinners::BouncingBar,
        "waiting for client connections, CTRL+C to stop",
        spinoff::Color::Blue,
    );

    let counter = Arc::new(Mutex::new(ClientCounter {
        current: 0,
        total: 0,
        spinner,
    }));

    loop {
        tokio::select! {
            result = server.accept() => {
                let (local, _) = result.into_diagnostic()?;
                spawn_new_connection(local, hostname, DEFAULT_REMOTE_PORT, counter.clone()).await?;
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    counter.lock().map(|mut x| x.stop()).unwrap();

    std::fs::remove_file(socket_path)
        .into_diagnostic()
        .context("error trying to remove unix socket")?;

    Ok(())
}
