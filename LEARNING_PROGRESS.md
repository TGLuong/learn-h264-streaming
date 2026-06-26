# Learning Progress: Camera Capture to H.264

Last updated: 2026-06-24

## Goal

Learn how to capture video from a camera and convert it into an H.264 bitstream, using this Rust repo as the practice workspace.

## Current Level

- Starting point.
- Repo currently contains a minimal Rust binary:
  - `Cargo.toml`
  - `src/main.rs`
- No capture or encoding implementation has been added yet.

## Learning Path Status

- [x] Stage 1: Understand the video pipeline concepts.
- [x] Stage 2: Use FFmpeg CLI to capture and encode camera video.
- [x] Stage 3: Inspect raw H.264 bitstream structure.
- [ ] Stage 4: Capture frames in Rust.
- [ ] Stage 5: Convert camera frames to encoder-friendly pixel format.
- [ ] Stage 6: Encode frames to H.264 from Rust.
- [ ] Stage 7: Write raw `.h264` output and verify playback.
- [ ] Stage 8: Learn low-latency streaming considerations.
- [ ] Stage 9: Optional macOS native path: AVFoundation + VideoToolbox.

## Next Session Prompt

Continue from Stage 1 in `docs/superpowers/plans/2026-06-24-learn-camera-h264.md`. Start by explaining the camera-to-H.264 pipeline and then give me the first hands-on exercise.

## Notes

- Preferred language: Vietnamese.
- Preferred style: step-by-step, hands-on, with short theory before each exercise.
- Target platform right now: macOS, based on the current workspace environment.
- Main objective: understand the concepts first, then implement a Rust prototype.

## Notes From Stage 1

- Understood that raw camera frames are large pixel buffers such as RGB, YUV, or NV12.
- Understood that H.264 is compressed video data derived from raw frames, using keyframes and inter-frame prediction.
- Understood that YUV formats reduce chroma detail because human vision is more sensitive to brightness than color detail.
- Understood that keyframes contain enough information to reconstruct an image without previous frames.
- Understood that SPS/PPS provide decoder configuration needed to decode the H.264 stream.

## Notes From Stage 2

- Listed AVFoundation devices with FFmpeg.
- Available video devices included FaceTime HD Camera, GiaLuong Camera, and Capture screen 0.
- Captured camera video to a raw H.264 `.h264` file using FFmpeg and `libx264`.
- Verified the captured H.264 output by playing it with `ffplay`.

## Notes From Stage 3

- Inspected the beginning of `captures/camera.h264` with `xxd`.
- Observed Annex B start codes such as `00 00 00 01` and `00 00 01`.
- Observed SPS NAL starting with byte `0x67`, PPS NAL starting with `0x68`, and SEI NAL starting with `0x06`.
- The SEI payload includes x264 encoder metadata text.
- Inspected packets with `ffprobe -show_packets`.
- Observed first packet at `pos=0`, `size=3687`, `flags=K__`, meaning it is a key packet.
- Observed the second packet at `pos=3687`, `size=123`, `flags=___`, meaning it is a much smaller non-key packet.

## Notes From Stage 4

- Started with the simplest Rust approach: a CLI wrapper that spawns the known-good FFmpeg pipeline.
- Added command:
  - `cargo run -- capture-h264 captures/rust-camera.h264`
- The Rust command creates the output directory if needed and invokes FFmpeg with AVFoundation camera index `0`, `libx264`, `ultrafast`, `zerolatency`, and raw H.264 output.
- Automated tests cover CLI parsing and FFmpeg argument construction.
- Next manual verification:
  - Run `cargo run -- capture-h264 captures/rust-camera.h264`.
  - Stop FFmpeg with `q` after a few seconds.
  - Run `ffplay captures/rust-camera.h264`.
