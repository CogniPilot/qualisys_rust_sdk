use std::convert::TryInto;

use crate::error::{QtmError, Result};
use crate::protocol::{ComponentType, Event, PacketType};

#[derive(Debug, Clone, PartialEq)]
pub struct PacketHeader {
    pub size: u32,
    pub packet_type: PacketType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Error(String),
    Command(String),
    Xml(String),
    Data(DataPacket),
    NoMoreData,
    Event(Event),
    Binary {
        packet_type: PacketType,
        payload: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataPacket {
    pub timestamp: u64,
    pub frame_number: u32,
    pub components: Vec<Component>,
}

impl DataPacket {
    pub fn component(&self, component_type: ComponentType) -> Option<&Component> {
        self.components
            .iter()
            .find(|component| component.id == component_type as u32)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub id: u32,
    pub data: ComponentData,
}

impl Component {
    pub fn component_type(&self) -> Option<ComponentType> {
        ComponentType::try_from(self.id).ok()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentData {
    TwoD(TwoDComponent),
    TwoDLinearized(TwoDComponent),
    ThreeD(ThreeDComponent),
    ThreeDResidual(ThreeDResidualComponent),
    ThreeDNoLabels(ThreeDNoLabelsComponent),
    ThreeDNoLabelsResidual(ThreeDNoLabelsResidualComponent),
    SixD(SixDComponent),
    SixDResidual(SixDResidualComponent),
    SixDEuler(SixDEulerComponent),
    SixDEulerResidual(SixDEulerResidualComponent),
    Analog(AnalogComponent),
    AnalogSingle(AnalogSingleComponent),
    Force(ForceComponent),
    ForceSingle(ForceSingleComponent),
    GazeVector(GazeVectorComponent),
    EyeTracker(EyeTrackerComponent),
    Image(ImageComponent),
    Timecode(TimecodeComponent),
    Skeleton(SkeletonComponent),
    Raw(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Marker2D {
    pub x: i32,
    pub y: i32,
    pub diameter_x: i16,
    pub diameter_y: i16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Camera2D {
    pub status_flag: u8,
    pub markers: Vec<Marker2D>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TwoDComponent {
    pub drop_rate: i16,
    pub out_of_sync_rate: i16,
    pub cameras: Vec<Camera2D>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Point3Residual {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub residual: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Point3NoLabel {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Point3NoLabelResidual {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub id: i32,
    pub residual: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThreeDComponent {
    pub drop_rate: i16,
    pub out_of_sync_rate: i16,
    pub markers: Vec<Point3>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThreeDResidualComponent {
    pub drop_rate: i16,
    pub out_of_sync_rate: i16,
    pub markers: Vec<Point3Residual>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThreeDNoLabelsComponent {
    pub drop_rate: i16,
    pub out_of_sync_rate: i16,
    pub markers: Vec<Point3NoLabel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThreeDNoLabelsResidualComponent {
    pub drop_rate: i16,
    pub out_of_sync_rate: i16,
    pub markers: Vec<Point3NoLabelResidual>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Body6D {
    pub position: Point3,
    pub rotation_matrix: [f32; 9],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Body6DResidual {
    pub position: Point3,
    pub rotation_matrix: [f32; 9],
    pub residual: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Body6DEuler {
    pub position: Point3,
    pub euler: [f32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Body6DEulerResidual {
    pub position: Point3,
    pub euler: [f32; 3],
    pub residual: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SixDComponent {
    pub drop_rate: i16,
    pub out_of_sync_rate: i16,
    pub bodies: Vec<Body6D>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SixDResidualComponent {
    pub drop_rate: i16,
    pub out_of_sync_rate: i16,
    pub bodies: Vec<Body6DResidual>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SixDEulerComponent {
    pub drop_rate: i16,
    pub out_of_sync_rate: i16,
    pub bodies: Vec<Body6DEuler>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SixDEulerResidualComponent {
    pub drop_rate: i16,
    pub out_of_sync_rate: i16,
    pub bodies: Vec<Body6DEulerResidual>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnalogDevice {
    pub device_id: i32,
    pub channel_count: u32,
    pub sample_count: u32,
    pub sample_number: Option<i32>,
    pub channels: Vec<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnalogComponent {
    pub devices: Vec<AnalogDevice>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnalogSingleDevice {
    pub device_id: i32,
    pub channel_count: u32,
    pub samples: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AnalogSingleComponent {
    pub devices: Vec<AnalogSingleDevice>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForceSample {
    pub force: [f32; 3],
    pub moment: [f32; 3],
    pub application_point: [f32; 3],
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForcePlate {
    pub plate_id: i32,
    pub force_number: i32,
    pub samples: Vec<ForceSample>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForceComponent {
    pub plates: Vec<ForcePlate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForceSinglePlate {
    pub plate_id: i32,
    pub sample: ForceSample,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForceSingleComponent {
    pub plates: Vec<ForceSinglePlate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GazeVectorSample {
    pub direction: Point3,
    pub position: Point3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GazeVectorSeries {
    pub sample_number: i32,
    pub samples: Vec<GazeVectorSample>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GazeVectorComponent {
    pub vectors: Vec<GazeVectorSeries>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EyeTrackerSample {
    pub left_pupil_diameter: f32,
    pub right_pupil_diameter: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EyeTrackerSeries {
    pub sample_number: i32,
    pub samples: Vec<EyeTrackerSample>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EyeTrackerComponent {
    pub trackers: Vec<EyeTrackerSeries>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub camera_id: i32,
    pub format: i32,
    pub width: i32,
    pub height: i32,
    pub crop_left: f32,
    pub crop_top: f32,
    pub crop_right: f32,
    pub crop_bottom: f32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageComponent {
    pub images: Vec<Image>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimecodeEntry {
    pub timecode_type: i32,
    pub high: u32,
    pub low: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimecodeComponent {
    pub entries: Vec<TimecodeEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkeletonSegment {
    pub id: i32,
    pub position: Point3,
    pub rotation: Quaternion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Skeleton {
    pub segments: Vec<SkeletonSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkeletonComponent {
    pub skeletons: Vec<Skeleton>,
}

pub fn parse_framed_packet(bytes: &[u8]) -> Result<Packet> {
    if bytes.len() < 8 {
        return Err(QtmError::invalid_packet(
            "packet is shorter than 8-byte header",
        ));
    }

    let header = parse_packet_header(&bytes[..8])?;
    let expected_len = usize::try_from(header.size)
        .map_err(|_| QtmError::invalid_packet("packet size does not fit usize"))?;

    if expected_len != bytes.len() {
        return Err(QtmError::invalid_packet(format!(
            "header size {} does not match buffer length {}",
            header.size,
            bytes.len()
        )));
    }

    decode_packet(header.packet_type, &bytes[8..])
}

pub fn parse_packet_header(bytes: &[u8]) -> Result<PacketHeader> {
    let size = u32::from_le_bytes(read_fixed::<4>(bytes, 0)?);
    let packet_type_raw = u32::from_le_bytes(read_fixed::<4>(bytes, 4)?);
    let packet_type =
        PacketType::try_from(packet_type_raw).map_err(QtmError::UnsupportedPacketType)?;

    Ok(PacketHeader { size, packet_type })
}

pub fn decode_packet(packet_type: PacketType, payload: &[u8]) -> Result<Packet> {
    match packet_type {
        PacketType::Error => Ok(Packet::Error(parse_string_payload(payload)?)),
        PacketType::Command => Ok(Packet::Command(parse_string_payload(payload)?)),
        PacketType::Xml => Ok(Packet::Xml(parse_string_payload(payload)?)),
        PacketType::Data => Ok(Packet::Data(parse_data_packet(payload)?)),
        PacketType::NoMoreData => Ok(Packet::NoMoreData),
        PacketType::Event => {
            let event_byte = *payload
                .first()
                .ok_or_else(|| QtmError::invalid_packet("event packet payload is empty"))?;
            let event = Event::try_from(event_byte).map_err(QtmError::UnsupportedEvent)?;
            Ok(Packet::Event(event))
        }
        other => Ok(Packet::Binary {
            packet_type: other,
            payload: payload.to_vec(),
        }),
    }
}

pub fn encode_framed_packet(packet_type: PacketType, payload: &[u8]) -> Result<Vec<u8>> {
    let total_len = 8usize
        .checked_add(payload.len())
        .ok_or_else(|| QtmError::invalid_packet("packet length overflow"))?;
    let total_len = u32::try_from(total_len)
        .map_err(|_| QtmError::invalid_packet("packet length does not fit u32"))?;

    let mut bytes = Vec::with_capacity(total_len as usize);
    bytes.extend_from_slice(&total_len.to_le_bytes());
    bytes.extend_from_slice(&(packet_type as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

pub fn encode_text_packet(packet_type: PacketType, text: &str) -> Result<Vec<u8>> {
    let mut payload = Vec::with_capacity(text.len() + 1);
    payload.extend_from_slice(text.as_bytes());
    payload.push(0);
    encode_framed_packet(packet_type, &payload)
}

pub fn encode_data_packet(packet: &DataPacket) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&packet.timestamp.to_le_bytes());
    payload.extend_from_slice(&packet.frame_number.to_le_bytes());
    payload.extend_from_slice(
        &u32::try_from(packet.components.len())
            .map_err(|_| QtmError::invalid_packet("component count does not fit u32"))?
            .to_le_bytes(),
    );

    for component in &packet.components {
        let component_payload = encode_component_data(&component.data)?;
        let component_size = 8usize
            .checked_add(component_payload.len())
            .ok_or_else(|| QtmError::invalid_packet("component length overflow"))?;
        let component_size = u32::try_from(component_size)
            .map_err(|_| QtmError::invalid_packet("component length does not fit u32"))?;
        payload.extend_from_slice(&component_size.to_le_bytes());
        payload.extend_from_slice(&component.id.to_le_bytes());
        payload.extend_from_slice(&component_payload);
    }

    encode_framed_packet(PacketType::Data, &payload)
}

fn encode_component_data(data: &ComponentData) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    match data {
        ComponentData::ThreeD(component) => {
            bytes.extend_from_slice(
                &u32::try_from(component.markers.len())
                    .map_err(|_| QtmError::invalid_packet("3D marker count does not fit u32"))?
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(&component.drop_rate.to_le_bytes());
            bytes.extend_from_slice(&component.out_of_sync_rate.to_le_bytes());
            for marker in &component.markers {
                write_point3(&mut bytes, marker);
            }
        }
        ComponentData::ThreeDNoLabels(component) => {
            bytes.extend_from_slice(
                &u32::try_from(component.markers.len())
                    .map_err(|_| QtmError::invalid_packet("3D marker count does not fit u32"))?
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(&component.drop_rate.to_le_bytes());
            bytes.extend_from_slice(&component.out_of_sync_rate.to_le_bytes());
            for marker in &component.markers {
                bytes.extend_from_slice(&marker.x.to_le_bytes());
                bytes.extend_from_slice(&marker.y.to_le_bytes());
                bytes.extend_from_slice(&marker.z.to_le_bytes());
                bytes.extend_from_slice(&marker.id.to_le_bytes());
            }
        }
        ComponentData::SixD(component) => {
            bytes.extend_from_slice(
                &i32::try_from(component.bodies.len())
                    .map_err(|_| QtmError::invalid_packet("6D body count does not fit i32"))?
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(&component.drop_rate.to_le_bytes());
            bytes.extend_from_slice(&component.out_of_sync_rate.to_le_bytes());
            for body in &component.bodies {
                write_point3(&mut bytes, &body.position);
                for value in body.rotation_matrix {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        ComponentData::SixDResidual(component) => {
            bytes.extend_from_slice(
                &i32::try_from(component.bodies.len())
                    .map_err(|_| QtmError::invalid_packet("6D body count does not fit i32"))?
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(&component.drop_rate.to_le_bytes());
            bytes.extend_from_slice(&component.out_of_sync_rate.to_le_bytes());
            for body in &component.bodies {
                write_point3(&mut bytes, &body.position);
                for value in body.rotation_matrix {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
                bytes.extend_from_slice(&body.residual.to_le_bytes());
            }
        }
        ComponentData::SixDEuler(component) => {
            bytes.extend_from_slice(
                &i32::try_from(component.bodies.len())
                    .map_err(|_| QtmError::invalid_packet("6D body count does not fit i32"))?
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(&component.drop_rate.to_le_bytes());
            bytes.extend_from_slice(&component.out_of_sync_rate.to_le_bytes());
            for body in &component.bodies {
                write_point3(&mut bytes, &body.position);
                for value in body.euler {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        ComponentData::SixDEulerResidual(component) => {
            bytes.extend_from_slice(
                &i32::try_from(component.bodies.len())
                    .map_err(|_| QtmError::invalid_packet("6D body count does not fit i32"))?
                    .to_le_bytes(),
            );
            bytes.extend_from_slice(&component.drop_rate.to_le_bytes());
            bytes.extend_from_slice(&component.out_of_sync_rate.to_le_bytes());
            for body in &component.bodies {
                write_point3(&mut bytes, &body.position);
                for value in body.euler {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
                bytes.extend_from_slice(&body.residual.to_le_bytes());
            }
        }
        ComponentData::Raw(raw) => bytes.extend_from_slice(raw),
        other => {
            return Err(QtmError::invalid_packet(format!(
                "encoding {other:?} is not implemented"
            )));
        }
    }
    Ok(bytes)
}

fn write_point3(bytes: &mut Vec<u8>, point: &Point3) {
    bytes.extend_from_slice(&point.x.to_le_bytes());
    bytes.extend_from_slice(&point.y.to_le_bytes());
    bytes.extend_from_slice(&point.z.to_le_bytes());
}

fn parse_string_payload(payload: &[u8]) -> Result<String> {
    let payload = payload.strip_suffix(&[0]).unwrap_or(payload);
    String::from_utf8(payload.to_vec()).map_err(Into::into)
}

fn parse_data_packet(payload: &[u8]) -> Result<DataPacket> {
    let mut cursor = Cursor::new(payload);
    let timestamp = cursor.read_u64()?;
    let frame_number = cursor.read_u32()?;
    let component_count = cursor.read_u32()?;

    let mut components = Vec::with_capacity(
        usize::try_from(component_count)
            .map_err(|_| QtmError::invalid_packet("component count does not fit usize"))?,
    );

    for _ in 0..component_count {
        let component_size = cursor.read_u32()?;
        let component_id = cursor.read_u32()?;
        let payload_len = component_size
            .checked_sub(8)
            .ok_or_else(|| QtmError::invalid_packet("component size is shorter than header"))?;
        let payload_len = usize::try_from(payload_len)
            .map_err(|_| QtmError::invalid_packet("component size does not fit usize"))?;
        let component_payload = cursor.take(payload_len)?;
        let data = parse_component(component_id, component_payload)?;
        components.push(Component {
            id: component_id,
            data,
        });
    }

    cursor.expect_exhausted("data packet")?;

    Ok(DataPacket {
        timestamp,
        frame_number,
        components,
    })
}

fn parse_component(component_id: u32, payload: &[u8]) -> Result<ComponentData> {
    let data = match ComponentType::try_from(component_id) {
        Ok(ComponentType::TwoD) => ComponentData::TwoD(parse_2d_component(payload)?),
        Ok(ComponentType::TwoDLinearized) => {
            ComponentData::TwoDLinearized(parse_2d_component(payload)?)
        }
        Ok(ComponentType::ThreeD) => ComponentData::ThreeD(parse_3d_component(payload)?),
        Ok(ComponentType::ThreeDResidual) => {
            ComponentData::ThreeDResidual(parse_3d_residual_component(payload)?)
        }
        Ok(ComponentType::ThreeDNoLabels) => {
            ComponentData::ThreeDNoLabels(parse_3d_nolabels_component(payload)?)
        }
        Ok(ComponentType::ThreeDNoLabelsResidual) => {
            ComponentData::ThreeDNoLabelsResidual(parse_3d_nolabels_residual_component(payload)?)
        }
        Ok(ComponentType::SixD) => ComponentData::SixD(parse_6d_component(payload)?),
        Ok(ComponentType::SixDResidual) => {
            ComponentData::SixDResidual(parse_6d_residual_component(payload)?)
        }
        Ok(ComponentType::SixDEuler) => {
            ComponentData::SixDEuler(parse_6d_euler_component(payload)?)
        }
        Ok(ComponentType::SixDEulerResidual) => {
            ComponentData::SixDEulerResidual(parse_6d_euler_residual_component(payload)?)
        }
        Ok(ComponentType::Analog) => ComponentData::Analog(parse_analog_component(payload)?),
        Ok(ComponentType::AnalogSingle) => {
            ComponentData::AnalogSingle(parse_analog_single_component(payload)?)
        }
        Ok(ComponentType::Force) => ComponentData::Force(parse_force_component(payload)?),
        Ok(ComponentType::ForceSingle) => {
            ComponentData::ForceSingle(parse_force_single_component(payload)?)
        }
        Ok(ComponentType::GazeVector) => {
            ComponentData::GazeVector(parse_gaze_vector_component(payload)?)
        }
        Ok(ComponentType::EyeTracker) => {
            ComponentData::EyeTracker(parse_eye_tracker_component(payload)?)
        }
        Ok(ComponentType::Image) => ComponentData::Image(parse_image_component(payload)?),
        Ok(ComponentType::Timecode) => ComponentData::Timecode(parse_timecode_component(payload)?),
        Ok(ComponentType::Skeleton) => ComponentData::Skeleton(parse_skeleton_component(payload)?),
        Err(_) => ComponentData::Raw(payload.to_vec()),
    };
    Ok(data)
}

fn parse_2d_component(payload: &[u8]) -> Result<TwoDComponent> {
    let mut cursor = Cursor::new(payload);
    let camera_count = cursor.read_u32()?;
    let drop_rate = cursor.read_i16()?;
    let out_of_sync_rate = cursor.read_i16()?;

    let mut cameras = Vec::with_capacity(to_usize(camera_count)?);
    for _ in 0..camera_count {
        let marker_count = cursor.read_i32()?;
        let marker_count = to_usize_non_negative(marker_count)?;
        let status_flag = cursor.read_u8()?;
        let mut markers = Vec::with_capacity(marker_count);
        for _ in 0..marker_count {
            markers.push(Marker2D {
                x: cursor.read_i32()?,
                y: cursor.read_i32()?,
                diameter_x: cursor.read_i16()?,
                diameter_y: cursor.read_i16()?,
            });
        }
        cameras.push(Camera2D {
            status_flag,
            markers,
        });
    }

    cursor.expect_exhausted("2D component")?;
    Ok(TwoDComponent {
        drop_rate,
        out_of_sync_rate,
        cameras,
    })
}

fn parse_3d_component(payload: &[u8]) -> Result<ThreeDComponent> {
    let mut cursor = Cursor::new(payload);
    let marker_count = cursor.read_u32()?;
    let drop_rate = cursor.read_i16()?;
    let out_of_sync_rate = cursor.read_i16()?;

    let mut markers = Vec::with_capacity(to_usize(marker_count)?);
    for _ in 0..marker_count {
        markers.push(cursor.read_point3()?);
    }

    cursor.expect_exhausted("3D component")?;
    Ok(ThreeDComponent {
        drop_rate,
        out_of_sync_rate,
        markers,
    })
}

fn parse_3d_residual_component(payload: &[u8]) -> Result<ThreeDResidualComponent> {
    let mut cursor = Cursor::new(payload);
    let marker_count = cursor.read_u32()?;
    let drop_rate = cursor.read_i16()?;
    let out_of_sync_rate = cursor.read_i16()?;

    let mut markers = Vec::with_capacity(to_usize(marker_count)?);
    for _ in 0..marker_count {
        markers.push(Point3Residual {
            x: cursor.read_f32()?,
            y: cursor.read_f32()?,
            z: cursor.read_f32()?,
            residual: cursor.read_f32()?,
        });
    }

    cursor.expect_exhausted("3D residual component")?;
    Ok(ThreeDResidualComponent {
        drop_rate,
        out_of_sync_rate,
        markers,
    })
}

fn parse_3d_nolabels_component(payload: &[u8]) -> Result<ThreeDNoLabelsComponent> {
    let mut cursor = Cursor::new(payload);
    let marker_count = cursor.read_u32()?;
    let drop_rate = cursor.read_i16()?;
    let out_of_sync_rate = cursor.read_i16()?;

    let mut markers = Vec::with_capacity(to_usize(marker_count)?);
    for _ in 0..marker_count {
        markers.push(Point3NoLabel {
            x: cursor.read_f32()?,
            y: cursor.read_f32()?,
            z: cursor.read_f32()?,
            id: cursor.read_i32()?,
        });
    }

    cursor.expect_exhausted("3D no-label component")?;
    Ok(ThreeDNoLabelsComponent {
        drop_rate,
        out_of_sync_rate,
        markers,
    })
}

fn parse_3d_nolabels_residual_component(payload: &[u8]) -> Result<ThreeDNoLabelsResidualComponent> {
    let mut cursor = Cursor::new(payload);
    let marker_count = cursor.read_u32()?;
    let drop_rate = cursor.read_i16()?;
    let out_of_sync_rate = cursor.read_i16()?;

    let mut markers = Vec::with_capacity(to_usize(marker_count)?);
    for _ in 0..marker_count {
        markers.push(Point3NoLabelResidual {
            x: cursor.read_f32()?,
            y: cursor.read_f32()?,
            z: cursor.read_f32()?,
            id: cursor.read_i32()?,
            residual: cursor.read_f32()?,
        });
    }

    cursor.expect_exhausted("3D no-label residual component")?;
    Ok(ThreeDNoLabelsResidualComponent {
        drop_rate,
        out_of_sync_rate,
        markers,
    })
}

fn parse_6d_component(payload: &[u8]) -> Result<SixDComponent> {
    let mut cursor = Cursor::new(payload);
    let body_count = cursor.read_i32()?;
    let body_count = to_usize_non_negative(body_count)?;
    let drop_rate = cursor.read_i16()?;
    let out_of_sync_rate = cursor.read_i16()?;

    let mut bodies = Vec::with_capacity(body_count);
    for _ in 0..body_count {
        bodies.push(Body6D {
            position: cursor.read_point3()?,
            rotation_matrix: cursor.read_f32_array::<9>()?,
        });
    }

    cursor.expect_exhausted("6D component")?;
    Ok(SixDComponent {
        drop_rate,
        out_of_sync_rate,
        bodies,
    })
}

fn parse_6d_residual_component(payload: &[u8]) -> Result<SixDResidualComponent> {
    let mut cursor = Cursor::new(payload);
    let body_count = cursor.read_i32()?;
    let body_count = to_usize_non_negative(body_count)?;
    let drop_rate = cursor.read_i16()?;
    let out_of_sync_rate = cursor.read_i16()?;

    let mut bodies = Vec::with_capacity(body_count);
    for _ in 0..body_count {
        bodies.push(Body6DResidual {
            position: cursor.read_point3()?,
            rotation_matrix: cursor.read_f32_array::<9>()?,
            residual: cursor.read_f32()?,
        });
    }

    cursor.expect_exhausted("6D residual component")?;
    Ok(SixDResidualComponent {
        drop_rate,
        out_of_sync_rate,
        bodies,
    })
}

fn parse_6d_euler_component(payload: &[u8]) -> Result<SixDEulerComponent> {
    let mut cursor = Cursor::new(payload);
    let body_count = cursor.read_i32()?;
    let body_count = to_usize_non_negative(body_count)?;
    let drop_rate = cursor.read_i16()?;
    let out_of_sync_rate = cursor.read_i16()?;

    let mut bodies = Vec::with_capacity(body_count);
    for _ in 0..body_count {
        bodies.push(Body6DEuler {
            position: cursor.read_point3()?,
            euler: cursor.read_f32_array::<3>()?,
        });
    }

    cursor.expect_exhausted("6D euler component")?;
    Ok(SixDEulerComponent {
        drop_rate,
        out_of_sync_rate,
        bodies,
    })
}

fn parse_6d_euler_residual_component(payload: &[u8]) -> Result<SixDEulerResidualComponent> {
    let mut cursor = Cursor::new(payload);
    let body_count = cursor.read_i32()?;
    let body_count = to_usize_non_negative(body_count)?;
    let drop_rate = cursor.read_i16()?;
    let out_of_sync_rate = cursor.read_i16()?;

    let mut bodies = Vec::with_capacity(body_count);
    for _ in 0..body_count {
        bodies.push(Body6DEulerResidual {
            position: cursor.read_point3()?,
            euler: cursor.read_f32_array::<3>()?,
            residual: cursor.read_f32()?,
        });
    }

    cursor.expect_exhausted("6D euler residual component")?;
    Ok(SixDEulerResidualComponent {
        drop_rate,
        out_of_sync_rate,
        bodies,
    })
}

fn parse_analog_component(payload: &[u8]) -> Result<AnalogComponent> {
    let mut cursor = Cursor::new(payload);
    let device_count = cursor.read_i32()?;
    let device_count = to_usize_non_negative(device_count)?;

    let mut devices = Vec::with_capacity(device_count);
    for _ in 0..device_count {
        let device_id = cursor.read_i32()?;
        let channel_count = cursor.read_i32()?;
        let channel_count_usize = to_usize_non_negative(channel_count)?;
        let sample_count = cursor.read_i32()?;
        let sample_count_usize = to_usize_non_negative(sample_count)?;

        let sample_number = if sample_count_usize > 0 {
            Some(cursor.read_i32()?)
        } else {
            None
        };

        let mut channels = Vec::with_capacity(channel_count_usize);
        for _ in 0..channel_count_usize {
            let mut samples = Vec::with_capacity(sample_count_usize);
            for _ in 0..sample_count_usize {
                samples.push(cursor.read_f32()?);
            }
            channels.push(samples);
        }

        devices.push(AnalogDevice {
            device_id,
            channel_count: channel_count_usize as u32,
            sample_count: sample_count_usize as u32,
            sample_number,
            channels,
        });
    }

    cursor.expect_exhausted("analog component")?;
    Ok(AnalogComponent { devices })
}

fn parse_analog_single_component(payload: &[u8]) -> Result<AnalogSingleComponent> {
    let mut cursor = Cursor::new(payload);
    let device_count = cursor.read_i32()?;
    let device_count = to_usize_non_negative(device_count)?;

    let mut devices = Vec::with_capacity(device_count);
    for _ in 0..device_count {
        let device_id = cursor.read_i32()?;
        let channel_count = cursor.read_i32()?;
        let channel_count_usize = to_usize_non_negative(channel_count)?;

        let mut samples = Vec::with_capacity(channel_count_usize);
        for _ in 0..channel_count_usize {
            samples.push(cursor.read_f32()?);
        }

        devices.push(AnalogSingleDevice {
            device_id,
            channel_count: channel_count_usize as u32,
            samples,
        });
    }

    cursor.expect_exhausted("analog single component")?;
    Ok(AnalogSingleComponent { devices })
}

fn parse_force_component(payload: &[u8]) -> Result<ForceComponent> {
    let mut cursor = Cursor::new(payload);
    let plate_count = cursor.read_i32()?;
    let plate_count = to_usize_non_negative(plate_count)?;

    let mut plates = Vec::with_capacity(plate_count);
    for _ in 0..plate_count {
        let plate_id = cursor.read_i32()?;
        let force_count = cursor.read_i32()?;
        let force_count_usize = to_usize_non_negative(force_count)?;
        let force_number = cursor.read_i32()?;

        let mut samples = Vec::with_capacity(force_count_usize);
        for _ in 0..force_count_usize {
            samples.push(cursor.read_force_sample()?);
        }

        plates.push(ForcePlate {
            plate_id,
            force_number,
            samples,
        });
    }

    cursor.expect_exhausted("force component")?;
    Ok(ForceComponent { plates })
}

fn parse_force_single_component(payload: &[u8]) -> Result<ForceSingleComponent> {
    let mut cursor = Cursor::new(payload);
    let plate_count = cursor.read_i32()?;
    let plate_count = to_usize_non_negative(plate_count)?;

    let mut plates = Vec::with_capacity(plate_count);
    for _ in 0..plate_count {
        plates.push(ForceSinglePlate {
            plate_id: cursor.read_i32()?,
            sample: cursor.read_force_sample()?,
        });
    }

    cursor.expect_exhausted("force single component")?;
    Ok(ForceSingleComponent { plates })
}

fn parse_gaze_vector_component(payload: &[u8]) -> Result<GazeVectorComponent> {
    let mut cursor = Cursor::new(payload);
    let vector_count = cursor.read_i32()?;
    let vector_count = to_usize_non_negative(vector_count)?;

    let mut vectors = Vec::with_capacity(vector_count);
    for _ in 0..vector_count {
        let sample_count = cursor.read_i32()?;
        let sample_count_usize = to_usize_non_negative(sample_count)?;
        let sample_number = cursor.read_i32()?;

        let mut samples = Vec::with_capacity(sample_count_usize);
        for _ in 0..sample_count_usize {
            samples.push(GazeVectorSample {
                direction: cursor.read_point3()?,
                position: cursor.read_point3()?,
            });
        }

        vectors.push(GazeVectorSeries {
            sample_number,
            samples,
        });
    }

    cursor.expect_exhausted("gaze vector component")?;
    Ok(GazeVectorComponent { vectors })
}

fn parse_eye_tracker_component(payload: &[u8]) -> Result<EyeTrackerComponent> {
    let mut cursor = Cursor::new(payload);
    let tracker_count = cursor.read_i32()?;
    let tracker_count = to_usize_non_negative(tracker_count)?;

    let mut trackers = Vec::with_capacity(tracker_count);
    for _ in 0..tracker_count {
        let sample_count = cursor.read_i32()?;
        let sample_count_usize = to_usize_non_negative(sample_count)?;
        let sample_number = cursor.read_i32()?;

        let mut samples = Vec::with_capacity(sample_count_usize);
        for _ in 0..sample_count_usize {
            samples.push(EyeTrackerSample {
                left_pupil_diameter: cursor.read_f32()?,
                right_pupil_diameter: cursor.read_f32()?,
            });
        }

        trackers.push(EyeTrackerSeries {
            sample_number,
            samples,
        });
    }

    cursor.expect_exhausted("eye tracker component")?;
    Ok(EyeTrackerComponent { trackers })
}

fn parse_image_component(payload: &[u8]) -> Result<ImageComponent> {
    let mut cursor = Cursor::new(payload);
    let image_count = cursor.read_i32()?;
    let image_count = to_usize_non_negative(image_count)?;

    let mut images = Vec::with_capacity(image_count);
    for _ in 0..image_count {
        let camera_id = cursor.read_i32()?;
        let format = cursor.read_i32()?;
        let width = cursor.read_i32()?;
        let height = cursor.read_i32()?;
        let crop_left = cursor.read_f32()?;
        let crop_top = cursor.read_f32()?;
        let crop_right = cursor.read_f32()?;
        let crop_bottom = cursor.read_f32()?;
        let image_size = cursor.read_i32()?;
        let image_data = cursor.take(to_usize_non_negative(image_size)?)?.to_vec();

        images.push(Image {
            camera_id,
            format,
            width,
            height,
            crop_left,
            crop_top,
            crop_right,
            crop_bottom,
            data: image_data,
        });
    }

    cursor.expect_exhausted("image component")?;
    Ok(ImageComponent { images })
}

fn parse_timecode_component(payload: &[u8]) -> Result<TimecodeComponent> {
    let mut cursor = Cursor::new(payload);
    let entry_count = cursor.read_i32()?;
    let entry_count = to_usize_non_negative(entry_count)?;

    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        entries.push(TimecodeEntry {
            timecode_type: cursor.read_i32()?,
            high: cursor.read_u32()?,
            low: cursor.read_u32()?,
        });
    }

    cursor.expect_exhausted("timecode component")?;
    Ok(TimecodeComponent { entries })
}

fn parse_skeleton_component(payload: &[u8]) -> Result<SkeletonComponent> {
    let mut cursor = Cursor::new(payload);
    let skeleton_count = cursor.read_i32()?;
    let skeleton_count = to_usize_non_negative(skeleton_count)?;

    let mut skeletons = Vec::with_capacity(skeleton_count);
    for _ in 0..skeleton_count {
        let segment_count = cursor.read_i32()?;
        let segment_count = to_usize_non_negative(segment_count)?;
        let mut segments = Vec::with_capacity(segment_count);

        for _ in 0..segment_count {
            segments.push(SkeletonSegment {
                id: cursor.read_i32()?,
                position: cursor.read_point3()?,
                rotation: Quaternion {
                    x: cursor.read_f32()?,
                    y: cursor.read_f32()?,
                    z: cursor.read_f32()?,
                    w: cursor.read_f32()?,
                },
            });
        }

        skeletons.push(Skeleton { segments });
    }

    cursor.expect_exhausted("skeleton component")?;
    Ok(SkeletonComponent { skeletons })
}

fn read_fixed<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N]> {
    let end = offset
        .checked_add(N)
        .ok_or_else(|| QtmError::invalid_packet("buffer offset overflow"))?;
    let slice = bytes
        .get(offset..end)
        .ok_or_else(|| QtmError::invalid_packet("buffer too short"))?;
    slice
        .try_into()
        .map_err(|_| QtmError::invalid_packet("buffer had unexpected length"))
}

fn to_usize(value: u32) -> Result<usize> {
    usize::try_from(value).map_err(|_| QtmError::invalid_packet("value does not fit usize"))
}

fn to_usize_non_negative(value: i32) -> Result<usize> {
    if value < 0 {
        return Err(QtmError::invalid_packet("negative count encountered"));
    }
    usize::try_from(value).map_err(|_| QtmError::invalid_packet("value does not fit usize"))
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or_else(|| QtmError::invalid_packet("cursor overflow"))?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .ok_or_else(|| QtmError::invalid_packet("packet truncated"))?;
        self.offset = end;
        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(*self
            .take(1)?
            .first()
            .ok_or_else(|| QtmError::invalid_packet("missing byte"))?)
    }

    fn read_i16(&mut self) -> Result<i16> {
        Ok(i16::from_le_bytes(self.take(2)?.try_into().map_err(
            |_| QtmError::invalid_packet("failed to read i16 from payload"),
        )?))
    }

    fn read_i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().map_err(
            |_| QtmError::invalid_packet("failed to read i32 from payload"),
        )?))
    }

    fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().map_err(
            |_| QtmError::invalid_packet("failed to read u32 from payload"),
        )?))
    }

    fn read_u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().map_err(
            |_| QtmError::invalid_packet("failed to read u64 from payload"),
        )?))
    }

    fn read_f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().map_err(
            |_| QtmError::invalid_packet("failed to read f32 from payload"),
        )?))
    }

    fn read_point3(&mut self) -> Result<Point3> {
        Ok(Point3 {
            x: self.read_f32()?,
            y: self.read_f32()?,
            z: self.read_f32()?,
        })
    }

    fn read_f32_array<const N: usize>(&mut self) -> Result<[f32; N]> {
        let mut values = [0.0f32; N];
        for value in &mut values {
            *value = self.read_f32()?;
        }
        Ok(values)
    }

    fn read_force_sample(&mut self) -> Result<ForceSample> {
        Ok(ForceSample {
            force: self.read_f32_array::<3>()?,
            moment: self.read_f32_array::<3>()?,
            application_point: self.read_f32_array::<3>()?,
        })
    }

    fn expect_exhausted(&self, context: &str) -> Result<()> {
        if self.offset == self.bytes.len() {
            return Ok(());
        }
        Err(QtmError::invalid_packet(format!(
            "{context} left {} unread bytes",
            self.bytes.len() - self.offset
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::PacketType;

    #[test]
    fn parses_data_packet_with_3d_and_6d_components() {
        let payload_3d = {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&2u32.to_le_bytes());
            bytes.extend_from_slice(&1i16.to_le_bytes());
            bytes.extend_from_slice(&2i16.to_le_bytes());
            for value in [1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes
        };

        let payload_6d = {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&1i32.to_le_bytes());
            bytes.extend_from_slice(&0i16.to_le_bytes());
            bytes.extend_from_slice(&0i16.to_le_bytes());
            for value in [10.0f32, 20.0, 30.0] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            for value in [1.0f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes
        };

        let mut data_payload = Vec::new();
        data_payload.extend_from_slice(&123u64.to_le_bytes());
        data_payload.extend_from_slice(&7u32.to_le_bytes());
        data_payload.extend_from_slice(&2u32.to_le_bytes());

        data_payload.extend_from_slice(&(8u32 + payload_3d.len() as u32).to_le_bytes());
        data_payload.extend_from_slice(&(ComponentType::ThreeD as u32).to_le_bytes());
        data_payload.extend_from_slice(&payload_3d);

        data_payload.extend_from_slice(&(8u32 + payload_6d.len() as u32).to_le_bytes());
        data_payload.extend_from_slice(&(ComponentType::SixD as u32).to_le_bytes());
        data_payload.extend_from_slice(&payload_6d);

        let mut framed_packet = Vec::new();
        framed_packet.extend_from_slice(&(8u32 + data_payload.len() as u32).to_le_bytes());
        framed_packet.extend_from_slice(&(PacketType::Data as u32).to_le_bytes());
        framed_packet.extend_from_slice(&data_payload);

        let packet = parse_framed_packet(&framed_packet).expect("packet should parse");

        match packet {
            Packet::Data(data) => {
                assert_eq!(data.timestamp, 123);
                assert_eq!(data.frame_number, 7);
                assert_eq!(data.components.len(), 2);
                match &data.components[0].data {
                    ComponentData::ThreeD(component) => {
                        assert_eq!(component.markers.len(), 2);
                        assert_eq!(
                            component.markers[0],
                            Point3 {
                                x: 1.0,
                                y: 2.0,
                                z: 3.0
                            }
                        );
                    }
                    other => panic!("unexpected component: {other:?}"),
                }
                match &data.components[1].data {
                    ComponentData::SixD(component) => {
                        assert_eq!(component.bodies.len(), 1);
                        assert_eq!(
                            component.bodies[0].position,
                            Point3 {
                                x: 10.0,
                                y: 20.0,
                                z: 30.0
                            }
                        );
                    }
                    other => panic!("unexpected component: {other:?}"),
                }
            }
            other => panic!("unexpected packet: {other:?}"),
        }
    }

    #[test]
    fn preserves_unknown_component_payloads() {
        let mut data_payload = Vec::new();
        data_payload.extend_from_slice(&1u64.to_le_bytes());
        data_payload.extend_from_slice(&1u32.to_le_bytes());
        data_payload.extend_from_slice(&1u32.to_le_bytes());
        data_payload.extend_from_slice(&12u32.to_le_bytes());
        data_payload.extend_from_slice(&99u32.to_le_bytes());
        data_payload.extend_from_slice(&[1, 2, 3, 4]);

        let mut framed_packet = Vec::new();
        framed_packet.extend_from_slice(&(8u32 + data_payload.len() as u32).to_le_bytes());
        framed_packet.extend_from_slice(&(PacketType::Data as u32).to_le_bytes());
        framed_packet.extend_from_slice(&data_payload);

        let packet = parse_framed_packet(&framed_packet).expect("packet should parse");
        match packet {
            Packet::Data(data) => match &data.components[0].data {
                ComponentData::Raw(bytes) => assert_eq!(bytes, &[1, 2, 3, 4]),
                other => panic!("unexpected component: {other:?}"),
            },
            other => panic!("unexpected packet: {other:?}"),
        }
    }
}
