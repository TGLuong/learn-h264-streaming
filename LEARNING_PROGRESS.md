# Learning Progress: Camera Capture to H.264

Last updated: 2026-06-27

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
- [x] Stage 8: Implement RTP packetization for Single NAL and FU-A.
- [ ] Stage 9: Learn low-latency streaming considerations.
- [ ] Stage 10: Optional capture/encoding internals: raw frames, pixel formats, and native macOS APIs.

## Next Session Prompt

Continue with receiver-side RTP/H.264 depacketization. The learner has verified H.264 RTP packetization on a real `.h264` file, compared it with FFmpeg RTP output in Wireshark, and is ready to reconstruct FU-A RTP payloads back into the original NAL unit bytes.

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
- Added/used a `packetize` inspection path that reads a real `.h264` file, extracts NAL units, packetizes the first few NAL units with MTU 1200, and prints concise RTP summaries.
- Verified output similar to:
  - `NAL 0 type 7 SPS len 22 -> Single RTP packet seq = 0`
  - `NAL 1 type 8 PPS len 4 -> Single RTP packet seq = 1`
  - `NAL 2 type 6 SEI len 607 -> Single RTP packet seq = 2`
  - `NAL 3 type 5 IDR slice len 2546 -> FU-A packets seq=3..5`
  - `NAL 4 type 5 IDR slice len 1673 -> FU-A packets seq=6..7`
- Confirmed the sequence number behavior: each RTP packet increments sequence by 1; a fragmented NAL consumes multiple sequence numbers.
- Noted a small formatting cleanup opportunity in the summary output: avoid printing `IDR slice slice`.
- Used FFmpeg to send the raw `.h264` as RTP to `127.0.0.1:5004` and captured it with Wireshark on macOS loopback interface `lo0`.
- Solved Wireshark permission issue by recognizing it as a macOS capture permission/ChmodBPF problem, not an RTP problem.
- Learned how to decode the UDP stream as RTP/H.264 in Wireshark and inspect:
  - RTP sequence number.
  - RTP marker bit.
  - H.264 Single NAL vs FU-A payload.
  - FU-A Start and End bits.
  - Original NAL type carried in the FU header.
- Observed a real Wireshark FU-A packet for an IDR slice:
  - FU indicator type 28 means FU-A.
  - Start bit 1 and End bit 0 means the first fragment.
  - Original NAL type 5 means IDR slice.
- Understood why FFmpeg may split the same IDR NAL into fewer RTP packets than the Rust prototype: FFmpeg used a larger effective RTP payload size, while the Rust code used MTU 1200.
- Clarified marker bit meaning:
  - Marker bit marks the last RTP packet of an access unit/frame.
  - It does not simply mean "last packet of this NAL" unless that NAL is also the final NAL of the frame.
  - Exact frame/access-unit boundary detection requires more H.264 slice-header parsing or using encoder/container boundaries.

## Current Next Step: Receiver-Side Depacketization

- Next lesson: reconstruct FU-A RTP payloads back into the original H.264 NAL unit.
- Start with a focused helper in `src/rtp_packetization.rs`:
  - Input: multiple FU-A RTP payload byte slices belonging to the same NAL.
  - Output: one reconstructed NAL unit byte vector.
- Key reconstruction rule:
  - `nal_header = (fu_indicator & 0xe0) | (fu_header & 0x1f)`
- First target test:
  - Input payloads: `[0x7c, 0x85, 0xaa, 0xbb, 0xcc]` and `[0x7c, 0x45, 0xdd, 0xee]`.
  - Expected NAL: `[0x65, 0xaa, 0xbb, 0xcc, 0xdd, 0xee]`.
