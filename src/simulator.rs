use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::{QtmError, Result};
use crate::packet::{
    Body6D, Body6DResidual, Component, ComponentData, DataPacket, Packet, Point3, SixDComponent,
    SixDResidualComponent, encode_data_packet, encode_text_packet, parse_framed_packet,
};
use crate::protocol::{ComponentType, INITIAL_GREETING, PacketType};

#[derive(Debug, Clone)]
pub struct SimulatorOptions {
    pub bind_address: SocketAddr,
    pub frame_rate_hz: u32,
    pub rigid_body_count: usize,
}

impl Default for SimulatorOptions {
    fn default() -> Self {
        Self {
            bind_address: SocketAddr::from(([127, 0, 0, 1], crate::protocol::LITTLE_ENDIAN_PORT)),
            frame_rate_hz: 240,
            rigid_body_count: 1,
        }
    }
}

pub struct QtmSimulator {
    options: SimulatorOptions,
}

impl QtmSimulator {
    pub fn new(options: SimulatorOptions) -> Self {
        Self { options }
    }

    pub fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(self.options.bind_address)?;
        for stream in listener.incoming() {
            let stream = stream?;
            let options = self.options.clone();
            thread::spawn(move || {
                if let Err(error) = handle_client(stream, options) {
                    eprintln!("qualisys simulator client error: {error}");
                }
            });
        }
        Ok(())
    }
}

fn handle_client(mut stream: TcpStream, options: SimulatorOptions) -> Result<()> {
    stream.write_all(&encode_text_packet(PacketType::Command, INITIAL_GREETING)?)?;
    let mut active_stream: Option<Arc<AtomicBool>> = None;

    loop {
        let packet = match read_packet(&mut stream) {
            Ok(packet) => packet,
            Err(error) => {
                stop_stream(&mut active_stream);
                return Err(error);
            }
        };
        let Packet::Command(command) = packet else {
            stream.write_all(&encode_text_packet(
                PacketType::Error,
                "simulator only accepts command packets",
            )?)?;
            continue;
        };

        let command_lower = command.to_ascii_lowercase();
        if command_lower.starts_with("version ") {
            stream.write_all(&encode_text_packet(
                PacketType::Command,
                "Version set to 1.27",
            )?)?;
        } else if command_lower == "qtmversion" {
            stream.write_all(&encode_text_packet(
                PacketType::Command,
                "QTM RT simulator 0.1",
            )?)?;
        } else if command_lower == "byteorder" {
            stream.write_all(&encode_text_packet(PacketType::Command, "little endian")?)?;
        } else if command_lower.starts_with("getparameters") {
            stream.write_all(&encode_text_packet(
                PacketType::Xml,
                &parameters_xml(&options),
            )?)?;
        } else if command_lower.starts_with("getcurrentframe") {
            let components = requested_components(&command);
            let frame = simulated_packet(Instant::now(), 1, options.rigid_body_count, &components);
            stream.write_all(&encode_data_packet(&frame)?)?;
        } else if command_lower == "streamframes stop" {
            stop_stream(&mut active_stream);
        } else if command_lower.starts_with("streamframes ") {
            stop_stream(&mut active_stream);
            let streaming = Arc::new(AtomicBool::new(true));
            start_stream(&command, &stream, &options, Arc::clone(&streaming))?;
            active_stream = Some(streaming);
        } else {
            stream.write_all(&encode_text_packet(
                PacketType::Error,
                &format!("unsupported simulator command: {command}"),
            )?)?;
        }
    }
}

fn stop_stream(active_stream: &mut Option<Arc<AtomicBool>>) {
    if let Some(streaming) = active_stream.take() {
        streaming.store(false, Ordering::SeqCst);
    }
}

fn start_stream(
    command: &str,
    stream: &TcpStream,
    options: &SimulatorOptions,
    streaming: Arc<AtomicBool>,
) -> Result<()> {
    let tcp_stream = stream.try_clone()?;
    let peer_addr = stream.peer_addr()?;
    let udp_target = stream_udp_target(command, peer_addr);
    let components = requested_components(command);
    let options = options.clone();

    thread::spawn(move || {
        let result = match udp_target {
            Some(target) => stream_udp(target, options, components, streaming),
            None => stream_tcp(tcp_stream, options, components, streaming),
        };
        if let Err(error) = result {
            eprintln!("qualisys simulator stream error: {error}");
        }
    });

    Ok(())
}

fn stream_udp(
    target: SocketAddr,
    options: SimulatorOptions,
    components: Vec<SimulatedComponent>,
    streaming: Arc<AtomicBool>,
) -> Result<()> {
    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0)))?;
    stream_frames(options, components, streaming, |bytes| {
        socket.send_to(bytes, target)?;
        Ok(())
    })
}

fn stream_tcp(
    mut stream: TcpStream,
    options: SimulatorOptions,
    components: Vec<SimulatedComponent>,
    streaming: Arc<AtomicBool>,
) -> Result<()> {
    stream_frames(options, components, streaming, |bytes| {
        stream.write_all(bytes)?;
        Ok(())
    })
}

fn stream_frames<F>(
    options: SimulatorOptions,
    components: Vec<SimulatedComponent>,
    streaming: Arc<AtomicBool>,
    mut send: F,
) -> Result<()>
where
    F: FnMut(&[u8]) -> Result<()>,
{
    let start = Instant::now();
    let frame_period = Duration::from_secs_f64(1.0 / f64::from(options.frame_rate_hz.max(1)));
    let mut frame_number = 1u32;

    while streaming.load(Ordering::SeqCst) {
        let packet = simulated_packet(start, frame_number, options.rigid_body_count, &components);
        let bytes = encode_data_packet(&packet)?;
        send(&bytes)?;
        frame_number = frame_number.wrapping_add(1);
        thread::sleep(frame_period);
    }

    Ok(())
}

