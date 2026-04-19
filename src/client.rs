use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::time::Duration;

use crate::error::{QtmError, Result};
use crate::packet::{decode_packet, parse_framed_packet, DataPacket, Packet};
use crate::parameters::{parse_mocap_parameters_xml, MocapParameters};
use crate::protocol::{
    build_get_current_frame_command, build_get_parameters_command, build_version_command,
    ComponentSelection, PacketType, ParameterSelection, ProtocolVersion, StreamFramesRequest,
    INITIAL_GREETING, LITTLE_ENDIAN_PORT,
};

#[derive(Debug, Clone, Copy)]
pub struct ConnectOptions {
    pub port: u16,
    pub version: ProtocolVersion,
    pub read_timeout: Duration,
    pub tcp_nodelay: bool,
}

impl Default for ConnectOptions {
    fn default() -> Self {
        Self {
            port: LITTLE_ENDIAN_PORT,
            version: ProtocolVersion::default(),
            read_timeout: Duration::from_secs(5),
            tcp_nodelay: true,
        }
    }
}

#[derive(Debug)]
pub enum StreamPacket {
    Data(DataPacket),
    NoMoreData,
}

pub struct QtmClient {
    command_stream: TcpStream,
    udp_socket: Option<UdpSocket>,
    version: ProtocolVersion,
    read_timeout: Duration,
}

impl QtmClient {
    pub fn connect(host: &str, options: ConnectOptions) -> Result<Self> {
        let command_stream = TcpStream::connect((host, options.port))?;
        command_stream.set_read_timeout(Some(options.read_timeout))?;
        command_stream.set_write_timeout(Some(options.read_timeout))?;
        if options.tcp_nodelay {
            command_stream.set_nodelay(true)?;
        }

        let mut client = Self {
            command_stream,
            udp_socket: None,
            version: options.version,
            read_timeout: options.read_timeout,
        };

        match client.read_packet()? {
            Packet::Command(message) if message.starts_with(INITIAL_GREETING) => {}
            Packet::Error(message) => return Err(QtmError::CommandFailed(message)),
            other => {
                return Err(QtmError::invalid_packet(format!(
                    "expected greeting packet, got {other:?}"
                )));
            }
        }

        let response = client.send_command_expect_text(&build_version_command(options.version))?;
        if !response.to_ascii_lowercase().starts_with("version set to ") {
            return Err(QtmError::invalid_packet(format!(
                "unexpected version response: {response}"
            )));
        }

        Ok(client)
    }

    pub fn version(&self) -> ProtocolVersion {
        self.version
    }

    pub fn udp_local_addr(&self) -> Result<Option<SocketAddr>> {
        self.udp_socket
            .as_ref()
            .map(UdpSocket::local_addr)
            .transpose()
            .map_err(Into::into)
    }

    pub fn qtm_version(&mut self) -> Result<String> {
        self.send_command_expect_text("qtmversion")
    }

    pub fn byte_order(&mut self) -> Result<String> {
        self.send_command_expect_text("byteorder")
    }

    pub fn get_parameters(&mut self, parameters: &[ParameterSelection]) -> Result<String> {
        self.send_command_expect_xml(&build_get_parameters_command(parameters))
    }

    pub fn get_mocap_parameters(&mut self) -> Result<MocapParameters> {
        let xml = self.get_parameters(&[
            ParameterSelection::ThreeD,
            ParameterSelection::SixD,
            ParameterSelection::Skeleton,
        ])?;
        parse_mocap_parameters_xml(&xml)
    }

    pub fn get_current_frame(&mut self, components: &[ComponentSelection]) -> Result<DataPacket> {
        self.send_command_expect_data(&build_get_current_frame_command(components))
    }

    pub fn send_xml(&mut self, xml: &str) -> Result<String> {
        self.write_text_packet(PacketType::Xml, xml)?;
        self.read_until(|packet| match packet {
            Packet::Command(message) => Some(Ok(message)),
            Packet::Error(message) => Some(Err(QtmError::CommandFailed(message))),
            Packet::Event(_) => None,
            other => Some(Err(QtmError::invalid_packet(format!(
                "expected XML command response, got {other:?}"
            )))),
        })
    }

    pub fn start_stream_frames(&mut self, request: &StreamFramesRequest) -> Result<()> {
        if let crate::protocol::StreamTransport::Udp { bind_address, .. } = &request.transport {
            let socket = UdpSocket::bind(bind_address)?;
            socket.set_read_timeout(Some(self.read_timeout))?;
            self.udp_socket = Some(socket);
        } else {
            self.udp_socket = None;
        }

        let resolved_udp_port = self
            .udp_socket
            .as_ref()
            .map(UdpSocket::local_addr)
            .transpose()?
            .map(|address| address.port());

        let command = request.to_command(resolved_udp_port);
        self.write_text_packet(PacketType::Command, &command)
    }

    pub fn stop_stream_frames(&mut self) -> Result<()> {
        self.write_text_packet(PacketType::Command, "streamframes stop")?;
        self.udp_socket = None;
        Ok(())
    }

