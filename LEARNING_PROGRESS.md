# Learning Progress: Camera Capture to H.264

Last updated: 2026-06-26

## Goal

Learn how to capture camera video, understand the H.264 bitstream structure, and eventually packetize H.264 for real-time transport such as RTP.

## Current Level

- Can capture camera video to raw H.264 by launching FFmpeg from Rust.
- Can inspect raw H.264 Annex B streams with a Rust inspector.
- Current learning focus: understand H.264 stream structure and H.264 over RTP packetization before going deeper into raw camera frame capture.

## Learning Path Status

- [x] Stage 1: Understand the video pipeline concepts.
- [x] Stage 2: Use FFmpeg CLI to capture and encode camera video.
- [x] Stage 3: Inspect raw H.264 Annex B bitstream structure.
- [x] Stage 4: Build a Rust H.264 inspector for NAL units and GOP summaries.
- [x] Stage 5: Understand raw H.264 stream vs MP4 container and Annex B vs AVCC.
- [x] Stage 6: Learn H.264 over RTP packetization.
- [x] Stage 7: Implement simple H.264 NAL unit extraction for RTP payloading.
- [ ] Stage 8: Implement RTP packetization for Single NAL and FU-A.
- [ ] Stage 9: Learn low-latency streaming considerations.
- [ ] Stage 10: Optional capture/encoding internals: raw frames, pixel formats, and native macOS APIs.

## Next Session Prompt

Continue from Stage 8 Step 4 in `docs/superpowers/plans/2026-06-24-learn-camera-h264.md`. The learner has implemented Annex B NAL extraction, Single NAL RTP packetization, and FU-A fragmentation. Next, guide them to verify packetization on the real `captures/rust-camera-g30.h264` file by printing concise per-NAL RTP summaries.

## Notes

- Preferred language: Vietnamese.
- Preferred style: step-by-step, hands-on, with short theory before each exercise.
- Target platform right now: macOS, based on the current workspace environment.
- Main objective: understand the concepts first, then implement a Rust prototype.
- Current priority: understand H.264 stream structure and RTP packetization. Raw frame capture and pixel conversion are deferred until after the H.264/RTP path is clear.

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

## Notes From H.264 Stream Study

- Added Rust inspector logic to find Annex B start codes and NAL headers.
- Learned that `nal_header & 0x1f` gives the H.264 NAL unit type.
- Identified SPS (`0x67`, type 7), PPS (`0x68`, type 8), SEI (`0x06`, type 6), IDR slices (`0x65`, type 5), and non-IDR slices (`0x41`, type 1).
- Learned that one frame can contain multiple slices, so IDR slice count is not the same as keyframe count.
- Built GOP summaries by grouping IDR slice clusters and following non-IDR slices.
- Observed x264 default GOP interval around 250 frames, about 8.3 seconds at 30fps.
- Added `-g 30`, `-keyint_min 30`, and `-sc_threshold 0` to observe 30-frame GOPs, about 1 second at 30fps.
- Compared raw `.h264` and `.mp4`: raw H.264 lacks container duration/bitrate metadata, while MP4 adds timeline, track metadata, and index information.
- Compared Annex B vs AVCC:
  - Annex B raw H.264 starts NAL units with start codes such as `00 00 00 01`.
  - MP4 stores H.264 samples with length prefixes such as `00 00 00 16`.

## Notes From H.264 over RTP Study

- Learned that RTP payloading starts from NAL unit bytes, not Annex B start codes.
- Implemented `find_nal_units_annex_b(bytes: &[u8]) -> Vec<&[u8]>` in `src/rtp_packetization.rs`.
- Learned that if `payload[0] & 0x1f` is in `1..=23`, the RTP payload is a Single NAL Unit packet.
- Learned that if `payload[0] & 0x1f == 28`, the RTP payload is FU-A.
- Implemented a learning-oriented `RtpPacket` struct.
- Implemented Single NAL packetization: small NAL units become one RTP packet whose payload is the original NAL bytes.
- Implemented FU-A fragmentation for large NAL units.
- Learned FU-A payload structure:
  - FU indicator: keeps original F/NRI bits and uses NAL type 28.
  - FU header: stores Start/End flags and the original NAL type.
- Added tests for:
  - Annex B NAL extraction without start codes.
  - Single NAL RTP packetization.
  - FU-A fragmentation.
  - Exact final FU-A chunk handling.
- Current next step: verify RTP packetization on a real `.h264` file by printing concise summaries such as:
  - `NAL 0 type 7 SPS len 22 -> Single RTP packet seq=100`
  - `NAL 3 type 5 IDR len 50350 -> FU-A packets seq=103..145`
