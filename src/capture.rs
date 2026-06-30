use std::{error::Error, fs, time::Instant};

use nokhwa::{
    Camera, nokhwa_initialize,
    pixel_format::YuyvFormat,
    utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType},
};
use openh264::{encoder::Encoder, formats::YUVSlices};
use rtp::{codecs::h264::H264Payloader, packetizer::Payloader};
use tokio::sync::mpsc;

pub async fn capture_frame_info() -> Result<(), Box<dyn Error>> {
    let mut encoder = Encoder::new()?;
    let mut payloader = H264Payloader::default();

    let (tx, mut rx) = mpsc::channel(10);
    tokio::task::spawn_blocking(move || {
        nokhwa_initialize(|_| println!("camera permission granted"));
        let index = CameraIndex::Index(0);
        let requested = RequestedFormat::new::<YuyvFormat>(RequestedFormatType::Exact(
            CameraFormat::new_from(1920, 1080, FrameFormat::YUYV, 30),
        ));
        let mut camera = Camera::new(index, requested).unwrap();
        camera.open_stream().unwrap();
        for _ in 0..300 {
            let frame = camera.frame().unwrap();
            tx.blocking_send(frame).unwrap();
        }
    });

    let mut stream_start: Option<Instant> = None;
    for i in 0..200 {
        let frame_start = Instant::now();
        let frame = rx.recv().await.unwrap();
        if i == 0 {
            stream_start = Some(Instant::now());
        }
        let width = frame.resolution().width() as usize;
        let height = frame.resolution().height() as usize;
        let t1 = Instant::now();
        if let Some((y, u, v)) = yuyv_to_yuv420p(frame.buffer(), width, height) {
            let convert_ms = t1.elapsed();
            let yuv = YUVSlices::new((&y, &u, &v), (width, height), (width, width / 2, width / 2));
            let t2 = Instant::now();
            let bitstream = encoder.encode(&yuv)?;
            let encode_ms = t2.elapsed();
            let t3 = Instant::now();
            let payloads = payloader.payload(1200, &bitstream.to_vec().into()).unwrap();
            let append_ms = t3.elapsed();
            let total_ms = frame_start.elapsed();
            if let Some(start) = stream_start {
                let elapsed = start.elapsed().as_secs_f64();
                let avg_fps = (i + 1) as f64 / elapsed;

                println!(
                    "frame {i}: convert={:.2}ms encode={:.2}ms append={:.2}ms total={:.2}ms avg_fps={:.2}",
                    convert_ms.as_secs_f64() * 1000.0,
                    encode_ms.as_secs_f64() * 1000.0,
                    append_ms.as_secs_f64() * 1000.0,
                    total_ms.as_secs_f64() * 1000.0,
                    avg_fps,
                );
            }
        }
    }

    Ok(())
}

fn yuyv_to_yuv420p(
    buffer: &[u8],
    width: usize,
    height: usize,
) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    if width % 2 != 0 || height % 2 != 0 {
        return None;
    }
    if buffer.len() != width * height * 2 {
        return None;
    }
    let mut u_plane = Vec::with_capacity((width / 2) * (height / 2));
    let mut v_plane = Vec::with_capacity((width / 2) * (height / 2));
    let mut y_plane = vec![0; width * height];

    for row_pair in 0..(height / 2) {
        let top_row = row_pair * 2;
        let bottom_row = top_row + 1;

        for pair in 0..(width / 2) {
            let top_offset = top_row * width * 2 + pair * 4;
            let bottom_offset = bottom_row * width * 2 + pair * 4;

            let x = pair * 2;

            y_plane[top_row * width + x] = buffer[top_offset];
            y_plane[top_row * width + x + 1] = buffer[top_offset + 2];

            y_plane[bottom_row * width + x] = buffer[bottom_offset];
            y_plane[bottom_row * width + x + 1] = buffer[bottom_offset + 2];

            let u1 = buffer[top_offset + 1] as u16;
            let v1 = buffer[top_offset + 3] as u16;
            let u2 = buffer[bottom_offset + 1] as u16;
            let v2 = buffer[bottom_offset + 3] as u16;

            u_plane.push(((u1 + u2) / 2) as u8);
            v_plane.push(((v1 + v2) / 2) as u8);
        }
    }
    Some((y_plane, u_plane, v_plane))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    #[test]
    fn converts_2x2_yuyv_to_yuv420p() {
        let yuyv = vec![
            10, 100, 20, 150, 
            30, 110, 40, 170
        ];

        let (y, u, v) = yuyv_to_yuv420p(&yuyv, 2, 2).expect("valid 2x2 YUYV frame");

        #[rustfmt::skip]
        assert_eq!(y, vec![10, 20, 30, 40]);
        assert_eq!(u, vec![105]);
        assert_eq!(v, vec![160]);
    }

    #[rustfmt::skip]
    #[test]
    fn converts_4x4_yuyv_to_yuv420p() {
        let yuyv: Vec<u8> = vec![
            10, 100, 11, 150, 12, 101, 13, 151, 
            20, 110, 21, 160, 22, 111, 23, 161, 
            30, 120, 31, 170, 32, 121, 33, 171, 
            40, 130, 41, 180, 42, 131, 43, 181,
        ];

        let (y, u, v) = yuyv_to_yuv420p(&yuyv, 4, 4).expect("valid 4x4 YUYV frame");

        assert_eq!(
            y,
            vec![
                10, 11, 12, 13, 20, 21, 22, 23, 30, 31, 32, 33, 40, 41, 42, 43, 
            ]
        );
        assert_eq!(u, vec![105, 106, 125, 126]);
        assert_eq!(v, vec![155, 156, 175, 176]);
    }

    #[rustfmt::skip]
    #[test]
    fn converts_6x4_yuyv_to_yuv420p() {
        let yuyv: Vec<u8> = vec![
            10, 100, 11, 150, 12, 101, 13, 151, 14, 102, 15, 152,
            20, 110, 21, 160, 22, 111, 23, 161, 24, 112, 25, 162,
            30, 120, 31, 170, 32, 121, 33, 171, 34, 122, 35, 172,
            40, 130, 41, 180, 42, 131, 43, 181, 44, 132, 45, 182,
        ];

        let (y, u, v) = yuyv_to_yuv420p(&yuyv, 6, 4).expect("valid 6x4 YUYV frame");

        assert_eq!(
            y,
            vec![
                10, 11, 12, 13, 14, 15,
                20, 21, 22, 23, 24, 25,
                30, 31, 32, 33, 34, 35,
                40, 41, 42, 43, 44, 45,
            ]
        );
        assert_eq!(u, vec![105, 106, 107, 125, 126, 127]);
        assert_eq!(v, vec![155, 156, 157, 175, 176, 177]);
    }
}
