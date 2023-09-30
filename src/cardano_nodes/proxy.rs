use clap::Parser;
use miette::{bail, Context, IntoDiagnostic};
use std::{path::PathBuf, sync::Arc};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpStream, UnixStream},
};
use tokio_rustls::TlsConnector;
use tracing::{debug, info, instrument, warn};

#[derive(Parser)]
pub struct Args {
    /// the name of the Cardano node instance
    instance: String,

    /// local path for the unix socket
    #[arg(long)]
    socket: Option<PathBuf>,

    /// override the default remote hostname
    #[arg(long)]
    host: Option<String>,

    /// override the default remote port
    #[arg(long)]
    port: Option<u16>,
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

fn define_remote_host(args: &Args, ctx: &crate::Context) -> miette::Result<String> {
    if let Some(explicit) = &args.host {
        return Ok(explicit.to_owned());
    }

    return Ok(format!(
        "cardanonode-{}-n2c-{}.{}",
        args.instance, ctx.config.project.name, ctx.config.operator.entrypoint,
    ));
}

const DEFAULT_REMOTE_PORT: u16 = 9443;

fn define_remote_port(args: &Args) -> u16 {
    args.port.unwrap_or(DEFAULT_REMOTE_PORT)
}

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

fn define_socket_path(args: &Args, ctx: &crate::Context) -> miette::Result<PathBuf> {
    let default = ctx
        .dirs
        .ensure_extension_dir("cardano-nodes", "v2")?
        .join(format!("{}.socket", args.instance));

    let path = args.socket.to_owned().unwrap_or(default);

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

#[instrument("proxy", skip_all)]
pub async fn run(args: Args, ctx: &crate::Context) -> miette::Result<()> {
    let socket = define_socket_path(&args, ctx).context("error defining unix socket path")?;
    debug!(path = ?socket, "socket path defined");

    let host = define_remote_host(&args, ctx).context("defining remote host")?;
    let port = define_remote_port(&args);

    debug!(host, port, "remote endpoint defined");

    let server = tokio::net::UnixListener::bind(&socket)
        .into_diagnostic()
        .context("error creating unix socket listener")?;

    loop {
        info!(path = ?socket, "waiting for client connections");

        tokio::select! {
            result = server.accept() => {
                let (local, _) = result.into_diagnostic()?;
                spawn_new_connection(local, &host, port).await?;
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    std::fs::remove_file(socket)
        .into_diagnostic()
        .context("error trying to remove unix socket")?;

    Ok(())
}
