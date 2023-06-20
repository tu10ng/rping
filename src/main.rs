use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

fn main() {
    let addr = ("www.example.com", 80).to_socket_addrs().unwrap();
    println!("{:?}", addr);
}
