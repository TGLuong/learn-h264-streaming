# Learning Progress: Camera Capture to H.264

Last updated: 2026-06-27

## Goal

Learn how to capture camera video, understand the H.264 bitstream structure, and eventually packetize H.264 for real-time transport such as RTP.

## Current Level

- Can capture camera video to raw H.264 by launching FFmpeg from Rust.
- Can inspect raw H.264 Annex B streams with a Rust inspector.
- Can packetize and depacketize learning-oriented H.264 RTP payloads in Rust.
- Can generate synthetic YUV420P frames and encode them into a raw H.264 bitstream with OpenH264.
- Current learning focus: understand `YUV420P frame -> OpenH264 encoder -> H.264 bitstream`, then connect a real camera/raw YUV frame source later.

## Learning Path Status

- [x] Stage 1: Understand the video pipeline concepts.
- [x] Stage 2: Use FFmpeg CLI to capture and encode camera video.
- [x] Stage 3: Inspect raw H.264 Annex B bitstream structure.
- [x] Stage 4: Build a Rust H.264 inspector for NAL units and GOP summaries.
- [x] Stage 5: Understand raw H.264 stream vs MP4 container and Annex B vs AVCC.
- [x] Stage 6: Learn H.264 over RTP packetization.
- [x] Stage 7: Implement simple H.264 NAL unit extraction for RTP payloading.
- [x] Stage 8: Implement RTP packetization for Single NAL and FU-A.
- [x] OpenH264 Stage 1: Generate synthetic YUV420P frames.
- [x] OpenH264 Stage 2: Encode synthetic YUV420P frames into raw `.h264`.
- [x] OpenH264 Stage 3: Verify OpenH264 output with the existing Annex B/NAL inspector.
- [x] OpenH264 Stage 4: Configure periodic intra/keyframe generation with `IntraFramePeriod`.
- [ ] OpenH264 Stage 5: Parameterize and clean up synthetic encoder settings.
- [ ] OpenH264 Stage 6: Feed real captured YUV frames into the same OpenH264 path.
- [ ] Stage 9: Learn low-latency streaming considerations.
- [ ] Stage 10: Optional capture/encoding internals: raw frames, pixel formats, and native macOS APIs.

## Next Session Prompt

Continue from the OpenH264 synthetic YUV encoder path. The learner wants to code themselves and use the assistant as a guide/reviewer. They have added `openh264 = "0.9.3"`, created `src/h264_encode.rs`, generated synthetic YUV420P frames, encoded them with OpenH264, wrote `captures/openh264-test.h264`, inspected the output with the existing `inspect` command, and confirmed periodic keyframes by configuring `IntraFramePeriod::from_num_frames(30)` via `Encoder::with_api_config(OpenH264API::from_source(), config)`.

Next useful step: avoid hard-coded encoder settings by parameterizing `encode_synthetic_h264_bytes(width, height, frame_count, intra_period)`, then add a test that uses `find_nal_units_annex_b` to confirm a periodic intra period produces multiple IDR NAL units. After that, update the CLI/file path to use those settings intentionally and discuss the bridge from synthetic frames to real captured YUV frames.

## Notes

- Preferred language: Vietnamese.
- Preferred style: step-by-step, hands-on, with short theory before each exercise.
- Target platform right now: macOS, based on the current workspace environment.
- Main objective: understand the concepts first, then implement a Rust prototype.
- Current priority: learn raw YUV frame encoding with OpenH264 before moving to real camera YUV capture.

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

- Implemented FU-A reconstruction back into the original H.264 NAL unit.
- Key reconstruction rule learned:
  - `nal_header = (fu_indicator & 0xe0) | (fu_header & 0x1f)`
- Implemented a learning-oriented `RtpDepacketizer` with:
  - `push_in(packet: RtpPacket)` to feed RTP packets into the receiver side.
  - `pop_out() -> Option<Vec<u8>>` to retrieve one reconstructed/output NAL unit at a time.
