use crate::{context::extract_context_data, rpc};
use clap::Parser;
use colored::Colorize;
use dmtri::demeter::ops::v1alpha::Resource;
use miette::{bail, Context, IntoDiagnostic};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpStream, UnixStream},
};
use tracing::{debug, error, info, warn};

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
    name: &str,
    dirs: &crate::dirs::Dirs,
    ctx: &crate::context::Context,
) -> miette::Result<PathBuf> {
    let default = dirs
        .ensure_tmp_dir(&ctx.namespace.name)?
        .join(format!("{name}.socket"));

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

    let remote = connect_remote(remote_host, remote_port).await?;
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

struct NodeOption(Resource);

impl std::fmt::Display for NodeOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let network = match serde_json::from_str::<serde_json::Value>(&self.0.spec) {
            Ok(spec) => match spec.get("network") {
                Some(v) => v.as_str().unwrap().to_string(),
                None => "unknown".to_string(),
            },
            Err(err) => {
                error!(?err);
                "unknown".to_string()
            }
        };

        write!(f, "{}: {} - {}", self.0.kind, self.0.name, network)
    }
}

async fn define_port(port_name: Option<String>, cli: &crate::Cli) -> miette::Result<Resource> {
    let (api_key, project_id, _) = extract_context_data(cli).await?;

    let response = rpc::resources::find(&api_key, &project_id).await?;
    if response.is_empty() {
        bail!("you don't have any cardano-node ports, run dmtrctl ports create");
    }

    let available: Vec<_> = response
        .into_iter()
        .filter(|p| p.kind == CARDANO_NODE_KIND)
        .map(NodeOption)
        .collect();

    if available.is_empty() {
        bail!("you don't have any cardano-node ports, run dmtrctl ports create");
    }

    if let Some(port_name) = port_name {
        let port = available
            .into_iter()
            .find(|p| p.0.name == port_name)
            .ok_or(miette::miette!("can't find port"))?;

        return Ok(port.0);
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

const CARDANO_NODE_KIND: &str = "CardanoNodePort";

pub async fn run(args: Args, cli: &crate::Cli) -> miette::Result<()> {
    let ctx = cli
        .context
        .as_ref()
        .ok_or(miette::miette!("missing context"))?;

    let resource = define_port(args.port, cli).await?;

    let spec: serde_json::Value = serde_json::from_str(&resource.spec)
        .into_diagnostic()
        .context("error parsing resource spec")?;

    let auth_token = spec.get("authToken").unwrap().as_str().unwrap();
    let hostname = format!("{}.cnode-m1.demeter.run", auth_token);

    let socket_path = define_socket_path(args.socket, &resource.name, &cli.dirs, ctx)
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
                spawn_new_connection(local, &hostname, DEFAULT_REMOTE_PORT, counter.clone()).await?;
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
