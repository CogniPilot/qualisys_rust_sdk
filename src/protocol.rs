use std::fmt;
use std::net::{IpAddr, SocketAddr};

pub const BASE_PORT: u16 = 22222;
pub const LITTLE_ENDIAN_PORT: u16 = BASE_PORT + 1;
pub const BIG_ENDIAN_PORT: u16 = BASE_PORT + 2;
pub const INITIAL_GREETING: &str = "QTM RT Interface connected";
pub const LATEST_PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion {
    major: 1,
    minor: 27,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProtocolVersion {
    pub major: u32,
    pub minor: u32,
}

impl ProtocolVersion {
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        LATEST_PROTOCOL_VERSION
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PacketType {
    Error = 0,
    Command = 1,
    Xml = 2,
    Data = 3,
    NoMoreData = 4,
    C3dFile = 5,
    Event = 6,
    Discover = 7,
    QtmFile = 8,
    None = 9,
}

impl TryFrom<u32> for PacketType {
    type Error = u32;

    fn try_from(value: u32) -> std::result::Result<Self, u32> {
        match value {
            0 => Ok(Self::Error),
            1 => Ok(Self::Command),
            2 => Ok(Self::Xml),
            3 => Ok(Self::Data),
            4 => Ok(Self::NoMoreData),
            5 => Ok(Self::C3dFile),
            6 => Ok(Self::Event),
            7 => Ok(Self::Discover),
            8 => Ok(Self::QtmFile),
            9 => Ok(Self::None),
            other => Err(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Event {
    Connected = 1,
    ConnectionClosed = 2,
    CaptureStarted = 3,
    CaptureStopped = 4,
    CaptureFetchingFinished = 5,
    CalibrationStarted = 6,
    CalibrationStopped = 7,
    RtFromFileStarted = 8,
    RtFromFileStopped = 9,
    WaitingForTrigger = 10,
    CameraSettingsChanged = 11,
    QtmShuttingDown = 12,
    CaptureSaved = 13,
    ReprocessingStarted = 14,
    ReprocessingStopped = 15,
    Trigger = 16,
}

impl TryFrom<u8> for Event {
    type Error = u8;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Connected),
            2 => Ok(Self::ConnectionClosed),
            3 => Ok(Self::CaptureStarted),
            4 => Ok(Self::CaptureStopped),
            5 => Ok(Self::CaptureFetchingFinished),
            6 => Ok(Self::CalibrationStarted),
            7 => Ok(Self::CalibrationStopped),
            8 => Ok(Self::RtFromFileStarted),
            9 => Ok(Self::RtFromFileStopped),
            10 => Ok(Self::WaitingForTrigger),
            11 => Ok(Self::CameraSettingsChanged),
            12 => Ok(Self::QtmShuttingDown),
            13 => Ok(Self::CaptureSaved),
            14 => Ok(Self::ReprocessingStarted),
            15 => Ok(Self::ReprocessingStopped),
            16 => Ok(Self::Trigger),
            other => Err(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ComponentType {
    ThreeD = 1,
    ThreeDNoLabels = 2,
    Analog = 3,
    Force = 4,
    SixD = 5,
    SixDEuler = 6,
    TwoD = 7,
    TwoDLinearized = 8,
    ThreeDResidual = 9,
    ThreeDNoLabelsResidual = 10,
    SixDResidual = 11,
    SixDEulerResidual = 12,
    AnalogSingle = 13,
    Image = 14,
    ForceSingle = 15,
    GazeVector = 16,
    Timecode = 17,
    Skeleton = 18,
    EyeTracker = 19,
}

impl TryFrom<u32> for ComponentType {
    type Error = u32;

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::ThreeD),
            2 => Ok(Self::ThreeDNoLabels),
            3 => Ok(Self::Analog),
            4 => Ok(Self::Force),
            5 => Ok(Self::SixD),
            6 => Ok(Self::SixDEuler),
            7 => Ok(Self::TwoD),
            8 => Ok(Self::TwoDLinearized),
            9 => Ok(Self::ThreeDResidual),
            10 => Ok(Self::ThreeDNoLabelsResidual),
            11 => Ok(Self::SixDResidual),
            12 => Ok(Self::SixDEulerResidual),
            13 => Ok(Self::AnalogSingle),
            14 => Ok(Self::Image),
            15 => Ok(Self::ForceSingle),
            16 => Ok(Self::GazeVector),
            17 => Ok(Self::Timecode),
            18 => Ok(Self::Skeleton),
            19 => Ok(Self::EyeTracker),
            other => Err(other),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterSelection {
    All,
    General,
    ThreeD,
    SixD,
    Analog,
    Force,
    GazeVector,
    EyeTracker,
    Image,
    Skeleton,
    SkeletonGlobal,
    Calibration,
}

impl ParameterSelection {
    pub fn as_command_fragment(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::General => "general",
            Self::ThreeD => "3d",
            Self::SixD => "6d",
            Self::Analog => "analog",
            Self::Force => "force",
            Self::GazeVector => "gazevector",
            Self::EyeTracker => "eyetracker",
            Self::Image => "image",
            Self::Skeleton => "skeleton",
            Self::SkeletonGlobal => "skeleton:global",
            Self::Calibration => "calibration",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentSelection {
    TwoD,
    TwoDLinearized,
    ThreeD,
    ThreeDResidual,
    ThreeDNoLabels,
    ThreeDNoLabelsResidual,
    Analog { channels: Vec<u32> },
    AnalogSingle { channels: Vec<u32> },
    Force,
    ForceSingle,
    SixD,
    SixDResidual,
    SixDEuler,
    SixDEulerResidual,
    Image,
    GazeVector,
    EyeTracker,
    Timecode,
    Skeleton { global: bool },
}

impl ComponentSelection {
    pub fn component_type(&self) -> ComponentType {
        match self {
            Self::TwoD => ComponentType::TwoD,
            Self::TwoDLinearized => ComponentType::TwoDLinearized,
            Self::ThreeD => ComponentType::ThreeD,
            Self::ThreeDResidual => ComponentType::ThreeDResidual,
            Self::ThreeDNoLabels => ComponentType::ThreeDNoLabels,
            Self::ThreeDNoLabelsResidual => ComponentType::ThreeDNoLabelsResidual,
            Self::Analog { .. } => ComponentType::Analog,
            Self::AnalogSingle { .. } => ComponentType::AnalogSingle,
            Self::Force => ComponentType::Force,
            Self::ForceSingle => ComponentType::ForceSingle,
            Self::SixD => ComponentType::SixD,
            Self::SixDResidual => ComponentType::SixDResidual,
            Self::SixDEuler => ComponentType::SixDEuler,
            Self::SixDEulerResidual => ComponentType::SixDEulerResidual,
            Self::Image => ComponentType::Image,
            Self::GazeVector => ComponentType::GazeVector,
            Self::EyeTracker => ComponentType::EyeTracker,
            Self::Timecode => ComponentType::Timecode,
            Self::Skeleton { .. } => ComponentType::Skeleton,
        }
    }

    pub fn as_command_fragment(&self) -> String {
        match self {
            Self::TwoD => "2d".to_owned(),
            Self::TwoDLinearized => "2dlin".to_owned(),
            Self::ThreeD => "3d".to_owned(),
            Self::ThreeDResidual => "3dres".to_owned(),
            Self::ThreeDNoLabels => "3dnolabels".to_owned(),
            Self::ThreeDNoLabelsResidual => "3dnolabelsres".to_owned(),
            Self::Analog { channels } => format_channels("analog", channels),
            Self::AnalogSingle { channels } => format_channels("analogsingle", channels),
            Self::Force => "force".to_owned(),
            Self::ForceSingle => "forcesingle".to_owned(),
            Self::SixD => "6d".to_owned(),
            Self::SixDResidual => "6dres".to_owned(),
            Self::SixDEuler => "6deuler".to_owned(),
            Self::SixDEulerResidual => "6deulerres".to_owned(),
            Self::Image => "image".to_owned(),
            Self::GazeVector => "gazevector".to_owned(),
            Self::EyeTracker => "eyetracker".to_owned(),
            Self::Timecode => "timecode".to_owned(),
            Self::Skeleton { global } => {
                if *global {
                    "skeleton:global".to_owned()
                } else {
                    "skeleton".to_owned()
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamRate {
    AllFrames,
    Frequency(u32),
    FrequencyDivisor(u32),
}

impl StreamRate {
    pub fn as_command_fragment(self) -> String {
        match self {
            Self::AllFrames => "allframes".to_owned(),
            Self::Frequency(value) => format!("frequency:{value}"),
            Self::FrequencyDivisor(value) => format!("frequencydivisor:{value}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamTransport {
    Tcp,
    Udp {
        bind_address: SocketAddr,
        destination: Option<IpAddr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamFramesRequest {
    pub rate: StreamRate,
    pub transport: StreamTransport,
    pub components: Vec<ComponentSelection>,
}

impl StreamFramesRequest {
    pub fn new(
        rate: StreamRate,
        transport: StreamTransport,
        components: impl Into<Vec<ComponentSelection>>,
    ) -> Self {
        Self {
            rate,
            transport,
            components: components.into(),
        }
    }

    pub fn to_command(&self, udp_port_override: Option<u16>) -> String {
        let mut parts = vec!["streamframes".to_owned(), self.rate.as_command_fragment()];

        if let StreamTransport::Udp {
            bind_address,
            destination,
        } = &self.transport
        {
            let port = udp_port_override.unwrap_or(bind_address.port());
            let token = match destination {
                Some(address) => format!("udp:{address}:{port}"),
                None => format!("udp:{port}"),
            };
            parts.push(token);
        }

        parts.extend(
            self.components
                .iter()
                .map(ComponentSelection::as_command_fragment),
        );
        parts.join(" ")
    }
}

pub fn build_version_command(version: ProtocolVersion) -> String {
    format!("version {version}")
}

pub fn build_get_parameters_command(parameters: &[ParameterSelection]) -> String {
    let fragments = if parameters.is_empty() {
        vec![ParameterSelection::All.as_command_fragment()]
    } else {
        parameters
            .iter()
            .copied()
            .map(ParameterSelection::as_command_fragment)
            .collect()
    };
    format!("getparameters {}", fragments.join(" "))
}

pub fn build_get_current_frame_command(components: &[ComponentSelection]) -> String {
    let fragments: Vec<String> = components
        .iter()
        .map(ComponentSelection::as_command_fragment)
        .collect();
    format!("getcurrentframe {}", fragments.join(" "))
}

fn format_channels(prefix: &str, channels: &[u32]) -> String {
    if channels.is_empty() {
        return prefix.to_owned();
    }

    let channel_list = channels
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("{prefix}:{channel_list}")
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use super::*;

    #[test]
    fn builds_stream_frames_udp_command() {
        let request = StreamFramesRequest::new(
            StreamRate::FrequencyDivisor(2),
            StreamTransport::Udp {
                bind_address: SocketAddr::from(([0, 0, 0, 0], 4545)),
                destination: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 10))),
            },
            [
                ComponentSelection::SixD,
                ComponentSelection::Skeleton { global: true },
            ],
        );

        assert_eq!(
            request.to_command(Some(4545)),
            "streamframes frequencydivisor:2 udp:192.168.0.10:4545 6d skeleton:global"
        );
    }

    #[test]
    fn builds_parameter_command() {
        assert_eq!(
            build_get_parameters_command(&[ParameterSelection::General, ParameterSelection::SixD]),
            "getparameters general 6d"
        );
    }

    #[test]
    fn builds_current_frame_command() {
        assert_eq!(
            build_get_current_frame_command(&[
                ComponentSelection::ThreeD,
                ComponentSelection::Analog {
                    channels: vec![1, 4, 7]
                },
            ]),
            "getcurrentframe 3d analog:1,4,7"
        );
    }
}
