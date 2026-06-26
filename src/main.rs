use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use crate::rtp_packetization::find_nal_units_annex_b;

pub mod rtp_packetization;

#[derive(Debug, PartialEq, Eq)]
enum CliCommand {
    CaptureH264 { output_path: PathBuf },
    Inspect { input_path: PathBuf },
    Packetize { input_path: PathBuf },
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GopSummary {
    sps_before_idr: bool,
    pps_before_idr: bool,
    idr_slices: usize,
    non_idr_slices: usize,
}

fn main() -> ExitCode {
    match run(env::args().collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    match parse_cli(args)? {
        CliCommand::CaptureH264 { output_path } => capture_h264(&output_path),
        CliCommand::Inspect { input_path } => inspect_h264(&input_path),
        CliCommand::Packetize { input_path } => packetize(&input_path),
    }
}

fn packetize(input_path: &Path) -> Result<(), Box<dyn Error>> {
    let bytes = fs::read(input_path)?;
    let packets = find_nal_units_annex_b(&bytes);
    println!("{packets:?}");
    Ok(())
}

fn parse_cli(args: Vec<String>) -> Result<CliCommand, String> {
    match args.as_slice() {
        [_, command, output_path] if command == "capture-h264" => Ok(CliCommand::CaptureH264 {
            output_path: output_path.into(),
        }),
        [_, command, input_path] if command == "inspect" => Ok(CliCommand::Inspect {
              input_path: input_path.into(),
        }),
        [_, command, input_path] if command == "packetize" => Ok(CliCommand::Packetize {
              input_path: input_path.into(),
        }),
        [program, ..] => Err(format!(
            "Usage: {program} capture-h264 <output-path>\nExample: {program} capture-h264 captures/rust-camera.h264"
        )),
        [] => Err(
            "Usage: video-capture capture-h264 <output-path>\nExample: video-capture capture-h264 captures/rust-camera.h264"
                .to_string(),
        ),
    }
}

fn capture_h264(output_path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let output = output_path
        .to_str()
        .ok_or("output path must be valid UTF-8")?;
    let status = Command::new("ffmpeg")
        .args(build_ffmpeg_capture_args(output))
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("ffmpeg exited with status {status}").into())
    }
}

fn build_ffmpeg_capture_args(output_path: &str) -> Vec<&str> {
    vec![
        "-f",
        "avfoundation",
        "-framerate",
        "30",
        "-video_size",
        "1280x720",
        "-i",
        "0",
        "-c:v",
        "libx264",
        "-preset",
        "ultrafast",
        "-tune",
        "zerolatency",
        "-g",
        "30",
        "-keyint_min",
        "30",
        "-sc_threshold",
        "0",
        "-f",
        "h264",
        output_path,
    ]
}

fn inspect_h264(input_path: &Path) -> Result<(), Box<dyn Error>> {
    let bytes = fs::read(input_path)?;
    let preview_len = bytes.len().min(64);
    let preview = &bytes[..preview_len];

    println!("first {preview_len} bytes: ");
    println!("{}", format_hex(preview));

    if contains_annex_b_start_code(&bytes) {
        println!("Found Annex B start code");
    } else {
        println!("No Annex B start code found in preview");
    }

    if let Some(nal_header) = first_nal_header_after_start_code(&bytes) {
        let nal_type = nal_header & 0x1f;
        println!("first nal header: 0x{nal_header:02x}");
        println!("first nal type: {nal_type}");
    }

    let nal_headers = find_nal_headers(&bytes);
    println!("NAL units:");
    for (index, nal_header) in nal_headers.iter().enumerate() {
        let nal_type = nal_header & 0x1f;
        let name = nal_type_name(nal_type);
        println!("{index}: header 0x{nal_header:02x}, type {nal_type} {name}");
    }

    println!("NAL summary:");
    let summary = count_header(&nal_headers);
    println!("{summary:?}");
    for (nal, count) in summary.into_iter() {
        let nal_type = nal & 0x1f;
        let name = nal_type_name(nal_type);
        println!("{name}: {count}");
    }

    let gops = summarize_gops(&nal_headers);

    println!("GOP summary:");
    for (index, gop) in gops.iter().enumerate() {
        println!(
            "GOP {index}: SPS before IDR: {}, PPS before IDR: {}, IDR slices: {}, non-IDR slices: {}",
            gop.sps_before_idr, gop.pps_before_idr, gop.idr_slices, gop.non_idr_slices,
        );
    }

    Ok(())
}

fn format_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_annex_b_start_code(bytes: &[u8]) -> bool {
    bytes
        .windows(4)
        .any(|window| window == [0x00, 0x00, 0x00, 0x01])
        || bytes.windows(3).any(|window| window == [0x00, 0x00, 0x01])
}

fn first_nal_header_after_start_code(bytes: &[u8]) -> Option<u8> {
    for index in 0..bytes.len() {
        if bytes[index..].starts_with(&[0x00, 0x00, 0x00, 0x01]) {
            return bytes.get(index + 4).copied();
        }
        if bytes[index..].starts_with(&[0x00, 0x00, 0x01]) {
            return bytes.get(index + 3).copied();
        }
    }
    None
}

fn count_header(bytes: &[u8]) -> HashMap<u8, usize> {
    let mut res = HashMap::new();
    for byte in bytes.iter() {
        let entry = res.entry(*byte).or_default();
        *entry += 1;
    }
    res
}

enum State {
    None,
    SPS,
    PPS,
    IDR,
    NonIDR,
}

