use clap::Parser;
use std::{
    net::{IpAddr, ToSocketAddrs},
    process, thread,
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// dns name or ip address
    hostname: String,

    #[arg(short = 'i', default_value_t = 1.0)]
    interval: f64,
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
    let interval = Duration::from_secs_f64(args.interval);

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
    loop {
        let time_begin = Instant::now();
        println!("{:#?}", config);

        let time_left_to_sleep = config.interval - Instant::now().duration_since(time_begin);
        if time_left_to_sleep > Duration::new(0, 0) {
            thread::sleep(time_left_to_sleep)
        }
    }
}
