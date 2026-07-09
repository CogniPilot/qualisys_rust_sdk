use std::time::Duration;

use qualisys_rust_sdk::prelude::*;
use qualisys_rust_sdk::rt::ComponentData;

fn main() -> qualisys_rust_sdk::Result<()> {
    let mut args = std::env::args().skip(1);
    let host = args
        .next()
        .or_else(|| std::env::var("QUALISYS_RT_HOST").ok())
        .unwrap_or_else(|| "127.0.0.1".to_owned());
    let default_options = ClientOptions::default();
    let port = optional_u16(args.next(), "QUALISYS_RT_PORT", default_options.port);
    let frame_count = optional_usize(args.next(), "QUALISYS_RT_FRAME_COUNT", 10);

    let mut client = Client::connect(
        &host,
        ClientOptions {
            port,
            read_timeout: Duration::from_secs(5),
            ..default_options
        },
    )?;
    println!("Connected to QTM version: {}", client.qtm_version()?);

    let request = StreamFramesRequest::new(
        StreamRate::AllFrames,
        StreamTransport::Udp {
            bind_address: "0.0.0.0:0".parse().expect("valid UDP bind address"),
            destination: None,
        },
        [ComponentSelection::SixD],
    );

    client.start_stream_frames(&request)?;
    let local_udp = client
        .udp_local_addr()?
        .expect("UDP socket should be active after stream start");
    println!("Listening for RT packets on UDP {}", local_udp);

    let mut accumulator = FrameAccumulator::for_components(request.components.clone());
    let mut printed = 0usize;
    loop {
        match client.recv_stream_packet()? {
            StreamPacket::Data(packet) => {
                for frame in accumulator.push(packet) {
                    if let Some(component) = frame.component(ComponentType::SixD)
                        && let ComponentData::SixD(bodies) = &component.data
                    {
                        println!(
                            "frame={} complete={} body_count={}",
                            frame.frame_number,
                            frame.complete,
                            bodies.bodies.len()
                        );
                        printed += 1;
                        if frame_count != 0 && printed >= frame_count {
                            client.stop_stream_frames()?;
                            return Ok(());
                        }
                    }
                }
            }
            StreamPacket::NoMoreData => {
                println!("QTM reported end of stream");
                break;
            }
        }
    }

    Ok(())
}

fn optional_u16(argument: Option<String>, environment: &str, default: u16) -> u16 {
    argument
        .or_else(|| std::env::var(environment).ok())
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn optional_usize(argument: Option<String>, environment: &str, default: usize) -> usize {
    argument
        .or_else(|| std::env::var(environment).ok())
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
