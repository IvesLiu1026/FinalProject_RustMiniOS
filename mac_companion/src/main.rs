use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serialport::SerialPort;

#[path = "../../shared/media_manifest.rs"]
mod media_manifest;

const DEFAULT_BAUD: u32 = 921_600;

#[derive(Debug, Clone)]
struct Config {
    port: String,
    baud: u32,
    media_root: PathBuf,
}

#[derive(Debug, Clone)]
struct Catalog {
    stills: Vec<StillEntry>,
    motion: Vec<MotionEntry>,
}

#[derive(Debug, Clone)]
struct StillEntry {
    label: String,
    width: u16,
    height: u16,
    scale: u16,
    path: PathBuf,
}

#[derive(Debug, Clone)]
struct MotionEntry {
    label: String,
    width: u16,
    height: u16,
    scale: u16,
    frame_delay_ms: u16,
    frames: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = parse_args(env::args().skip(1).collect())?;
    let catalog = Catalog::load(&config.media_root)?;
    let mut port = serialport::new(&config.port, config.baud)
        .timeout(Duration::from_millis(100))
        .open()?;

    println!(
        "MiniOS Mac companion serving {} stills / {} motion clips on {} @ {} baud",
        catalog.stills.len(),
        catalog.motion.len(),
        config.port,
        config.baud
    );

    serve_loop(&mut *port, &catalog)
}

fn parse_args(args: Vec<String>) -> Result<Config, Box<dyn Error>> {
    let mut port = None;
    let mut baud = DEFAULT_BAUD;
    let mut media_root = PathBuf::from("assets/test_media/converted");
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--port" => {
                index += 1;
                port = args.get(index).cloned();
            }
            "--baud" => {
                index += 1;
                baud = args
                    .get(index)
                    .ok_or("missing value for --baud")?
                    .parse::<u32>()?;
            }
            "--media-root" => {
                index += 1;
                media_root =
                    PathBuf::from(args.get(index).ok_or("missing value for --media-root")?);
            }
            "--list-ports" => {
                print_ports()?;
                std::process::exit(0);
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                return Err(format!("unknown argument: {other}").into());
            }
        }
        index += 1;
    }

    let Some(port) = port else {
        print_usage();
        print_ports()?;
        return Err("missing --port".into());
    };

    Ok(Config {
        port,
        baud,
        media_root,
    })
}

fn print_usage() {
    println!("Usage:");
    println!(
        "  cargo run --manifest-path mac_companion/Cargo.toml -- --port /dev/tty.usbserial-XXXX"
    );
    println!("Optional:");
    println!("  --baud 921600");
    println!("  --media-root assets/test_media/converted");
    println!("  --list-ports");
}

fn print_ports() -> Result<(), Box<dyn Error>> {
    println!("Available serial ports:");
    for port in serialport::available_ports()? {
        println!("  {}", port.port_name);
    }
    Ok(())
}

fn serve_loop(port: &mut dyn SerialPort, catalog: &Catalog) -> Result<(), Box<dyn Error>> {
    loop {
        let Some(line) = read_line(port)? else {
            continue;
        };

        if line == "HELLO|1" {
            send_catalog(port, catalog)?;
            continue;
        }

        if let Some(request) = line.strip_prefix("GET|S|") {
            if let Ok(index) = request.parse::<usize>() {
                if let Some(still) = catalog.stills.get(index) {
                    send_still(port, index, still)?;
                }
            }
            continue;
        }

        if let Some(request) = line.strip_prefix("GET|M|") {
            let mut parts = request.split('|');
            let clip_index = parts.next().and_then(|value| value.parse::<usize>().ok());
            let frame_index = parts.next().and_then(|value| value.parse::<usize>().ok());
            if let (Some(clip_index), Some(frame_index)) = (clip_index, frame_index) {
                if let Some(clip) = catalog.motion.get(clip_index) {
                    send_motion_frame(port, clip_index, frame_index, clip)?;
                }
            }
        }
    }
}

