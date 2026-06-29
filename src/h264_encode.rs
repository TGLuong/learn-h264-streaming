use std::{error::Error, fs, path::Path};

use openh264::{
    OpenH264API,
    encoder::{Encoder, EncoderConfig, IntraFramePeriod},
};

pub struct SyntheticYuvFrame {
    pub width: usize,
    pub height: usize,
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
}

impl SyntheticYuvFrame {
    pub fn generate_yuv420p_frame(width: usize, height: usize, frame_index: usize) -> Self {
        assert!(width % 2 == 0);
        assert!(height % 2 == 0);
        let mut y = Vec::with_capacity(width * height);
        for row in 0..height {
            for col in 0..width {
                let value = ((col + row + frame_index) % 256) as u8;
                y.push(value);
            }
        }
        let u = vec![128; (width / 2) * (height / 2)];
        let v = vec![128; (width / 2) * (height / 2)];
        Self {
            width,
            height,
            y,
            u,
            v,
        }
    }
}

pub fn encode_synthetic_h264_file(output_path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let bytes = encode_synthetic_h264_bytes(320, 240, 6000)?;
    fs::write(output_path, bytes)?;
    Ok(())
}

pub fn encode_synthetic_h264_bytes(
    width: usize,
    height: usize,
    frame_count: usize,
) -> Result<Vec<u8>, openh264::Error> {
    let config = EncoderConfig::new().intra_frame_period(IntraFramePeriod::from_num_frames(30));
    let api = OpenH264API::from_source();
    let mut encoder = Encoder::with_api_config(api, config)?;
    let mut output = Vec::new();

    for frame_index in 0..frame_count {
        let frame = SyntheticYuvFrame::generate_yuv420p_frame(width, height, frame_index);
        let yuv = openh264::formats::YUVSlices::new(
            (&frame.y, &frame.u, &frame.v),
            (frame.width, frame.height),
            (frame.width, frame.width / 2, frame.width / 2),
        );
        let bitstream = encoder.encode(&yuv)?;
        bitstream.write_vec(&mut output);
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_synthetic_h264_file() -> Result<(), Box<dyn std::error::Error>> {
        let output_path = std::env::temp_dir().join("video-capture-openh264-test.h264");

        encode_synthetic_h264_file(&output_path)?;

        let bytes = std::fs::read(&output_path)?;
        assert!(!bytes.is_empty());
        assert!(bytes.windows(4).any(|window| window == [0, 0, 0, 1]));

        let _ = std::fs::remove_file(output_path);

        Ok(())
    }

    #[test]
    fn encodes_multiple_synthetic_frames_to_h264_bytes() -> Result<(), openh264::Error> {
        let bytes = encode_synthetic_h264_bytes(320, 240, 10)?;

        assert!(!bytes.is_empty());

        let start_code_count = bytes
            .windows(4)
            .filter(|window| *window == [0, 0, 0, 1])
            .count();

        assert!(start_code_count >= 3);

        Ok(())
    }

    #[test]
    fn encodes_one_synthetic_yuv_frame_to_h264_bytes() -> Result<(), openh264::Error> {
        let frame = SyntheticYuvFrame::generate_yuv420p_frame(320, 240, 0);
        let yuv = openh264::formats::YUVSlices::new(
            (&frame.y, &frame.u, &frame.v),
            (frame.width, frame.height),
            (frame.width, frame.width / 2, frame.width / 2),
        );
        let mut encoder = openh264::encoder::Encoder::new()?;
        let bitstream = encoder.encode(&yuv)?;
        let bytes = bitstream.to_vec();
        assert!(!bytes.is_empty());
        assert!(bytes.windows(4).any(|window| window == [0, 0, 0, 1]));
        Ok(())
    }

    #[test]
    fn generates_yuv420p_planes_with_expected_sizes() {
        let frame = SyntheticYuvFrame::generate_yuv420p_frame(4, 2, 0);

        assert_eq!(frame.y.len(), 8);
        assert_eq!(frame.u.len(), 2);
        assert_eq!(frame.v.len(), 2);
    }

    #[test]
    fn generates_moving_luma_gradient() {
        let frame0 = SyntheticYuvFrame::generate_yuv420p_frame(4, 2, 0);
        let frame1 = SyntheticYuvFrame::generate_yuv420p_frame(4, 2, 1);

        assert_eq!(frame0.y, vec![0, 1, 2, 3, 1, 2, 3, 4]);
        assert_eq!(frame1.y, vec![1, 2, 3, 4, 2, 3, 4, 5]);
    }
}
