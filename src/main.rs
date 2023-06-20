use clap::Parser;
use pnet::{
    packet::{
        icmp::{echo_reply, echo_request, IcmpTypes},
        ip::IpNextHeaderProtocols,
        Packet,
    },
    transport::{icmp_packet_iter, transport_channel, TransportChannelType, TransportProtocol},
};
use std::{
    net::{IpAddr, ToSocketAddrs},
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// dns name or ip address
    hostname: String,

    #[arg(short = 'i', default_value_t = 1, value_parser = clap::value_parser!(u64).range(1..))]
    interval: u64,
}

#[derive(Debug)]
struct Config {
    // 93.184.216.34
    destination: IpAddr,
    interval: Duration,
}

impl Config {
    pub fn new(destination: IpAddr, interval: Duration) -> Option<Config> {
        Some(Config {
            destination,
            interval,
        })
    }
}

fn parse() -> Option<Config> {
    let args = Cli::parse();
    let destination = (args.hostname, 80)
        .to_socket_addrs()
        .unwrap()
        .nth(0)
        .unwrap()
        .ip();
    let interval = Duration::from_secs(args.interval);

    Config::new(destination, interval)
}

fn main() {
    if let Some(config) = parse() {
        run(config);
    } else {
        process::exit(1);
    }
}

fn run(config: Config) {
    let mut sequence: u16 = 0;
    eprintln!("{:#?}", config);

    // handle \C-c
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    loop {
        // when \C-c is pressed
        if !running.load(Ordering::SeqCst) {
            break;
        }

        let time_begin = Instant::now();

        // send message
        let timeout: Duration = Duration::new(5, 0);
        let identifier: u16 = 114;
        match ping(config.destination, timeout, sequence, identifier) {
            Some(time) => {
                println!(
                    "64 bytes from {}: icmp_seq={} time={}ms",
                    config.destination,
                    sequence,
                    time.as_millis()
                );
            }
            None => {
                println!("no answer");
            }
        }
        sequence += 1;

        // sleep until interval is reached
        let time_left_to_sleep = config.interval - Instant::now().duration_since(time_begin);
        if time_left_to_sleep > Duration::new(0, 0) {
            thread::sleep(time_left_to_sleep)
        }
    }

    // when \C-c is pressed
    println!("statistics: ");
}

fn ping(address: IpAddr, timeout: Duration, sequence: u16, identifier: u16) -> Option<Duration> {
    let size = 64;
    let mut packet_buffer: Vec<u8> = vec![0; size];
    // ipv4
    assert!(address.is_ipv4());
    let mut packet = echo_request::MutableEchoRequestPacket::new(&mut packet_buffer).unwrap();
    packet.set_icmp_type(IcmpTypes::EchoRequest);
    packet.set_sequence_number(sequence);
    packet.set_identifier(identifier);
    packet.set_checksum(pnet::util::checksum(packet.packet(), 1));
    let (mut tx, mut rx) = transport_channel(
        size,
        TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Icmp)),
    )
    .unwrap();
    tx.send_to(packet, address).unwrap();

    let time_start = Instant::now();
    let mut rx_iter = icmp_packet_iter(&mut rx);
    loop {
        let data = rx_iter.next_with_timeout(timeout).unwrap();
        match data {
            Some(data) => {
                let (received, _) = data;
                if received.get_icmp_type() == IcmpTypes::EchoReply {
                    let reply = echo_reply::EchoReplyPacket::new(received.packet()).unwrap();
                    if reply.get_identifier() == identifier
                        && reply.get_sequence_number() == sequence
                    {
                        return Some(Instant::now().duration_since(time_start));
                    } else {
                        panic!("maybe impossible sequence number");
                    }
                }
            }

            None => return None,
        }
    }
}
