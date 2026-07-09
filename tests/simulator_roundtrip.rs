use std::net::{SocketAddr, TcpListener};
use std::thread;
use std::time::{Duration, Instant};

use qualisys_rust_sdk::rt::{
    AssembledFrame, Client, ClientOptions, Component, ComponentData, ComponentSelection,
    ComponentType, DataPacket, FrameAccumulator, ParameterSelection, QtmSimulator,
    SimulatorOptions, StreamFramesRequest, StreamPacket, StreamRate, StreamTransport,
};

#[test]
fn simulator_supports_client_queries_and_streaming() {
    let mut client = start_simulator_client(3);

    assert_eq!(client.qtm_version().unwrap(), "QTM RT simulator 0.1");
    assert_eq!(client.byte_order().unwrap(), "little endian");

    let raw_parameters = client
        .get_parameters(&[ParameterSelection::General, ParameterSelection::SixD])
        .unwrap();
    assert!(raw_parameters.contains("<Frequency>240</Frequency>"));

    let parameters = client.get_mocap_parameters().unwrap();
    let six_d = parameters
        .six_d
        .as_ref()
        .expect("simulator should expose 6D parameters");
    let body_names = six_d
        .bodies
        .iter()
        .map(|body| body.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(body_names, vec!["sim_body_1", "sim_body_2", "sim_body_3"]);
    assert!(six_d.bodies.iter().all(|body| body.enabled == Some(true)));
    assert!(
        six_d
            .bodies
            .iter()
            .all(|body| body.color_rgb == Some([0, 255, 0]))
    );

    let current_frame = client
        .get_current_frame(&[ComponentSelection::SixDResidual])
        .unwrap();
    assert_eq!(current_frame.frame_number, 1);
    assert_six_d_residual_packet(&current_frame, 3);

    let current_six_d_frame = client
        .get_current_frame(&[ComponentSelection::SixD])
        .unwrap();
    assert_eq!(current_six_d_frame.frame_number, 1);
    assert_six_d_packet(&current_six_d_frame, 3);

    assert_streams_frames(
        &mut client,
        StreamTransport::Udp {
            bind_address: SocketAddr::from(([127, 0, 0, 1], 0)),
            destination: None,
        },
        3,
    );
    assert_streams_frames(&mut client, StreamTransport::Tcp, 3);
}

fn start_simulator_client(rigid_body_count: usize) -> Client {
    let bind_address = unused_loopback_address();
    let options = SimulatorOptions {
        bind_address,
        frame_rate_hz: 240,
        rigid_body_count,
    };

    thread::spawn(move || {
        QtmSimulator::new(options)
            .run()
            .expect("simulator should keep serving clients");
    });

    connect_with_retry(bind_address)
}

fn unused_loopback_address() -> SocketAddr {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
    listener.local_addr().unwrap()
}

fn connect_with_retry(address: SocketAddr) -> Client {
    let deadline = Instant::now() + Duration::from_secs(5);
    let options = ClientOptions {
        port: address.port(),
        read_timeout: Duration::from_secs(2),
        ..ClientOptions::default()
    };

    loop {
        match Client::connect("127.0.0.1", options) {
            Ok(client) => return client,
            Err(error) if Instant::now() < deadline => {
                let _ = error;
                thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("simulator did not accept connections on {address}: {error}"),
        }
    }
}

fn assert_streams_frames(client: &mut Client, transport: StreamTransport, expected_bodies: usize) {
    let components = vec![ComponentSelection::SixDResidual];
    let request = StreamFramesRequest::new(StreamRate::AllFrames, transport, components.clone());
    client.start_stream_frames(&request).unwrap();

    let mut accumulator = FrameAccumulator::for_components(components);
    let mut frames = Vec::new();
    while frames.len() < 3 {
        match client.recv_stream_packet().unwrap() {
            StreamPacket::Data(packet) => frames.extend(accumulator.push(packet)),
            StreamPacket::NoMoreData => panic!("simulator ended stream before test frames arrived"),
        }
    }

    client.stop_stream_frames().unwrap();

    for frame in &frames {
        assert!(frame.complete);
        assert_six_d_residual_frame(frame, expected_bodies);
    }

    assert!(
        frames
            .windows(2)
            .all(|window| window[1].frame_number > window[0].frame_number)
    );
}

fn assert_six_d_residual_packet(packet: &DataPacket, expected_bodies: usize) {
    let component = packet
        .component(ComponentType::SixDResidual)
        .expect("simulator should emit 6D residual data");
    assert_six_d_residual_component(component, expected_bodies);
}

fn assert_six_d_packet(packet: &DataPacket, expected_bodies: usize) {
    let component = packet
        .component(ComponentType::SixD)
        .expect("simulator should emit 6D data when requested");
    let ComponentData::SixD(six_d) = &component.data else {
        panic!("expected 6D component, got {:?}", component.data);
    };

    assert_eq!(six_d.drop_rate, 0);
    assert_eq!(six_d.out_of_sync_rate, 0);
    assert_eq!(six_d.bodies.len(), expected_bodies);
}

fn assert_six_d_residual_frame(frame: &AssembledFrame, expected_bodies: usize) {
    let component = frame
        .component(ComponentType::SixDResidual)
        .expect("simulator stream should emit 6D residual data");
    assert_six_d_residual_component(component, expected_bodies);
}

fn assert_six_d_residual_component(component: &Component, expected_bodies: usize) {
    let ComponentData::SixDResidual(six_d) = &component.data else {
        panic!("expected 6D residual component, got {:?}", component.data);
    };

    assert_eq!(six_d.drop_rate, 0);
    assert_eq!(six_d.out_of_sync_rate, 0);
    assert_eq!(six_d.bodies.len(), expected_bodies);

    for (index, body) in six_d.bodies.iter().enumerate() {
        assert!(body.position.x.is_finite());
        assert!(body.position.y.is_finite());
        assert!(body.position.z.is_finite());
        assert!(body.rotation_matrix.iter().all(|value| value.is_finite()));
        assert_position_uses_millimeters(body.position.x, body.position.y, body.position.z);
        assert_rotation_matrix_is_orthonormal(body.rotation_matrix);

        let expected_residual = 0.001 * (index + 1) as f32;
        assert!((body.residual - expected_residual).abs() < f32::EPSILON);
    }
}

fn assert_position_uses_millimeters(x: f32, y: f32, z: f32) {
    let xy_radius = x.hypot(y);
    assert!((xy_radius - 750.0).abs() < 1.0);
    assert!((900.0..=1100.0).contains(&z));
}

fn assert_rotation_matrix_is_orthonormal(rotation: [f32; 9]) {
    let row_0 = [rotation[0], rotation[1], rotation[2]];
    let row_1 = [rotation[3], rotation[4], rotation[5]];
    let row_2 = [rotation[6], rotation[7], rotation[8]];

    assert!((norm(row_0) - 1.0).abs() < 0.001);
    assert!((norm(row_1) - 1.0).abs() < 0.001);
    assert!((norm(row_2) - 1.0).abs() < 0.001);
    assert!(dot(row_0, row_1).abs() < 0.001);
    assert!(dot(row_0, row_2).abs() < 0.001);
    assert!(dot(row_1, row_2).abs() < 0.001);
}

fn norm(values: [f32; 3]) -> f32 {
    dot(values, values).sqrt()
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter().zip(b).map(|(left, right)| left * right).sum()
}
