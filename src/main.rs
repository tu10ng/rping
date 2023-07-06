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

    /// Stop after sending count ECHO_REQUEST packets.
    #[arg(short = 'c', long = "count", default_value_t = 0, value_parser = clap::value_parser!(u16).range(0..))]
    count: u16,

    /// Wait interval seconds between sending each packet. positive integer allowed.
    #[arg(short = 'i', long = "interval", default_value_t = 1, value_parser = clap::value_parser!(u64).range(1..))]
    interval: u64,

    /// Quiet output. Nothing is displayed except the summary lines at startup time and when finished.
    #[arg(short = 'q')]
    quiet: bool,

    /// Specifies the number of data bytes to be sent.
    #[arg(short = 's', long = "packetsize", default_value_t = 56, value_parser = clap::value_parser!(u64).range(1..))]
    packet_size: u64,

    /// Set the IP Time to Live.
    #[arg(short = 't', long = "ttl", default_value_t = 52, value_parser = clap::value_parser!(u8).range(1..))]
    ttl: u8,
}

#[derive(Debug)]
struct Config {
    destination: IpAddr,
    count: u16,
    interval: Duration,
    quiet: bool,
    packet_size: usize,
    ttl: u8,
}

impl Config {
    pub fn new(
        destination: IpAddr,
        count: u16,
        interval: Duration,
        quiet: bool,
        packet_size: usize,
        ttl: u8,
    ) -> Option<Config> {
        Some(Config {
            destination,
            count,
            interval,
            quiet,
            packet_size,
            ttl,
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
    let packet_size: usize = args.packet_size.try_into().unwrap();

    Config::new(
        destination,
        args.count,
        interval,
        args.quiet,
        packet_size,
        args.ttl,
    )
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
    let mut stat_received = 0;
    let time_init = Instant::now();
    println!(
        "RPING {} {} bytes of data",
        config.destination, config.packet_size
    );

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
        match ping(config.destination, config.ttl, config.packet_size, sequence) {
            Some(time) => {
                stat_received += 1;
                if !config.quiet {
                    println!(
                        "{} bytes from {}: icmp_seq={} ttl={} time={}ms",
                        config.packet_size + 8,
                        config.destination,
                        sequence,
                        config.ttl,
                        time.as_millis()
                    );
                }
            }
            None => {
                if !config.quiet {
                    println!("no answer");
                }
            }
        }
        sequence += 1;

        // sleep until interval is reached
        if config.interval > Instant::now().duration_since(time_begin) {
            let time_left_to_sleep = config.interval - Instant::now().duration_since(time_begin);
            thread::sleep(time_left_to_sleep);
        }

        // end loop if reached count
        if config.count != 0 && sequence >= config.count {
            break;
        }
    }

    // when \C-c is pressed
    println!("--- {} rping statistics ---", config.destination);
    println!(
        "{} packets transmitted, {} received, {}% packet loss, time {}ms",
        sequence,
        stat_received,
        (sequence - stat_received) / sequence,
        Instant::now().duration_since(time_init).as_millis()
    );
}

fn ping(address: IpAddr, ttl: u8, packet_size: usize, sequence: u16) -> Option<Duration> {
    let timeout: Duration = Duration::new(5, 0);
    let identifier: u16 = (std::process::id() % u16::max_value() as u32) as u16;
    let size = packet_size + 8; // 56 data bytes + 8 icmp header
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
    tx.set_ttl(ttl).unwrap();

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
