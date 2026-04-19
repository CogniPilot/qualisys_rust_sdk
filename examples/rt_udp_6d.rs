use qualisys_rust_sdk::prelude::*;
use qualisys_rust_sdk::rt::ComponentData;

fn main() -> qualisys_rust_sdk::Result<()> {
    let host = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1".to_owned());

    let mut client = Client::connect(&host, ClientOptions::default())?;
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
    loop {
        match client.recv_stream_packet()? {
            StreamPacket::Data(packet) => {
                for frame in accumulator.push(packet) {
                    if let Some(component) = frame.component(ComponentType::SixD) {
                        if let ComponentData::SixD(bodies) = &component.data {
                            println!(
                                "frame={} complete={} body_count={}",
                                frame.frame_number,
                                frame.complete,
                                bodies.bodies.len()
                            );
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