- Confirmed receiver-side payload shapes:
  - Single NAL: `1 RTP -> 1 NAL`
  - FU-A: `many RTP -> 1 NAL`
  - STAP-A: `1 RTP -> many NAL`
- Learned STAP-A structure:
  - RTP payload type 24.
  - First byte is STAP-A header.
  - Each contained NAL is encoded as 2-byte big-endian length followed by NAL bytes.
  - Example: `78 00 03 67 aa bb 00 02 68 cc` outputs `[67 aa bb]` and `[68 cc]`.
- Added tests for:
  - FU-A payload reconstruction helper.
  - `RtpDepacketizer` outputting a Single NAL packet.
  - Mixed Single NAL and FU-A packets in order.
  - STAP-A outputting each aggregated NAL.
- Latest verification:
  - `cargo test` passes with 17 tests.
- Cleanup opportunities:
  - Remove debug `println!` calls from FU-A/depacketizer paths.
  - Consider changing `reconstruct_fu_a_payloads(payload: &Vec<Vec<u8>>)` to a slice-based signature.
  - Consider validating malformed STAP-A payloads instead of silently ignoring truncated NAL data.

## Next Conceptual Step: RTP Receiver Robustness

- Decide how the depacketizer should behave for malformed or incomplete RTP/H.264 payloads.
- Useful next exercises:
  - Add a test for truncated STAP-A length fields.
  - Add a test for a FU-A end fragment arriving without a start fragment.
  - Add simple sequence-number gap detection for fragmented FU-A NALs.
  - Discuss marker bit and access-unit/frame boundary handling again, now from the receiver side.

## Notes From OpenH264 Synthetic YUV Encoding

- Shifted learning goal from RTP receiver robustness to encoding raw YUV frames into H.264 using OpenH264.
- Added/used design spec:
  - `docs/superpowers/specs/2026-06-27-openh264-yuv-encode-design.md`
- Added dependency:
  - `openh264 = "0.9.3"`
- Created `src/h264_encode.rs` with a learning-oriented `SyntheticYuvFrame`.
- Learned YUV420P plane sizes:
  - Y plane: `width * height`
  - U plane: `(width / 2) * (height / 2)`
  - V plane: `(width / 2) * (height / 2)`
- Learned that U/V value `128` is neutral chroma, so changing only Y creates grayscale brightness changes.
- Learned stride meaning:
  - Width is the meaningful pixel count per row.
  - Stride is the byte distance in memory from the start of one row to the start of the next row.
  - For the current no-padding synthetic YUV420P frames: `(y_stride, u_stride, v_stride) = (width, width / 2, width / 2)`.
- Used `openh264::formats::YUVSlices::new` to wrap existing Y, U, and V buffers without needing a camera source yet.
- Encoded one synthetic YUV frame and verified output bytes contain Annex B start codes.
- Encoded multiple synthetic frames and wrote raw H.264 bytes to `captures/openh264-test.h264`.
- Added CLI command:
  - `cargo run -- encode-synthetic-h264 captures/openh264-test.h264`
- Verified OpenH264 output with:
  - `cargo run -- inspect captures/openh264-test.h264`
- Observed valid H.264 output with SPS, PPS, IDR slice, and non-IDR slices.
- For a 6000-frame synthetic stream with default `Encoder::new()`, observed only one IDR at the beginning:
  - SPS: 1
  - PPS: 1
  - IDR slice: 1
  - non-IDR slice: 5999
- Learned that OpenH264's default `EncoderConfig::new()` has `intra_frame_period` set to `IntraFramePeriod::from_num_frames(0)`, which disables periodic intra frames.
- Configured periodic keyframes with:
  - `EncoderConfig::new().intra_frame_period(IntraFramePeriod::from_num_frames(30))`
  - `Encoder::with_api_config(OpenH264API::from_source(), config)`
- Confirmed that setting an intra period creates many more IDR/keyframes.
- Current cleanup/next coding task:
  - Parameterize width, height, frame count, and intra period instead of hard-coding them.
  - Add a test that encodes about 90 frames with intra period 30 and counts IDR NAL units using `find_nal_units_annex_b`.
