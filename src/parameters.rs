//! Typed helpers for the mocap-related subset of QTM `GetParameters` XML.
//!
//! The RT protocol exposes a broad XML settings surface. This module currently
//! models the pieces needed to describe a motion-capture project for streaming:
//! `3D`, `6D`, and `Skeleton`. Additional parameter sections can be added here
//! later without changing the client transport layer.

use roxmltree::{Document, Node};

use crate::error::{QtmError, Result};

const PARAMETERS_ROOT_PREFIX: &str = "QTM_Parameters_Ver_";

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MocapParameters {
    pub envelope_name: String,
    pub three_d: Option<MocapThreeDParameters>,
    pub six_d: Option<MocapSixDParameters>,
    pub skeletons: Option<MocapSkeletonParameters>,
}

impl MocapParameters {
    pub fn new(envelope_name: impl Into<String>) -> Self {
        Self {
            envelope_name: envelope_name.into(),
            three_d: None,
            six_d: None,
            skeletons: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.three_d.is_none() && self.six_d.is_none() && self.skeletons.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MocapThreeDParameters {
    pub axis_upwards: Option<String>,
    pub calibration_time: Option<String>,
    pub labels: Vec<MocapThreeDLabel>,
    pub bones: Vec<MocapThreeDBone>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MocapThreeDLabel {
    pub name: String,
    pub rgb_color: Option<u32>,
    pub trajectory_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MocapThreeDBone {
    pub from_name: String,
    pub to_name: String,
    pub color: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MocapSixDParameters {
    pub bodies: Vec<MocapRigidBody>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MocapRigidBody {
    pub name: String,
    pub enabled: Option<bool>,
    pub color_rgb: Option<[u8; 3]>,
    pub maximum_residual: Option<f32>,
    pub minimum_markers_in_body: Option<u32>,
    pub bone_length_tolerance: Option<f32>,
    pub filter_preset: Option<String>,
    pub points: Vec<MocapRigidBodyPoint>,
    pub data_origin: Option<MocapOrigin>,
    pub data_orientation: Option<MocapOrientation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MocapRigidBodyPoint {
    pub position: [f32; 3],
    pub virtual_point: Option<bool>,
    pub physical_id: Option<i32>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MocapOrigin {
    pub origin_type: Option<i32>,
    pub position: Option<[f32; 3]>,
    pub relative_body: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MocapOrientation {
    pub rotation_matrix: Option<[f32; 9]>,
    pub relative_body: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MocapSkeletonParameters {
    pub skeletons: Vec<MocapSkeleton>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MocapSkeleton {
    pub name: String,
    pub scale: Option<f32>,
    pub segments: Vec<MocapSkeletonSegment>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MocapSkeletonSegment {
    pub name: String,
    pub id: Option<i32>,
    pub solver: Option<String>,
    pub transform: Option<MocapTransform>,
    pub default_transform: Option<MocapTransform>,
    pub endpoint: Option<[f32; 3]>,
    pub markers: Vec<MocapSkeletonMarker>,
    pub rigid_bodies: Vec<MocapSkeletonRigidBody>,
    pub child_segments: Vec<MocapSkeletonSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MocapTransform {
    pub position: Option<[f32; 3]>,
    pub rotation_xyzw: Option<[f32; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MocapSkeletonMarker {
    pub name: String,
    pub position: Option<[f32; 3]>,
    pub weight: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MocapSkeletonRigidBody {
    pub name: String,
    pub transform: Option<MocapTransform>,
    pub weight: Option<f32>,
}

pub fn parse_mocap_parameters_xml(xml: &str) -> Result<MocapParameters> {
    let document = Document::parse(xml)?;
    let root = document.root_element();
    let root_name = root.tag_name().name();

    let mut parameters = MocapParameters::new(root_name);
    match root_name {
        "The_3D" => {
            parameters.three_d = Some(parse_three_d(root)?);
        }
        "The_6D" => {
            parameters.six_d = Some(parse_six_d(root)?);
        }
        "Skeletons" => {
            parameters.skeletons = Some(parse_skeletons(root)?);
        }
        name if name.starts_with(PARAMETERS_ROOT_PREFIX) || name == "QTM_Settings" => {
            for child in root.children().filter(|node| node.is_element()) {
                match child.tag_name().name() {
                    "The_3D" => {
                        assign_once(&mut parameters.three_d, "The_3D", parse_three_d(child)?)?
                    }
                    "The_6D" => assign_once(&mut parameters.six_d, "The_6D", parse_six_d(child)?)?,
                    "Skeletons" => assign_once(
                        &mut parameters.skeletons,
                        "Skeletons",
                        parse_skeletons(child)?,
                    )?,
                    _ => {}
                }
            }
        }
        other => {
            return Err(QtmError::invalid_parameters_xml(format!(
                "unsupported root element `{other}`"
            )));
        }
    }

    Ok(parameters)
}

fn assign_once<T>(slot: &mut Option<T>, name: &str, value: T) -> Result<()> {
    if slot.is_some() {
        return Err(QtmError::invalid_parameters_xml(format!(
            "duplicate `{name}` element"
        )));
    }
    *slot = Some(value);
    Ok(())
}

fn parse_three_d(node: Node<'_, '_>) -> Result<MocapThreeDParameters> {
    let labels = node
        .children()
        .filter(|child| child.is_element() && child.has_tag_name("Label"))
        .map(parse_three_d_label)
        .collect::<Result<Vec<_>>>()?;

    let bones = child_element(node, "Bones")
        .map(|bones| {
            bones
                .children()
                .filter(|child| child.is_element() && child.has_tag_name("Bone"))
                .map(parse_three_d_bone)
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();

    Ok(MocapThreeDParameters {
        axis_upwards: optional_child_text(node, "AxisUpwards"),
        calibration_time: optional_child_text(node, "CalibrationTime"),
        labels,
        bones,
    })
}

fn parse_three_d_label(node: Node<'_, '_>) -> Result<MocapThreeDLabel> {
    Ok(MocapThreeDLabel {
        name: required_child_text(node, "Name", "3D label")?,
        rgb_color: optional_child_text(node, "RGBColor")
            .map(|value| parse_value::<u32>(&value, "3D label RGBColor"))
            .transpose()?,
        trajectory_type: optional_child_text(node, "Trajectory_Type"),
    })
}

fn parse_three_d_bone(node: Node<'_, '_>) -> Result<MocapThreeDBone> {
    Ok(MocapThreeDBone {
        from_name: required_attr(node, &["From", "FromName", "fromName"], "3D bone")?,
        to_name: required_attr(node, &["To", "ToName", "toName"], "3D bone")?,
        color: optional_attr(node, &["Color"], "3D bone color")?
            .map(|value| parse_value::<u32>(&value, "3D bone color"))
            .transpose()?,
    })
}

fn parse_six_d(node: Node<'_, '_>) -> Result<MocapSixDParameters> {
    let bodies = node
        .children()
        .filter(|child| child.is_element() && child.has_tag_name("Body"))
        .map(parse_six_d_body)
        .collect::<Result<Vec<_>>>()?;

    Ok(MocapSixDParameters { bodies })
}

fn parse_six_d_body(node: Node<'_, '_>) -> Result<MocapRigidBody> {
    let color_rgb = child_element(node, "Color")
        .map(|color| {
            Result::<[u8; 3]>::Ok([
                parse_required_attr_value::<u8>(color, &["R"], "6D body color")?,
                parse_required_attr_value::<u8>(color, &["G"], "6D body color")?,
                parse_required_attr_value::<u8>(color, &["B"], "6D body color")?,
            ])
        })
        .transpose()?;

    let points = child_element(node, "Points")
        .map(|points| {
            points
                .children()
                .filter(|child| child.is_element() && child.has_tag_name("Point"))
                .map(parse_six_d_point)
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();

    Ok(MocapRigidBody {
        name: required_child_text(node, "Name", "6D body")?,
        enabled: optional_child_bool(node, "Enabled")?,
        color_rgb,
        maximum_residual: optional_child_f32(node, "MaximumResidual")?,
        minimum_markers_in_body: optional_child_text(node, "MinimumMarkersInBody")
            .map(|value| parse_value::<u32>(&value, "6D MinimumMarkersInBody"))
            .transpose()?,
        bone_length_tolerance: optional_child_f32(node, "BoneLengthTolerance")?,
        filter_preset: child_element(node, "Filter")
            .and_then(|filter| filter.attribute("Preset"))
            .map(ToOwned::to_owned),
        points,
        data_origin: child_element(node, "Data_origin")
            .map(parse_origin)
            .transpose()?,
        data_orientation: child_element(node, "Data_orientation")
            .map(parse_orientation)
            .transpose()?,
    })
}

fn parse_six_d_point(node: Node<'_, '_>) -> Result<MocapRigidBodyPoint> {
    Ok(MocapRigidBodyPoint {
        position: parse_xyz_attrs(node, "6D point")?,
        virtual_point: optional_attr(node, &["Virtual"], "6D point Virtual")?
            .map(|value| parse_bool_value(&value, "6D point Virtual"))
            .transpose()?,
        physical_id: optional_attr(node, &["PhysicalId"], "6D point PhysicalId")?
            .map(|value| parse_value::<i32>(&value, "6D point PhysicalId"))
            .transpose()?,
        name: optional_attr(node, &["Name"], "6D point name")?,
    })
}

fn parse_origin(node: Node<'_, '_>) -> Result<MocapOrigin> {
    Ok(MocapOrigin {
        origin_type: node
            .text()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(|value| parse_value::<i32>(value, "Data_origin value"))
            .transpose()?,
        position: optional_xyz_attrs(node, "Data_origin")?,
        relative_body: optional_attr(node, &["Relative_body"], "Data_origin Relative_body")?
            .map(|value| parse_value::<i32>(&value, "Data_origin Relative_body"))
            .transpose()?,
    })
}

fn parse_orientation(node: Node<'_, '_>) -> Result<MocapOrientation> {
    let rotation_matrix = parse_optional_rotation_matrix(node, "Data_orientation")?;
    let relative_body = optional_attr(node, &["Relative_body"], "Data_orientation Relative_body")?
        .map(|value| parse_value::<i32>(&value, "Data_orientation Relative_body"))
        .transpose()?;

    Ok(MocapOrientation {
        rotation_matrix,
        relative_body,
    })
}

fn parse_skeletons(node: Node<'_, '_>) -> Result<MocapSkeletonParameters> {
    let skeletons = node
        .children()
        .filter(|child| child.is_element() && child.has_tag_name("Skeleton"))
        .map(parse_skeleton)
        .collect::<Result<Vec<_>>>()?;

    Ok(MocapSkeletonParameters { skeletons })
}

fn parse_skeleton(node: Node<'_, '_>) -> Result<MocapSkeleton> {
    let segments = child_element(node, "Segments")
        .map(|segments| {
            segments
                .children()
                .filter(|child| child.is_element() && child.has_tag_name("Segment"))
                .map(parse_skeleton_segment)
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();

    Ok(MocapSkeleton {
        name: required_attr(node, &["Name"], "Skeleton")?,
        scale: optional_child_f32(node, "Scale")?,
        segments,
    })
}

fn parse_skeleton_segment(node: Node<'_, '_>) -> Result<MocapSkeletonSegment> {
    let markers = child_element(node, "Markers")
        .map(|markers| {
            markers
                .children()
                .filter(|child| child.is_element() && child.has_tag_name("Marker"))
                .map(parse_skeleton_marker)
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();

    let rigid_bodies = child_element(node, "RigidBodies")
        .map(|rigid_bodies| {
            rigid_bodies
                .children()
                .filter(|child| child.is_element() && child.has_tag_name("RigidBody"))
                .map(parse_skeleton_rigid_body)
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?
        .unwrap_or_default();

    let child_segments = node
        .children()
        .filter(|child| child.is_element() && child.has_tag_name("Segment"))
        .map(parse_skeleton_segment)
        .collect::<Result<Vec<_>>>()?;

    Ok(MocapSkeletonSegment {
        name: required_attr(node, &["Name"], "Skeleton segment")?,
        id: optional_attr(node, &["ID"], "Skeleton segment ID")?
            .map(|value| parse_value::<i32>(&value, "Skeleton segment ID"))
            .transpose()?,
        solver: optional_child_text(node, "Solver"),
        transform: child_element(node, "Transform")
            .map(parse_transform)
            .transpose()?,
        default_transform: child_element(node, "DefaultTransform")
            .map(parse_transform)
            .transpose()?,
        endpoint: child_element(node, "Endpoint")
            .map(|endpoint| parse_xyz_attrs(endpoint, "Skeleton endpoint"))
            .transpose()?,
        markers,
        rigid_bodies,
        child_segments,
    })
}

fn parse_transform(node: Node<'_, '_>) -> Result<MocapTransform> {
    Ok(MocapTransform {
        position: child_element(node, "Position")
            .map(|position| parse_xyz_attrs(position, "Transform position"))
            .transpose()?,
        rotation_xyzw: child_element(node, "Rotation")
            .map(|rotation| parse_xyzw_attrs(rotation, "Transform rotation"))
            .transpose()?,
    })
}

fn parse_skeleton_marker(node: Node<'_, '_>) -> Result<MocapSkeletonMarker> {
    Ok(MocapSkeletonMarker {
        name: required_attr(node, &["Name"], "Skeleton marker")?,
        position: child_element(node, "Position")
            .map(|position| parse_xyz_attrs(position, "Skeleton marker position"))
            .transpose()?,
        weight: optional_child_f32(node, "Weight")?,
    })
}

fn parse_skeleton_rigid_body(node: Node<'_, '_>) -> Result<MocapSkeletonRigidBody> {
    Ok(MocapSkeletonRigidBody {
        name: required_attr(node, &["Name"], "Skeleton rigid body")?,
        transform: child_element(node, "Transform")
            .map(parse_transform)
            .transpose()?,
        weight: optional_child_f32(node, "Weight")?,
    })
}

fn child_element<'a, 'input>(parent: Node<'a, 'input>, name: &str) -> Option<Node<'a, 'input>> {
    parent
        .children()
        .find(|node| node.is_element() && node.has_tag_name(name))
}

fn optional_child_text(parent: Node<'_, '_>, name: &str) -> Option<String> {
    child_element(parent, name)
        .and_then(|node| node.text())
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(ToOwned::to_owned)
}

fn required_child_text(parent: Node<'_, '_>, name: &str, context: &str) -> Result<String> {
    optional_child_text(parent, name).ok_or_else(|| {
        QtmError::invalid_parameters_xml(format!("missing `{name}` element in {context}"))
    })
}

fn optional_child_bool(parent: Node<'_, '_>, name: &str) -> Result<Option<bool>> {
    optional_child_text(parent, name)
        .map(|value| parse_bool_value(&value, name))
        .transpose()
}

fn optional_child_f32(parent: Node<'_, '_>, name: &str) -> Result<Option<f32>> {
    optional_child_text(parent, name)
        .map(|value| parse_value::<f32>(&value, name))
        .transpose()
}

fn required_attr(node: Node<'_, '_>, names: &[&str], context: &str) -> Result<String> {
    optional_attr(node, names, context)?.ok_or_else(|| {
        QtmError::invalid_parameters_xml(format!(
            "missing attribute `{}` in {context}",
            names.join(" or ")
        ))
    })
}

fn optional_attr(node: Node<'_, '_>, names: &[&str], _context: &str) -> Result<Option<String>> {
    Ok(names
        .iter()
        .find_map(|name| node.attribute(*name))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned))
}

fn parse_required_attr_value<T>(node: Node<'_, '_>, names: &[&str], context: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value = required_attr(node, names, context)?;
    parse_value(&value, context)
}

fn parse_xyz_attrs(node: Node<'_, '_>, context: &str) -> Result<[f32; 3]> {
    Ok([
        parse_required_attr_value::<f32>(node, &["X"], context)?,
        parse_required_attr_value::<f32>(node, &["Y"], context)?,
        parse_required_attr_value::<f32>(node, &["Z"], context)?,
    ])
}

fn optional_xyz_attrs(node: Node<'_, '_>, context: &str) -> Result<Option<[f32; 3]>> {
    let x = optional_attr(node, &["X"], context)?;
    let y = optional_attr(node, &["Y"], context)?;
    let z = optional_attr(node, &["Z"], context)?;

    match (x, y, z) {
        (None, None, None) => Ok(None),
        (Some(x), Some(y), Some(z)) => Ok(Some([
            parse_value::<f32>(&x, context)?,
            parse_value::<f32>(&y, context)?,
            parse_value::<f32>(&z, context)?,
        ])),
        _ => Err(QtmError::invalid_parameters_xml(format!(
            "incomplete XYZ attributes in {context}"
        ))),
    }
}

fn parse_xyzw_attrs(node: Node<'_, '_>, context: &str) -> Result<[f32; 4]> {
    Ok([
        parse_required_attr_value::<f32>(node, &["X"], context)?,
        parse_required_attr_value::<f32>(node, &["Y"], context)?,
        parse_required_attr_value::<f32>(node, &["Z"], context)?,
        parse_required_attr_value::<f32>(node, &["W"], context)?,
    ])
}

fn parse_optional_rotation_matrix(node: Node<'_, '_>, context: &str) -> Result<Option<[f32; 9]>> {
    let names = [
        "R11", "R12", "R13", "R21", "R22", "R23", "R31", "R32", "R33",
    ];
    let values = names
        .iter()
        .map(|name| optional_attr(node, &[*name], context))
        .collect::<Result<Vec<_>>>()?;

    if values.iter().all(Option::is_none) {
        return Ok(None);
    }

    if values.iter().any(Option::is_none) {
        return Err(QtmError::invalid_parameters_xml(format!(
            "incomplete rotation matrix attributes in {context}"
        )));
    }

    let mut matrix = [0.0; 9];
    for (index, value) in values.into_iter().enumerate() {
        matrix[index] =
            parse_value::<f32>(value.as_deref().expect("value already checked"), context)?;
    }
    Ok(Some(matrix))
}

fn parse_bool_value(value: &str, context: &str) -> Result<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        other => Err(QtmError::invalid_parameters_xml(format!(
            "invalid boolean `{other}` in {context}"
        ))),
    }
}

fn parse_value<T>(value: &str, context: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value.trim().parse::<T>().map_err(|error| {
        QtmError::invalid_parameters_xml(format!("invalid `{context}` value `{value}`: {error}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PARAMETERS_XML: &str = r#"
<QTM_Parameters_Ver_1.27>
  <General>
    <Frequency>300</Frequency>
  </General>
  <The_3D>
    <AxisUpwards>+Z</AxisUpwards>
    <CalibrationTime>2026.04.10 12:30:00</CalibrationTime>
    <Labels>2</Labels>
    <Label>
      <Name>Head</Name>
      <RGBColor>255</RGBColor>
      <Trajectory_Type>Measured</Trajectory_Type>
    </Label>
    <Label>
      <Name>Hand</Name>
      <RGBColor>65280</RGBColor>
      <Trajectory_Type>Virtual</Trajectory_Type>
    </Label>
    <Bones>
      <Bone From="Head" To="Hand" Color="16711680" />
    </Bones>
  </The_3D>
  <The_6D>
    <Body>
      <Name>Torso</Name>
      <Enabled>true</Enabled>
      <Color R="1" G="2" B="3" />
      <MaximumResidual>0.5</MaximumResidual>
      <MinimumMarkersInBody>3</MinimumMarkersInBody>
      <BoneLengthTolerance>1.5</BoneLengthTolerance>
      <Filter Preset="Static pose" />
      <Points>
        <Point X="0.0" Y="1.0" Z="2.0" Virtual="0" PhysicalId="10" Name="P1" />
        <Point X="3.0" Y="4.0" Z="5.0" Virtual="1" PhysicalId="11" Name="P2" />
      </Points>
      <Data_origin X="10.0" Y="20.0" Z="30.0" Relative_body="0">1</Data_origin>
      <Data_orientation R11="1" R12="0" R13="0" R21="0" R22="1" R23="0" R31="0" R32="0" R33="1" Relative_body="0" />
    </Body>
  </The_6D>
  <Skeletons>
    <Skeleton Name="Human">
      <Scale>1.0</Scale>
      <Segments>
        <Segment Name="Pelvis" ID="10">
          <Solver>Global Optimization</Solver>
          <Transform>
            <Position X="1.0" Y="2.0" Z="3.0" />
            <Rotation X="0.0" Y="0.0" Z="0.0" W="1.0" />
          </Transform>
          <Markers>
            <Marker Name="PelvisMarker">
              <Position X="4.0" Y="5.0" Z="6.0" />
              <Weight>0.8</Weight>
            </Marker>
          </Markers>
          <RigidBodies>
            <RigidBody Name="Torso">
              <Transform>
                <Position X="7.0" Y="8.0" Z="9.0" />
                <Rotation X="0.0" Y="0.0" Z="0.0" W="1.0" />
              </Transform>
              <Weight>1.0</Weight>
            </RigidBody>
          </RigidBodies>
          <Segment Name="Thigh" ID="11">
            <Endpoint X="10.0" Y="11.0" Z="12.0" />
          </Segment>
        </Segment>
      </Segments>
    </Skeleton>
  </Skeletons>
</QTM_Parameters_Ver_1.27>
"#;

    #[test]
    fn parses_mocap_parameter_envelope() {
        let parameters =
            parse_mocap_parameters_xml(SAMPLE_PARAMETERS_XML).expect("valid mocap XML");

        assert_eq!(parameters.envelope_name, "QTM_Parameters_Ver_1.27");
        assert!(!parameters.is_empty());

        let three_d = parameters.three_d.expect("3D section");
        assert_eq!(three_d.axis_upwards.as_deref(), Some("+Z"));
        assert_eq!(three_d.labels.len(), 2);
        assert_eq!(three_d.labels[0].name, "Head");
        assert_eq!(three_d.labels[1].rgb_color, Some(65_280));
        assert_eq!(three_d.bones[0].from_name, "Head");
        assert_eq!(three_d.bones[0].to_name, "Hand");

        let six_d = parameters.six_d.expect("6D section");
        assert_eq!(six_d.bodies.len(), 1);
        assert_eq!(six_d.bodies[0].name, "Torso");
        assert_eq!(six_d.bodies[0].enabled, Some(true));
        assert_eq!(six_d.bodies[0].color_rgb, Some([1, 2, 3]));
        assert_eq!(six_d.bodies[0].points.len(), 2);
        assert_eq!(
            six_d.bodies[0]
                .data_origin
                .as_ref()
                .and_then(|o| o.origin_type),
            Some(1)
        );
        assert_eq!(
            six_d.bodies[0]
                .data_orientation
                .as_ref()
                .and_then(|o| o.rotation_matrix),
            Some([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
        );

        let skeletons = parameters.skeletons.expect("Skeleton section");
        assert_eq!(skeletons.skeletons.len(), 1);
        assert_eq!(skeletons.skeletons[0].name, "Human");
        assert_eq!(skeletons.skeletons[0].segments.len(), 1);
        assert_eq!(skeletons.skeletons[0].segments[0].name, "Pelvis");
        assert_eq!(
            skeletons.skeletons[0].segments[0].child_segments[0].id,
            Some(11)
        );
    }

    #[test]
    fn parses_direct_section_root() {
        let parameters = parse_mocap_parameters_xml(
            r#"
<The_6D>
  <Body>
    <Name>RigidBody</Name>
  </Body>
</The_6D>
"#,
        )
        .expect("valid direct 6D XML");

        assert_eq!(parameters.envelope_name, "The_6D");
        assert!(parameters.three_d.is_none());
        assert_eq!(parameters.six_d.expect("6D").bodies[0].name, "RigidBody");
    }

    #[test]
    fn rejects_unknown_root() {
        let error = parse_mocap_parameters_xml("<General />").expect_err("unknown root");
        assert!(matches!(error, QtmError::InvalidParametersXml(_)));
    }
}
