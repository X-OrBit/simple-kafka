mod server;

use server::Server;
use simplelog::*;
use std::net::{IpAddr, SocketAddr};
use tokio::io;

use clap::Parser;
use log::error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Server ip address.
    ///
    /// Example: 127.0.0.1
    #[arg(long)]
    address: IpAddr,

    /// Server port.
    ///
    /// Example: 7000
    #[arg(long)]
    port: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .unwrap();

    let args = Args::parse();

    let socket_address = SocketAddr::new(args.address, args.port);

    let mut server = match Server::new(&socket_address).await {
        Err(e) => {
            error!("Not able to start server on {} -- {}", socket_address, e);
            return Err(e);
        }
        Ok(server) => server,
    };
    server.run().await?;

    Ok(())
}
