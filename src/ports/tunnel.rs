use crate::api::{self, Instance, PortInfo, PortOptions};
use clap::Parser;
use miette::{bail, Context, IntoDiagnostic};
use std::{path::PathBuf, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpStream, UnixStream},
};
use tokio_rustls::TlsConnector;
use tracing::{debug, info, warn};

#[derive(Parser)]
pub struct Args {}

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

async fn connect_remote(
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

    let mut roots = rustls::RootCertStore::empty();

    for cert in certs {
        roots.add(&rustls::Certificate(cert.0)).unwrap();
    }

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(Arc::new(roots))
        .with_no_client_auth();

    let config = Arc::new(config);

    let domain = rustls::ServerName::try_from(host)
        .into_diagnostic()
        .context("invalid DNS name")?;

    let remote = TlsConnector::from(config)
        .connect(domain, remote)
        .await
        .into_diagnostic()
        .context("couldn't connect to TLS server")?;

    Ok(remote)
}

fn define_socket_path(
    port: &PortInfo,
    socket: Option<PathBuf>,
    dirs: &crate::dirs::Dirs,
    ctx: &crate::core::Context,
) -> miette::Result<PathBuf> {
    let default = dirs
        .ensure_tmp_dir(&ctx.namespace.name)?
        .join(format!("{}-{}.socket", port.network, port.version));

    let path = socket.to_owned().unwrap_or(default);

    if path.exists() {
        bail!("path for the socket already exists");
    }

    Ok(path)
}

async fn spawn_new_connection(
    local: UnixStream,
    remote_host: &str,
    remote_port: u16,
) -> miette::Result<()> {
    info!("new client connected to socket");

    let remote = connect_remote(&remote_host, remote_port).await?;
    info!("connected to remote endpoint");

    let copy_op = copy_bytes(local, remote);

    tokio::spawn(copy_op);
    info!("proxy running");

    Ok(())
}

// #[instrument("connect", skip_all)]
pub async fn run(cli: &crate::Cli) -> miette::Result<()> {
    let ctx = cli
        .context
        .as_ref()
        .ok_or(miette::miette!("missing context"))?;

    let tunnel_options = vec!["node"];

    let _tunnel = inquire::Select::new("Choose the port to tunnel", tunnel_options)
        .prompt()
        .into_diagnostic()?;

    let options: PortOptions = api::get_public(&format!("metadata/ports/{}", "cardano-node"))
        .await
        .into_diagnostic()?;

    let network_options = options.networks.clone();

    let network = inquire::Select::new("Choose the network", network_options)
        .prompt()
        .into_diagnostic()?;

    let network_versions = options.get_network_versions(&network);

    let version = inquire::Select::new("Choose the version", network_versions)
        .prompt()
        .into_diagnostic()?;

    let existing_ports: Vec<PortInfo> = api::get(cli, &format!("ports/{}", "cardano-node"))
        .await
        .into_diagnostic()?;

    let hostname: String;
    let port_info: PortInfo;
    // check if the port already exists using network and version
    if let Some(port) = existing_ports
        .iter()
        .find(|p| p.network == network && p.version == version)
    {
        port_info = port.clone();
        match &port.instance {
            Instance::NodePort(instance) => {
                hostname = instance.authenticated_endpoint.clone();
            }
            Instance::PostgresPort(_) => todo!(),
            Instance::HttpPort(_) => todo!(),
        }
    } else {
        let create_new_confirm =
            inquire::Confirm::new("Port does not exist. Do you want to create a new one?")
                .prompt()
                .into_diagnostic()?;

        if create_new_confirm {
            let new_port = api::create_port(cli, "cardano-node", &network, &version, "1")
                .await
                .into_diagnostic()?;

            port_info = new_port.clone();

            match new_port.instance {
                Instance::NodePort(instance) => {
                    hostname = instance.authenticated_endpoint.clone();
                }
                Instance::PostgresPort(_) => todo!(),
                Instance::HttpPort(_) => todo!(),
            }
        } else {
            bail!("port does not exist");
        }
    }

    let default_socket_path = define_socket_path(&port_info, None, &cli.dirs, ctx)
        .context("error defining unix socket path")?;

    let socket_path_input = inquire::Text::new("Enter the socket path")
        .with_help_message("The path to the unix socket")
        .with_default(&default_socket_path.to_string_lossy().to_string())
        .prompt()
        .into_diagnostic()?;

    let socket_path: Option<PathBuf> = PathBuf::from(socket_path_input).into();

    debug!(path = ?socket_path, "socket path defined");

    //check socket_path is not empty
    let default_socket_socket = socket_path.clone().unwrap();

    let server = tokio::net::UnixListener::bind(&default_socket_socket)
        .into_diagnostic()
        .context("error creating unix socket listener")?;

    loop {
        info!(path = ?default_socket_socket, "waiting for client connections");

        tokio::select! {
            result = server.accept() => {
                let (local, _) = result.into_diagnostic()?;
                spawn_new_connection(local, &hostname, DEFAULT_REMOTE_PORT).await?;
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    std::fs::remove_file(default_socket_socket)
        .into_diagnostic()
        .context("error trying to remove unix socket")?;

    Ok(())
}
