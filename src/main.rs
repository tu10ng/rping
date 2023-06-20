use clap::Parser;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli
{
    /// dns name or ip address
    destination: String,
}


fn main() {
    let args = Cli::parse();
    
    let addr = (args.destination, 80).to_socket_addrs().unwrap().nth(0).unwrap().ip();
    println!("{:?}", addr);
}
