//! QTM real-time protocol surface for the Rust SDK.
//!
//! This module is the stable home for RT functionality. It intentionally
//! exposes both idiomatic Rust names and compatibility aliases that make it
//! easier to map examples and concepts from the existing Qualisys SDKs.

pub mod bridge {
    pub use crate::bridge::*;
}

pub mod packet {
    pub use crate::packet::*;
}

pub mod parameters {
    pub use crate::parameters::*;
}

pub mod protocol {
    pub use crate::protocol::*;
}

pub use crate::bridge::{AssembledFrame, BytePublisher, FrameAccumulator, FrameEncoder, FrameSink};
pub use crate::client::{ConnectOptions, QtmClient, StreamPacket};
pub use crate::error::{QtmError, Result};
pub use crate::packet::{
    Component, ComponentData, DataPacket, Packet, PacketHeader, encode_data_packet,
    encode_framed_packet,
};
pub use crate::parameters::{
    MocapParameters, MocapRigidBody, MocapRigidBodyPoint, MocapSixDParameters, MocapSkeleton,
    MocapSkeletonMarker, MocapSkeletonParameters, MocapSkeletonRigidBody, MocapSkeletonSegment,
    MocapThreeDBone, MocapThreeDLabel, MocapThreeDParameters, MocapTransform,
    parse_mocap_parameters_xml,
};
pub use crate::protocol::{
    BASE_PORT, BIG_ENDIAN_PORT, ComponentSelection, ComponentType, Event, INITIAL_GREETING,
    LATEST_PROTOCOL_VERSION, LITTLE_ENDIAN_PORT, PacketType, ParameterSelection, ProtocolVersion,
    StreamFramesRequest, StreamRate, StreamTransport,
};
pub use crate::simulator::{QtmSimulator, SimulatorOptions};

pub type Client = crate::client::QtmClient;
pub type ClientOptions = crate::client::ConnectOptions;

pub type RTProtocol = crate::client::QtmClient;
pub type RTPacket = crate::packet::DataPacket;
pub type RTPacketType = crate::protocol::PacketType;
pub type RTComponentType = crate::protocol::ComponentType;
pub type RTEvent = crate::protocol::Event;

pub type QRTConnection = crate::client::QtmClient;
pub type QRTPacket = crate::packet::DataPacket;
pub type QRTPacketType = crate::protocol::PacketType;
pub type QRTComponentType = crate::protocol::ComponentType;
pub type QRTEvent = crate::protocol::Event;