fn read_packet(stream: &mut TcpStream) -> Result<Packet> {
    let mut header = [0u8; 8];
    stream.read_exact(&mut header).map_err(map_closed)?;
    let size = u32::from_le_bytes(header[..4].try_into().expect("fixed header"));
    let payload_size = size
        .checked_sub(8)
        .ok_or_else(|| QtmError::invalid_packet("packet size is shorter than header"))?;
    let payload_size = usize::try_from(payload_size)
        .map_err(|_| QtmError::invalid_packet("packet size does not fit usize"))?;
    let mut bytes = Vec::with_capacity(8 + payload_size);
    bytes.extend_from_slice(&header);
    bytes.resize(8 + payload_size, 0);
    stream.read_exact(&mut bytes[8..]).map_err(map_closed)?;
    parse_framed_packet(&bytes)
}

fn map_closed(error: std::io::Error) -> QtmError {
    if matches!(
        error.kind(),
        ErrorKind::UnexpectedEof | ErrorKind::ConnectionAborted | ErrorKind::ConnectionReset
    ) {
        return QtmError::invalid_packet("client connection closed");
    }
    error.into()
}

fn stream_udp_target(command: &str, peer_addr: SocketAddr) -> Option<SocketAddr> {
    command.split_whitespace().find_map(|part| {
        let udp = part.strip_prefix("udp:")?;
        let mut parts = udp.rsplitn(2, ':');
        let port = parts.next()?.parse::<u16>().ok()?;
        let host = parts.next();
        match host {
            Some(host) => format!("{host}:{port}").parse().ok(),
            None => Some(SocketAddr::new(peer_addr.ip(), port)),
        }
    })
}

#[derive(Debug, Clone, Copy)]
enum SimulatedComponent {
    SixD,
    SixDResidual,
}

impl SimulatedComponent {
    fn from_command_token(token: &str) -> Option<Self> {
        match token.trim().to_ascii_lowercase().as_str() {
            "6d" => Some(Self::SixD),
            "6dres" | "6dresidual" => Some(Self::SixDResidual),
            _ => None,
        }
    }
}

fn requested_components(command: &str) -> Vec<SimulatedComponent> {
    let components = command
        .split_whitespace()
        .filter_map(SimulatedComponent::from_command_token)
        .collect::<Vec<_>>();

    if components.is_empty() {
        return vec![SimulatedComponent::SixDResidual];
    }

    components
}

fn simulated_packet(
    start: Instant,
    frame_number: u32,
    rigid_body_count: usize,
    components: &[SimulatedComponent],
) -> DataPacket {
    let elapsed = start.elapsed();
    let timestamp = elapsed.as_micros() as u64;
    let t = elapsed.as_secs_f32();
    let bodies = (0..rigid_body_count)
        .map(|index| simulated_body(index as i32 + 1, t + index as f32 * 0.5))
        .collect::<Vec<_>>();

    let components = components
        .iter()
        .copied()
        .map(|component| simulated_component(component, &bodies))
        .collect();

    DataPacket {
        timestamp,
        frame_number,
        components,
    }
}

fn simulated_component(component: SimulatedComponent, bodies: &[Body6DResidual]) -> Component {
    match component {
        SimulatedComponent::SixD => Component {
            id: ComponentType::SixD as u32,
            data: ComponentData::SixD(SixDComponent {
                drop_rate: 0,
                out_of_sync_rate: 0,
                bodies: bodies
                    .iter()
                    .map(|body| Body6D {
                        position: body.position.clone(),
                        rotation_matrix: body.rotation_matrix,
                    })
                    .collect(),
            }),
        },
        SimulatedComponent::SixDResidual => Component {
            id: ComponentType::SixDResidual as u32,
            data: ComponentData::SixDResidual(SixDResidualComponent {
                drop_rate: 0,
                out_of_sync_rate: 0,
                bodies: bodies.to_vec(),
            }),
        },
    }
}

fn simulated_body(id: i32, t: f32) -> Body6DResidual {
    let yaw = 0.35 * t;
    let (sin_yaw, cos_yaw) = yaw.sin_cos();

    Body6DResidual {
        position: Point3 {
            x: 750.0 * t.cos(),
            y: 750.0 * t.sin(),
            z: 1000.0 + 100.0 * (2.0 * t).sin(),
        },
        rotation_matrix: [cos_yaw, -sin_yaw, 0.0, sin_yaw, cos_yaw, 0.0, 0.0, 0.0, 1.0],
        residual: 0.001 * id as f32,
    }
}

fn parameters_xml(options: &SimulatorOptions) -> String {
    let mut xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<QTM_Parameters_Ver_1.27>
  <General>
    <Frequency>{}</Frequency>
  </General>
  <The_6D>
"#,
        options.frame_rate_hz.max(1)
    );

    for index in 0..options.rigid_body_count {
        xml.push_str(&format!(
            r#"    <Body>
      <Name>sim_body_{}</Name>
      <Color R="0" G="255" B="0" />
      <Enabled>true</Enabled>
    </Body>
"#,
            index + 1
        ));
    }

    xml.push_str(
        r#"  </The_6D>
</QTM_Parameters_Ver_1.27>
"#,
    );
    xml
}
