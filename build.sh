cargo build --release && 
sudo setcap cap_net_raw=+eip target/release/rping && 
cargo run --release -- "$@"