fn summarize_gops(bytes: &[u8]) -> Vec<GopSummary> {
    let mut current = GopSummary::default();
    let mut state = State::None;
    let mut res = vec![];
    for byte in bytes.iter() {
        let nal_type = byte & 0x1f;
        match nal_type {
            7 => match state {
                State::None => {
                    current.sps_before_idr = true;
                    state = State::SPS;
                }
                State::NonIDR => {
                    res.push(current.clone());
                    current = GopSummary::default();
                    current.sps_before_idr = true;
                    state = State::SPS;
                }
                _ => {}
            },
            8 => match state {
                State::SPS => {
                    current.pps_before_idr = true;
                    state = State::PPS;
                }
                State::NonIDR => {
                    res.push(current.clone());
                    current = GopSummary::default();
                    current.sps_before_idr = false;
                    current.pps_before_idr = true;
                    state = State::PPS;
                }
                _ => {}
            },
            5 => match state {
                State::PPS => {
                    current.idr_slices += 1;
                    state = State::IDR;
                }
                State::IDR => {
                    current.idr_slices += 1;
                }
                State::NonIDR => {
                    res.push(current.clone());
                    current = GopSummary::default();
                    current.sps_before_idr = false;
                    current.pps_before_idr = false;
                    current.idr_slices += 1;
                    state = State::IDR;
                }
                _ => {}
            },
            1 => match state {
                State::IDR => {
                    current.non_idr_slices += 1;
                    state = State::NonIDR;
                }
                State::NonIDR => {
                    current.non_idr_slices += 1;
                    state = State::NonIDR;
                }
                _ => {}
            },
            _ => {}
        }
    }
    res.push(current);

    res
}

fn nal_type_name(nal_type: u8) -> &'static str {
    match nal_type {
        1 => "non-IDR slice",
        5 => "IDR slice",
        6 => "SEI",
        7 => "SPS",
        8 => "PPS",
        _ => "unknown",
    }
}

fn find_nal_headers(bytes: &[u8]) -> Vec<u8> {
    let len = bytes.len();
    let mut index = 0;
    let mut res = vec![];
    while index + 3 < len {
        if index + 4 < len {
            if bytes[index..index + 4] == [0x00, 0x00, 0x00, 0x01] {
                if let Some(nal) = bytes.get(index + 4).copied() {
                    res.push(nal);
                }
                index += 5;
                continue;
            }
        }
        if bytes[index..index + 3] == [0x00, 0x00, 0x01] {
            if let Some(nal) = bytes.get(index + 3).copied() {
                res.push(nal);
            }
            index += 4;
        } else {
            index += 1;
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_gops_from_idr_clusters() {
        let nal_headers = vec![
            0x67, 0x68, 0x65, 0x65, 0x41, 0x41, 0x67, 0x68, 0x65, 0x65, 0x65, 0x41,
        ];

        assert_eq!(
            summarize_gops(&nal_headers),
            vec![
                GopSummary {
                    sps_before_idr: true,
                    pps_before_idr: true,
                    idr_slices: 2,
                    non_idr_slices: 2,
                },
                GopSummary {
                    sps_before_idr: true,
                    pps_before_idr: true,
                    idr_slices: 3,
                    non_idr_slices: 1,
                },
            ]
        );
    }

    #[test]
    fn names_common_h264_nal_types() {
        assert_eq!(nal_type_name(7), "SPS");
        assert_eq!(nal_type_name(8), "PPS");
        assert_eq!(nal_type_name(5), "IDR slice");
        assert_eq!(nal_type_name(1), "non-IDR slice");
    }

    #[test]
    fn finds_multiple_nal_headers_after_start_codes() {
        let bytes = [
            0x00, 0x00, 0x00, 0x01, 0x67, 0xaa, 0xbb, 0x00, 0x00, 0x01, 0x68, 0xcc, 0x00, 0x00,
            0x01, 0x65,
        ];

        assert_eq!(find_nal_headers(&bytes), vec![0x67, 0x68, 0x65]);
    }

    #[test]
    fn parse_inspect_command() {
        let args = vec![
            "video-capture".to_string(),
            "inspect".to_string(),
            "capture/rust-camera.h264".to_string(),
        ];
        let command = parse_cli(args).expect("command should parse");
        assert_eq!(
            command,
            CliCommand::Inspect {
                input_path: "capture/rust-camera.h264".into()
            }
        );
    }

    #[test]
    fn detects_annex_b_start_code() {
        let bytes = [0x12, 0x00, 0x00, 0x00, 0x01, 0x67];
        assert!(contains_annex_b_start_code(&bytes));
    }

    #[test]
    fn finds_first_nal_header_after_start_code() {
        let bytes = [0x00, 0x00, 0x00, 0x01, 0x67, 0x64];

        assert_eq!(first_nal_header_after_start_code(&bytes), Some(0x67));
    }

    #[test]
    fn parses_capture_h264_command() {
        let args = vec![
            "video-capture".to_string(),
            "capture-h264".to_string(),
            "captures/rust-camera.h264".to_string(),
        ];

        let command = parse_cli(args).expect("command should parse");

        assert_eq!(
            command,
            CliCommand::CaptureH264 {
                output_path: "captures/rust-camera.h264".into(),
            }
        );
    }

    #[test]
    fn builds_avfoundation_x264_args_for_output_path() {
        let args = build_ffmpeg_capture_args("captures/rust-camera.h264");

        assert_eq!(
            args,
            vec![
                "-f",
                "avfoundation",
                "-framerate",
                "30",
                "-video_size",
                "1280x720",
                "-i",
                "0",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-tune",
                "zerolatency",
                "-g",
                "30",
                "-keyint_min",
                "30",
                "-sc_threshold",
                "0",
                "-f",
                "h264",
                "captures/rust-camera.h264",
            ]
        );
    }
}