    pub fn recv_stream_packet(&mut self) -> Result<StreamPacket> {
        if let Some(socket) = &self.udp_socket {
            let mut buffer = [0u8; u16::MAX as usize];
            let bytes_read = socket.recv(&mut buffer).map_err(map_timeout)?;
            let packet = parse_framed_packet(&buffer[..bytes_read])?;
            return stream_packet_from_packet(packet);
        }

        self.read_until(|packet| match packet {
            Packet::Data(data) => Some(Ok(StreamPacket::Data(data))),
            Packet::NoMoreData => Some(Ok(StreamPacket::NoMoreData)),
            Packet::Event(_) => None,
            Packet::Command(message) if message == INITIAL_GREETING => None,
            Packet::Error(message) => Some(Err(QtmError::CommandFailed(message))),
            other => Some(Err(QtmError::invalid_packet(format!(
                "expected streaming packet, got {other:?}"
            )))),
        })
    }

    pub fn next_packet(&mut self) -> Result<Packet> {
        self.read_packet()
    }

    fn send_command_expect_text(&mut self, command: &str) -> Result<String> {
        self.write_text_packet(PacketType::Command, command)?;
        self.read_until(|packet| match packet {
            Packet::Command(message) if message == INITIAL_GREETING => None,
            Packet::Command(message) => Some(Ok(message)),
            Packet::Error(message) => Some(Err(QtmError::CommandFailed(message))),
            Packet::Event(_) => None,
            other => Some(Err(QtmError::invalid_packet(format!(
                "expected command packet, got {other:?}"
            )))),
        })
    }

    fn send_command_expect_xml(&mut self, command: &str) -> Result<String> {
        self.write_text_packet(PacketType::Command, command)?;
        self.read_until(|packet| match packet {
            Packet::Xml(xml) => Some(Ok(xml)),
            Packet::Error(message) => Some(Err(QtmError::CommandFailed(message))),
            Packet::Event(_) => None,
            other => Some(Err(QtmError::invalid_packet(format!(
                "expected XML packet, got {other:?}"
            )))),
        })
    }

    fn send_command_expect_data(&mut self, command: &str) -> Result<DataPacket> {
        self.write_text_packet(PacketType::Command, command)?;
        self.read_until(|packet| match packet {
            Packet::Data(data) => Some(Ok(data)),
            Packet::Error(message) => Some(Err(QtmError::CommandFailed(message))),
            Packet::Event(_) => None,
            other => Some(Err(QtmError::invalid_packet(format!(
                "expected data packet, got {other:?}"
            )))),
        })
    }

    fn write_text_packet(&mut self, packet_type: PacketType, payload: &str) -> Result<()> {
        let payload_len = payload
            .len()
            .checked_add(1)
            .ok_or_else(|| QtmError::invalid_packet("command payload overflow"))?;
        let total_len = 8usize
            .checked_add(payload_len)
            .ok_or_else(|| QtmError::invalid_packet("packet length overflow"))?;
        let total_len = u32::try_from(total_len)
            .map_err(|_| QtmError::invalid_packet("packet length does not fit u32"))?;

        let mut frame = Vec::with_capacity(total_len as usize);
        frame.extend_from_slice(&total_len.to_le_bytes());
        frame.extend_from_slice(&(packet_type as u32).to_le_bytes());
        frame.extend_from_slice(payload.as_bytes());
        frame.push(0);
        self.command_stream.write_all(&frame).map_err(map_timeout)?;
        Ok(())
    }

    fn read_packet(&mut self) -> Result<Packet> {
        let mut header = [0u8; 8];
        self.command_stream
            .read_exact(&mut header)
            .map_err(map_timeout)?;
        let size = u32::from_le_bytes(header[..4].try_into().expect("slice has fixed size"));
        let packet_type_raw =
            u32::from_le_bytes(header[4..8].try_into().expect("slice has fixed size"));
        let packet_type =
            PacketType::try_from(packet_type_raw).map_err(QtmError::UnsupportedPacketType)?;
        let payload_size = size
            .checked_sub(8)
            .ok_or_else(|| QtmError::invalid_packet("packet size is shorter than header"))?;
        let payload_size = usize::try_from(payload_size)
            .map_err(|_| QtmError::invalid_packet("packet size does not fit usize"))?;
        let mut payload = vec![0u8; payload_size];
        self.command_stream
            .read_exact(&mut payload)
            .map_err(map_timeout)?;
        decode_packet(packet_type, &payload)
    }

    fn read_until<T>(
        &mut self,
        mut selector: impl FnMut(Packet) -> Option<Result<T>>,
    ) -> Result<T> {
        loop {
            let packet = self.read_packet()?;
            if let Some(result) = selector(packet) {
                return result;
            }
        }
    }
}

fn stream_packet_from_packet(packet: Packet) -> Result<StreamPacket> {
    match packet {
        Packet::Data(data) => Ok(StreamPacket::Data(data)),
        Packet::NoMoreData => Ok(StreamPacket::NoMoreData),
        Packet::Error(message) => Err(QtmError::CommandFailed(message)),
        other => Err(QtmError::invalid_packet(format!(
            "expected streaming packet, got {other:?}"
        ))),
    }
}

fn map_timeout(error: std::io::Error) -> QtmError {
    match error.kind() {
        ErrorKind::TimedOut | ErrorKind::WouldBlock => QtmError::Timeout,
        _ => error.into(),
    }
}
