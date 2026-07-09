use std::error::Error;
use std::net::SocketAddr;

use clap::Parser;
use qualisys_rust_sdk::rt::{QtmSimulator, SimulatorOptions};

#[derive(Debug, Parser)]
#[command(
    name = "qualisys-sim",
    version,
    about = "QTM RT simulator that streams synthetic motion-capture frames",
    long_about = "Serves a small QTM RT-compatible endpoint for development without a live \
Qualisys system. The simulator answers basic RT commands and streams synthetic 6D residual frames.",
    next_line_help = true,
    after_help = "\
Examples:
  qualisys-sim
  qualisys-sim --bind 127.0.0.1:22224 --hz 240 --rigid-bodies 2

Environment:
  QUALISYS_SIM_BIND, QUALISYS_SIM_HZ, QUALISYS_SIM_RIGID_BODIES"
)]
struct Cli {
    #[arg(
        short,
        long,
        env = "QUALISYS_SIM_BIND",
        value_name = "ADDR:PORT",
        default_value = "127.0.0.1:22223",
        help = "TCP address for the simulated QTM RT command socket"
    )]
    bind: SocketAddr,

    #[arg(
        long = "hz",
        alias = "frame-rate-hz",
        env = "QUALISYS_SIM_HZ",
        value_name = "HZ",
        default_value_t = 240,
        help = "Synthetic stream frame rate"
    )]
    frame_rate_hz: u32,

    #[arg(
        long,
        env = "QUALISYS_SIM_RIGID_BODIES",
        value_name = "N",
        default_value_t = 1,
        help = "Number of synthetic rigid bodies per frame"
    )]
    rigid_bodies: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    QtmSimulator::new(SimulatorOptions {
        bind_address: cli.bind,
        frame_rate_hz: cli.frame_rate_hz,
        rigid_body_count: cli.rigid_bodies,
    })
    .run()?;
    Ok(())
}
