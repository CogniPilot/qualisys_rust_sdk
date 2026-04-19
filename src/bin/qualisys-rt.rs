use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};
use qualisys_rust_sdk::rt::{
    AssembledFrame, Client, ClientOptions, Component, ComponentData, ComponentSelection,
    ComponentType, DataPacket, FrameAccumulator, ParameterSelection, ProtocolVersion,
    StreamFramesRequest, StreamPacket, StreamRate, StreamTransport, LITTLE_ENDIAN_PORT,
};

type CliResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
#[command(
    name = "qualisys-rt",
    version,
    about = "Diagnostic CLI for the Qualisys QTM real-time protocol"
)]
struct Cli {
    #[arg(long, global = true, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, global = true, default_value_t = LITTLE_ENDIAN_PORT)]
    port: u16,

    #[arg(long, global = true, default_value = "1.27", value_parser = parse_protocol_version)]
    protocol_version: ProtocolVersion,

    #[arg(long, global = true, default_value_t = 5_000)]
    timeout_ms: u64,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Connect to QTM and print basic RT interface information.
    Info,
    /// Fetch and print QTM GetParameters XML.
    Params {
        /// Comma-separated parameter list, for example: general,3d,6d,skeleton.
        #[arg(long, default_value = "general,3d,6d,skeleton")]
        parameters: String,
    },
    /// Fetch one current frame and print decoded component summaries.
    Frame {
        /// Comma-separated component list, for example: 3d,6d,skeleton.
        #[arg(long, default_value = "6d")]
        components: String,
    },
    /// Start a real-time stream and print decoded frame summaries.
    Stream {
        /// Comma-separated component list, for example: 3d,6d,skeleton.
        #[arg(long, default_value = "6d")]
        components: String,
        /// Stream rate: allframes, frequency:<hz>, or divisor:<n>.
        #[arg(long, default_value = "allframes", value_parser = parse_stream_rate)]
        rate: StreamRate,
        /// Stream over UDP or TCP.
        #[arg(long, value_enum, default_value = "udp")]
        transport: StreamTransportArg,
        /// UDP bind address used when --transport udp.
        #[arg(long, default_value = "0.0.0.0:0")]
        bind: SocketAddr,
        /// Optional UDP destination address sent to QTM.
        #[arg(long)]
        destination: Option<IpAddr>,
        /// Number of assembled frames to print. Use 0 to stream until QTM stops or the process is interrupted.
        #[arg(long, default_value_t = 10)]
        count: usize,
        /// Print per-component details for each assembled frame.
        #[arg(long)]
        verbose: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum StreamTransportArg {
    Udp,
    Tcp,
}

fn main() -> CliResult<()> {
    let cli = Cli::parse();
    let mut client = connect(&cli)?;

    match &cli.command {
        Command::Info => print_info(&mut client, &cli),
        Command::Params { parameters } => print_parameters(&mut client, parameters),
        Command::Frame { components } => print_current_frame(&mut client, components),
        Command::Stream {
            components,
            rate,
            transport,
            bind,
            destination,
            count,
            verbose,
        } => stream_frames(
            &mut client,
            components,
            *rate,
            *transport,
            *bind,
            *destination,
            *count,
            *verbose,
        ),
    }
}

fn connect(cli: &Cli) -> CliResult<Client> {
    let options = ClientOptions {
        port: cli.port,
        version: cli.protocol_version,
        read_timeout: Duration::from_millis(cli.timeout_ms),
        ..ClientOptions::default()
    };

    Ok(Client::connect(&cli.host, options)?)
}

fn print_info(client: &mut Client, cli: &Cli) -> CliResult<()> {
    println!("host: {}", cli.host);
    println!("port: {}", cli.port);
    println!("protocol_version: {}", client.version());
    println!("qtm_version: {}", client.qtm_version()?);
    println!("byte_order: {}", client.byte_order()?);
    Ok(())
}

fn print_parameters(client: &mut Client, parameters: &str) -> CliResult<()> {
    let parameters = parse_parameter_list(parameters)?;
    let xml = client.get_parameters(&parameters)?;
    println!("{xml}");
    Ok(())
}

fn print_current_frame(client: &mut Client, components: &str) -> CliResult<()> {
    let components = parse_component_list(components)?;
    let packet = client.get_current_frame(&components)?;
    print_packet(&packet);
    Ok(())
}

fn stream_frames(
    client: &mut Client,
    components: &str,
    rate: StreamRate,
    transport: StreamTransportArg,
    bind: SocketAddr,
    destination: Option<IpAddr>,
    count: usize,
    verbose: bool,
) -> CliResult<()> {
    let components = parse_component_list(components)?;
    let request = StreamFramesRequest::new(
        rate,
        match transport {
            StreamTransportArg::Udp => StreamTransport::Udp {
                bind_address: bind,
                destination,
            },
            StreamTransportArg::Tcp => StreamTransport::Tcp,
        },
        components.clone(),
    );

    client.start_stream_frames(&request)?;
    if let Some(address) = client.udp_local_addr()? {
        println!("udp_local_addr: {address}");
    }

    let mut accumulator = FrameAccumulator::for_components(components);
    let mut printed = 0usize;

    loop {
        match client.recv_stream_packet()? {
            StreamPacket::Data(packet) => {
                for frame in accumulator.push(packet) {
                    print_frame(&frame, verbose);
                    printed += 1;
                    if count != 0 && printed >= count {
                        client.stop_stream_frames()?;
                        return Ok(());
                    }
                }
            }
            StreamPacket::NoMoreData => {
                println!("QTM reported end of stream");
                return Ok(());
            }
        }
    }
}

fn print_packet(packet: &DataPacket) {
    println!(
        "frame={} timestamp={} components={}",
        packet.frame_number,
        packet.timestamp,
        packet.components.len()
    );
    for component in &packet.components {
        println!(
            "  {}: {}",
            component_label(component),
            summarize_component(&component.data)
        );
    }
}

fn print_frame(frame: &AssembledFrame, verbose: bool) {
    let summaries = frame
        .components
        .values()
        .map(|component| {
            format!(
                "{}={}",
                component_label(component),
                summarize_component(&component.data)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    println!(
        "frame={} timestamp={} complete={} components=[{}]",
        frame.frame_number, frame.timestamp, frame.complete, summaries
    );

    if verbose {
        for component in frame.components.values() {
            println!(
                "  {}: {}",
                component_label(component),
                summarize_component(&component.data)
            );
        }
    }
}

fn component_label(component: &Component) -> String {
    component
        .component_type()
        .map(component_type_label)
        .unwrap_or_else(|| format!("component_{}", component.id))
}

fn component_type_label(component_type: ComponentType) -> String {
    format!("{component_type:?}")
}

fn summarize_component(data: &ComponentData) -> String {
    match data {
        ComponentData::TwoD(component) | ComponentData::TwoDLinearized(component) => {
            let markers = component
                .cameras
                .iter()
                .map(|camera| camera.markers.len())
                .sum::<usize>();
            format!("{} cameras, {markers} markers", component.cameras.len())
        }
        ComponentData::ThreeD(component) => format!("{} markers", component.markers.len()),
        ComponentData::ThreeDResidual(component) => {
            format!("{} markers", component.markers.len())
        }
        ComponentData::ThreeDNoLabels(component) => {
            format!("{} markers", component.markers.len())
        }
        ComponentData::ThreeDNoLabelsResidual(component) => {
            format!("{} markers", component.markers.len())
        }
        ComponentData::SixD(component) => format!("{} bodies", component.bodies.len()),
        ComponentData::SixDResidual(component) => format!("{} bodies", component.bodies.len()),
        ComponentData::SixDEuler(component) => format!("{} bodies", component.bodies.len()),
        ComponentData::SixDEulerResidual(component) => {
            format!("{} bodies", component.bodies.len())
        }
        ComponentData::Analog(component) => {
            let channels = component
                .devices
                .iter()
                .map(|device| device.channels.len())
                .sum::<usize>();
            format!("{} devices, {channels} channels", component.devices.len())
        }
        ComponentData::AnalogSingle(component) => {
            let samples = component
                .devices
                .iter()
                .map(|device| device.samples.len())
                .sum::<usize>();
            format!("{} devices, {samples} samples", component.devices.len())
        }
        ComponentData::Force(component) => {
            let samples = component
                .plates
                .iter()
                .map(|plate| plate.samples.len())
                .sum::<usize>();
            format!("{} plates, {samples} samples", component.plates.len())
        }
        ComponentData::ForceSingle(component) => format!("{} plates", component.plates.len()),
        ComponentData::GazeVector(component) => {
            let samples = component
                .vectors
                .iter()
                .map(|series| series.samples.len())
                .sum::<usize>();
            format!("{} vectors, {samples} samples", component.vectors.len())
        }
        ComponentData::EyeTracker(component) => {
            let samples = component
                .trackers
                .iter()
                .map(|series| series.samples.len())
                .sum::<usize>();
            format!("{} trackers, {samples} samples", component.trackers.len())
        }
        ComponentData::Image(component) => {
            let bytes = component
                .images
                .iter()
                .map(|image| image.data.len())
                .sum::<usize>();
            format!("{} images, {bytes} bytes", component.images.len())
        }
        ComponentData::Timecode(component) => format!("{} entries", component.entries.len()),
        ComponentData::Skeleton(component) => {
            let segments = component
                .skeletons
                .iter()
                .map(|skeleton| skeleton.segments.len())
                .sum::<usize>();
            format!(
                "{} skeletons, {segments} segments",
                component.skeletons.len()
            )
        }
        ComponentData::Raw(bytes) => format!("{} raw bytes", bytes.len()),
    }
}

fn parse_component_list(input: &str) -> CliResult<Vec<ComponentSelection>> {
    let components = split_list(input)
        .map(parse_component)
        .collect::<Result<Vec<_>, _>>()
        .map_err(invalid_input)?;

    if components.is_empty() {
        return Err(invalid_input("component list cannot be empty"));
    }

    Ok(components)
}

fn parse_component(input: &str) -> Result<ComponentSelection, String> {
    match normalize_token(input).as_str() {
        "2d" => Ok(ComponentSelection::TwoD),
        "2dlin" | "2dlinearized" => Ok(ComponentSelection::TwoDLinearized),
        "3d" => Ok(ComponentSelection::ThreeD),
        "3dres" | "3dresidual" => Ok(ComponentSelection::ThreeDResidual),
        "3dnolabels" => Ok(ComponentSelection::ThreeDNoLabels),
        "3dnolabelsres" | "3dnolabelsresidual" => Ok(ComponentSelection::ThreeDNoLabelsResidual),
        "analog" => Ok(ComponentSelection::Analog {
            channels: Vec::new(),
        }),
        "analogsingle" => Ok(ComponentSelection::AnalogSingle {
            channels: Vec::new(),
        }),
        "force" => Ok(ComponentSelection::Force),
        "forcesingle" => Ok(ComponentSelection::ForceSingle),
        "6d" => Ok(ComponentSelection::SixD),
        "6dres" | "6dresidual" => Ok(ComponentSelection::SixDResidual),
        "6deuler" => Ok(ComponentSelection::SixDEuler),
        "6deulerres" | "6deulerresidual" => Ok(ComponentSelection::SixDEulerResidual),
        "image" => Ok(ComponentSelection::Image),
        "gazevector" => Ok(ComponentSelection::GazeVector),
        "eyetracker" => Ok(ComponentSelection::EyeTracker),
        "timecode" => Ok(ComponentSelection::Timecode),
        "skeleton" => Ok(ComponentSelection::Skeleton { global: false }),
        "skeletonglobal" | "skeleton:global" => Ok(ComponentSelection::Skeleton { global: true }),
        other => Err(format!("unsupported component '{other}'")),
    }
}

fn parse_parameter_list(input: &str) -> CliResult<Vec<ParameterSelection>> {
    let parameters = split_list(input)
        .map(parse_parameter)
        .collect::<Result<Vec<_>, _>>()
        .map_err(invalid_input)?;

    if parameters.is_empty() {
        return Err(invalid_input("parameter list cannot be empty"));
    }

    if parameters.len() > 1
        && parameters
            .iter()
            .any(|parameter| matches!(parameter, ParameterSelection::All))
    {
        return Err(invalid_input(
            "'all' cannot be combined with other parameter selections",
        ));
    }

    Ok(parameters)
}

fn parse_parameter(input: &str) -> Result<ParameterSelection, String> {
    match normalize_token(input).as_str() {
        "all" => Ok(ParameterSelection::All),
        "general" => Ok(ParameterSelection::General),
        "3d" => Ok(ParameterSelection::ThreeD),
        "6d" => Ok(ParameterSelection::SixD),
        "analog" => Ok(ParameterSelection::Analog),
        "force" => Ok(ParameterSelection::Force),
        "gazevector" => Ok(ParameterSelection::GazeVector),
        "eyetracker" => Ok(ParameterSelection::EyeTracker),
        "image" => Ok(ParameterSelection::Image),
        "skeleton" => Ok(ParameterSelection::Skeleton),
        "skeletonglobal" | "skeleton:global" => Ok(ParameterSelection::SkeletonGlobal),
        "calibration" => Ok(ParameterSelection::Calibration),
        other => Err(format!("unsupported parameter selection '{other}'")),
    }
}

fn split_list(input: &str) -> impl Iterator<Item = &str> {
    input
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
}

fn normalize_token(input: &str) -> String {
    input
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|character| !matches!(character, '-' | '_' | ' '))
        .collect()
}

fn parse_protocol_version(input: &str) -> Result<ProtocolVersion, String> {
    let (major, minor) = input
        .split_once('.')
        .ok_or_else(|| "protocol version must use MAJOR.MINOR format".to_owned())?;
    let major = major
        .parse()
        .map_err(|_| "protocol major version must be an unsigned integer".to_owned())?;
    let minor = minor
        .parse()
        .map_err(|_| "protocol minor version must be an unsigned integer".to_owned())?;
    Ok(ProtocolVersion::new(major, minor))
}

fn parse_stream_rate(input: &str) -> Result<StreamRate, String> {
    let normalized = input.trim().to_ascii_lowercase();
    if matches!(normalized.as_str(), "all" | "allframes") {
        return Ok(StreamRate::AllFrames);
    }

    if let Some(value) = normalized
        .strip_prefix("frequency:")
        .or_else(|| normalized.strip_prefix("hz:"))
    {
        return value
            .parse()
            .map(StreamRate::Frequency)
            .map_err(|_| "frequency must be an unsigned integer".to_owned());
    }

    if let Some(value) = normalized
        .strip_prefix("frequencydivisor:")
        .or_else(|| normalized.strip_prefix("divisor:"))
    {
        return value
            .parse()
            .map(StreamRate::FrequencyDivisor)
            .map_err(|_| "frequency divisor must be an unsigned integer".to_owned());
    }

    Err("stream rate must be allframes, frequency:<hz>, or divisor:<n>".to_owned())
}

fn invalid_input(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        message.into(),
    ))
}