fn send_catalog(port: &mut dyn SerialPort, catalog: &Catalog) -> Result<(), Box<dyn Error>> {
    writeln!(
        port,
        "READY|{}|{}",
        catalog.stills.len(),
        catalog.motion.len()
    )?;
    for (index, still) in catalog.stills.iter().enumerate() {
        writeln!(
            port,
            "S|{}|{}|{}|{}|{}",
            index,
            still.width,
            still.height,
            still.scale,
            sanitize_label(&still.label)
        )?;
    }
    for (index, clip) in catalog.motion.iter().enumerate() {
        writeln!(
            port,
            "M|{}|{}|{}|{}|{}|{}|{}",
            index,
            clip.width,
            clip.height,
            clip.scale,
            clip.frame_delay_ms,
            clip.frames.len(),
            sanitize_label(&clip.label)
        )?;
    }
    writeln!(port, "END")?;
    port.flush()?;
    Ok(())
}

fn send_still(
    port: &mut dyn SerialPort,
    index: usize,
    still: &StillEntry,
) -> Result<(), Box<dyn Error>> {
    let payload = fs::read(&still.path)?;
    writeln!(
        port,
        "FRAME|S|{}|{}|{}|{}|{}",
        index,
        still.width,
        still.height,
        still.scale,
        payload.len()
    )?;
    port.write_all(&payload)?;
    port.flush()?;
    Ok(())
}

fn send_motion_frame(
    port: &mut dyn SerialPort,
    clip_index: usize,
    frame_index: usize,
    clip: &MotionEntry,
) -> Result<(), Box<dyn Error>> {
    if clip.frames.is_empty() {
        return Ok(());
    }
    let actual_index = frame_index % clip.frames.len();
    let payload = fs::read(&clip.frames[actual_index])?;
    writeln!(
        port,
        "FRAME|M|{}|{}|{}|{}|{}|{}",
        clip_index,
        actual_index,
        clip.width,
        clip.height,
        clip.scale,
        payload.len()
    )?;
    port.write_all(&payload)?;
    port.flush()?;
    Ok(())
}

fn read_line(port: &mut dyn SerialPort) -> Result<Option<String>, Box<dyn Error>> {
    let mut bytes = Vec::with_capacity(96);
    let mut scratch = [0u8; 1];
    loop {
        match port.read(&mut scratch) {
            Ok(0) => return Ok(None),
            Ok(_) => match scratch[0] {
                b'\n' => {
                    let line = String::from_utf8(bytes)?;
                    return Ok(Some(line));
                }
                b'\r' => {}
                byte => {
                    bytes.push(byte);
                }
            },
            Err(err) if err.kind() == std::io::ErrorKind::TimedOut => {
                if bytes.is_empty() {
                    return Ok(None);
                }
            }
            Err(err) => return Err(err.into()),
        }
    }
}

impl Catalog {
    fn load(media_root: &Path) -> Result<Self, Box<dyn Error>> {
        let manifests_root = media_root.join("manifests");
        let firmware_root = media_root.join("firmware");
        let stills_root = firmware_root.join("stills");
        let motion_root = firmware_root.join("motion");

        let stills = media_manifest::collect_stills(&stills_root, &manifests_root)
            .into_iter()
            .map(|entry| StillEntry {
                label: entry.label,
                width: entry.width,
                height: entry.height,
                scale: entry.scale,
                path: entry.path,
            })
            .collect::<Vec<_>>();
        let motion = media_manifest::collect_motion_clips(&motion_root, &manifests_root)
            .into_iter()
            .map(|entry| MotionEntry {
                label: entry.label,
                width: entry.width,
                height: entry.height,
                scale: entry.scale,
                frame_delay_ms: entry.frame_delay_ms,
                frames: entry.frames,
            })
            .collect::<Vec<_>>();

        Ok(Self { stills, motion })
    }
}

fn sanitize_label(label: &str) -> String {
    label
        .replace('|', " ")
        .replace('\r', " ")
        .replace('\n', " ")
}
