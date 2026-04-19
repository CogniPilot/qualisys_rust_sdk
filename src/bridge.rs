use std::collections::{BTreeMap, BTreeSet};

use crate::packet::{Component, DataPacket};
use crate::protocol::{ComponentSelection, ComponentType};

#[derive(Debug, Clone, PartialEq)]
pub struct AssembledFrame {
    pub timestamp: u64,
    pub frame_number: u32,
    pub complete: bool,
    pub components: BTreeMap<u32, Component>,
}

impl AssembledFrame {
    pub fn component(&self, component_type: ComponentType) -> Option<&Component> {
        self.components.get(&(component_type as u32))
    }

    fn from_packet(packet: DataPacket) -> Self {
        let components = packet
            .components
            .into_iter()
            .map(|component| (component.id, component))
            .collect();

        Self {
            timestamp: packet.timestamp,
            frame_number: packet.frame_number,
            complete: false,
            components,
        }
    }

    fn merge_packet(&mut self, packet: DataPacket) {
        debug_assert_eq!(self.timestamp, packet.timestamp);
        debug_assert_eq!(self.frame_number, packet.frame_number);

        for component in packet.components {
            self.components.insert(component.id, component);
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameAccumulator {
    expected_components: BTreeSet<u32>,
    current: Option<AssembledFrame>,
}

impl FrameAccumulator {
    pub fn new(expected_components: impl IntoIterator<Item = u32>) -> Self {
        Self {
            expected_components: expected_components.into_iter().collect(),
            current: None,
        }
    }

    pub fn for_components(
        expected_components: impl IntoIterator<Item = ComponentSelection>,
    ) -> Self {
        Self::new(
            expected_components
                .into_iter()
                .map(|component| component.component_type() as u32),
        )
    }

    pub fn push(&mut self, packet: DataPacket) -> Vec<AssembledFrame> {
        let mut emitted = Vec::new();

        match self.current.take() {
            None => {
                self.accept_packet(packet, &mut emitted);
            }
            Some(mut current)
                if current.timestamp == packet.timestamp
                    && current.frame_number == packet.frame_number =>
            {
                current.merge_packet(packet);
                if self.is_complete(&current) {
                    emitted.push(self.finalize(current));
                } else {
                    self.current = Some(current);
                }
            }
            Some(current) => {
                emitted.push(self.finalize(current));
                self.accept_packet(packet, &mut emitted);
            }
        }

        emitted
    }

    pub fn flush(&mut self) -> Option<AssembledFrame> {
        self.current.take().map(|frame| self.finalize(frame))
    }

    fn accept_packet(&mut self, packet: DataPacket, emitted: &mut Vec<AssembledFrame>) {
        let frame = AssembledFrame::from_packet(packet);
        if self.is_complete(&frame) {
            emitted.push(self.finalize(frame));
        } else {
            self.current = Some(frame);
        }
    }

    fn is_complete(&self, frame: &AssembledFrame) -> bool {
        self.expected_components.is_empty()
            || self
                .expected_components
                .iter()
                .all(|component| frame.components.contains_key(component))
    }

    fn finalize(&self, mut frame: AssembledFrame) -> AssembledFrame {
        frame.complete = self.is_complete(&frame);
        frame
    }
}

pub trait FrameEncoder {
    type Error;

    fn encode(&mut self, frame: &AssembledFrame) -> std::result::Result<Vec<u8>, Self::Error>;
}

pub trait BytePublisher {
    type Error;

    fn publish(&mut self, topic: &str, payload: &[u8]) -> std::result::Result<(), Self::Error>;
}

pub trait FrameSink {
    type Error;

    fn publish_frame(&mut self, frame: &AssembledFrame) -> std::result::Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use crate::packet::{Component, ComponentData, DataPacket, Point3, ThreeDComponent};
    use crate::protocol::{ComponentSelection, ComponentType};

    use super::*;

    #[test]
    fn accumulates_udp_split_frames() {
        let mut accumulator = FrameAccumulator::for_components([
            ComponentSelection::ThreeD,
            ComponentSelection::SixD,
        ]);

        let partial = DataPacket {
            timestamp: 10,
            frame_number: 99,
            components: vec![Component {
                id: ComponentType::ThreeD as u32,
                data: ComponentData::ThreeD(ThreeDComponent {
                    drop_rate: 0,
                    out_of_sync_rate: 0,
                    markers: vec![Point3 {
                        x: 1.0,
                        y: 2.0,
                        z: 3.0,
                    }],
                }),
            }],
        };

        assert!(accumulator.push(partial).is_empty());

        let complete = DataPacket {
            timestamp: 10,
            frame_number: 99,
            components: vec![Component {
                id: ComponentType::SixD as u32,
                data: ComponentData::Raw(vec![1, 2, 3]),
            }],
        };

        let emitted = accumulator.push(complete);
        assert_eq!(emitted.len(), 1);
        assert!(emitted[0].complete);
        assert_eq!(emitted[0].components.len(), 2);
    }

    #[test]
    fn flushes_incomplete_frame_when_new_frame_arrives() {
        let mut accumulator = FrameAccumulator::for_components([ComponentSelection::ThreeD]);
        let first = DataPacket {
            timestamp: 1,
            frame_number: 1,
            components: vec![],
        };
        let second = DataPacket {
            timestamp: 2,
            frame_number: 2,
            components: vec![],
        };

        assert!(accumulator.push(first).is_empty());
        let emitted = accumulator.push(second);
        assert_eq!(emitted.len(), 1);
        assert!(!emitted[0].complete);
        assert_eq!(emitted[0].frame_number, 1);
    }
}
