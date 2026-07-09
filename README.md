# Qualisys Rust SDK

`qualisys_rust_sdk` is a Rust-first foundation for an official Qualisys SDK. It
currently focuses on the QTM real-time (RT) protocol and is intentionally
structured so Qualisys or external contributors can extend it toward feature
parity with `qualisys_cpp_sdk` over time.

## Status

- Current focus: RT protocol transport, packet decoding, UDP frame assembly,
  and mocap metadata parsing from `GetParameters`.
- Current transport: little-endian RT port (`22223`).
- Current scope: streaming-oriented subset with packet models for the documented
  RT components.
- Planned growth path: settings XML helpers, richer control APIs, higher-level
  typed settings models, and parity with the C++ examples.

## Design Goals

- Match Qualisys naming where it helps discoverability and migration.
- Keep the public Rust API idiomatic and documented.
- Preserve low-level protocol access so new RT features can be added without
  reworking the crate architecture.
- Make UDP streaming practical by including a frame accumulator that merges
  split component packets for the same frame.

## Public API Layout

- `qualisys_rust_sdk::rt`: stable RT-facing namespace for the SDK.
- `qualisys_rust_sdk::prelude`: common imports for applications.
- Compatibility aliases such as `QRTConnection`, `QRTPacket`,
  `QRTComponentType`, and `QRTEvent` are exposed to make the Rust SDK easier to
  align with the existing Python and C++ SDKs.

## Diagnostic CLI

The crate includes a `qualisys-rt` binary for basic QTM RT diagnostics and a
`qualisys-sim` binary that serves synthetic RT frames when motion capture
hardware is not present:

```sh
cargo run --bin qualisys-sim -- --bind 127.0.0.1:22223 --hz 240
cargo run --bin qualisys-rt -- info --host 192.168.1.10
cargo run --bin qualisys-rt -- params --host 192.168.1.10 --parameters general,3d,6d,skeleton
cargo run --bin qualisys-rt -- frame --host 192.168.1.10 --components 6d
cargo run --bin qualisys-rt -- stream --host 192.168.1.10 --components 3d,6d --count 100
```

Use `--help` on the binary or any subcommand to see supported options.

## Quick Start

```rust,no_run
use qualisys_rust_sdk::prelude::*;
use qualisys_rust_sdk::rt::ComponentData;

fn main() -> qualisys_rust_sdk::Result<()> {
    let mut client = Client::connect("127.0.0.1", ClientOptions::default())?;

    let request = StreamFramesRequest::new(
        StreamRate::AllFrames,
        StreamTransport::Udp {
            bind_address: "0.0.0.0:0".parse().expect("valid UDP bind address"),
            destination: None,
        },
        [ComponentSelection::SixD],
    );

    client.start_stream_frames(&request)?;

    let mut accumulator = FrameAccumulator::for_components(request.components.clone());
    loop {
        match client.recv_stream_packet()? {
            StreamPacket::Data(packet) => {
                for frame in accumulator.push(packet) {
                    if let Some(component) = frame.component(ComponentType::SixD) {
                        if let ComponentData::SixD(bodies) = &component.data {
                            println!(
                                "frame={} bodies={}",
                                frame.frame_number,
                                bodies.bodies.len()
                            );
                        }
                    }
                }
            }
            StreamPacket::NoMoreData => break,
        }
    }

    Ok(())
}
```

## Mocap Metadata

The RT client can now fetch and parse the mocap-relevant XML settings needed to
describe a live project:

```rust,no_run
use qualisys_rust_sdk::prelude::*;

fn main() -> qualisys_rust_sdk::Result<()> {
    let mut client = Client::connect("127.0.0.1", ClientOptions::default())?;
    let parameters = client.get_mocap_parameters()?;

    if let Some(three_d) = &parameters.three_d {
        for label in &three_d.labels {
            println!("marker label: {}", label.name);
        }
    }

    Ok(())
}
```

Current parsed sections:

- `GetParameters 3D`
- `GetParameters 6D`
- `GetParameters Skeleton`

The parser accepts either the versioned `QTM_Parameters_Ver_*` envelope or one
of those section roots directly, which makes it easy to test with captured XML.

## Extension Roadmap

- Extend the XML settings models toward broader C++ SDK parity.
- Add remaining command helpers and parity-oriented examples.
- Add optional async wrappers while keeping the blocking core stable.
- Add conformance tests against captures and live QTM sessions.
