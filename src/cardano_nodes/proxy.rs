use std::{os::fd::AsFd, path::Path, sync::Arc, time::Duration};
use tokio_rustls::TlsConnector;
use tracing::instrument;

use clap::Parser;
use miette::{bail, Context, IntoDiagnostic};
use tokio::io::{self, AsyncRead, AsyncWrite};
use tracing::{info, warn};

#[derive(Parser)]
pub struct Args {
    //#[arg(short, long)]
    //path2: String
    #[arg(help = "the name of the cardano node instance")]
    instance: String,
}

pub async fn proxy<T1, T2>(s1: T1, s2: T2) -> miette::Result<()>
where
    T1: AsyncRead + AsyncWrite + Unpin,
    T2: AsyncRead + AsyncWrite + Unpin,
{
    let (mut read_1, mut write_1) = io::split(s1);
    let (mut read_2, mut write_2) = io::split(s2);

    tokio::select! {
        res=io::copy(&mut read_1, &mut write_2)=>{
            match res {
                Ok(read) => println!("read {read} bytes"),
                Err(err) => bail!(err),
            }
        },
        res=io::copy(&mut read_2, &mut write_1)=>{
            match res {
                Ok(read) => println!("read {read} bytes"),
                Err(err) => bail!(err),
            }
        }
    }

    println!("closing connection");

    Ok(())
}

#[instrument(skip_all)]
pub async fn run(args: Args) -> miette::Result<()> {
    let socket = Path::new("node0.socket");

    // Bind to socket
    let server = match tokio::net::UnixListener::bind(&socket) {
        Err(err) => bail!(err),
        Ok(stream) => stream,
    };

    let (local, client) = server.accept().await.into_diagnostic()?;

    println!("connected");
    info!(?client, "new client connected to node0");

    let remote = tokio::net::TcpStream::connect("node-preprod-stable.us1.demeter.run:9443")
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

    let domain = rustls::ServerName::try_from("node-preprod-stable.us1.demeter.run")
        .into_diagnostic()
        .context("invalid DNS name")?;

    let remote = TlsConnector::from(config)
        .connect(domain, remote)
        .await
        .into_diagnostic()
        .context("couldn't connect to TLS server")?;

    proxy(local, remote).await?;

    Ok(())
}
