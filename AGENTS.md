# Agent Notes

This repository is being used as a Vietnamese, hands-on learning workspace for H.264 video.

## Read First

At the start of a new session, read these files before guiding the learner:

- `LEARNING_PROGRESS.md`
- `docs/superpowers/plans/2026-06-24-learn-camera-h264.md`
- `docs/superpowers/specs/2026-06-27-openh264-yuv-encode-design.md`
- `src/main.rs`

## Current Learning Goal

The learner has shifted the current priority to learning how to encode raw YUV frames into an H.264 bitstream from Rust using OpenH264.

The active plan is `docs/superpowers/specs/2026-06-27-openh264-yuv-encode-design.md`.

Start with synthetic YUV420P frames before real camera capture. Do not jump directly into native camera APIs, RGB/YUV conversion from real devices, or macOS AVFoundation/VideoToolbox unless the learner explicitly changes the plan.

## Current Progress

Completed:

- Camera capture to raw `.h264` using FFmpeg from Rust.
- Annex B start code detection.
- NAL header and NAL type inspection.
- SPS/PPS/SEI/IDR/non-IDR identification.
- Slice vs frame basics.
- GOP summary from IDR slice clusters.
- x264 GOP interval experiment with `-g 30`, `-keyint_min 30`, and `-sc_threshold 0`.
- Raw `.h264` vs MP4 container comparison.
- Annex B vs AVCC comparison.
- Basic mux/demux/remux terminology.
- H.264 over RTP concepts: Single NAL packets vs FU-A fragmentation.
- Full Annex B NAL unit extraction for RTP payloading.
- Learning-oriented RTP packet model.
- Single NAL RTP packetization.
- FU-A packetization with tests for normal fragmentation and exact final chunk handling.
- RTP packetization verification with a real `.h264` file.
- Wireshark inspection of RTP/H.264 Single NAL and FU-A packets.
- Receiver-side depacketization basics:
  - Single NAL: `1 RTP -> 1 NAL`
  - FU-A: `many RTP -> 1 NAL`
  - STAP-A: `1 RTP -> many NAL`
- OpenH264 synthetic YUV encoding:
  - Generated YUV420P frames in Rust.
  - Encoded synthetic YUV frames with `openh264`.
  - Wrote `captures/openh264-test.h264`.
  - Verified the output with the existing Annex B/NAL inspector.
  - Learned stride vs width for planar YUV input.
  - Configured periodic IDR/keyframes with `IntraFramePeriod::from_num_frames(30)`.

Current next stage:

- OpenH264 YUV encoding milestone:
  - Parameterize width, height, frame count, and intra period instead of hard-coding them.
  - Add a test that counts IDR NAL units using `find_nal_units_annex_b`.
  - Then bridge from synthetic frames to a real captured YUV frame source.

Recommended next lesson:

1. Read the OpenH264 YUV encode design spec.
2. Read `src/h264_encode.rs` and review the learner's latest code.
3. Guide the learner to parameterize:
   - `width`
   - `height`
   - `frame_count`
   - `intra_period`
4. Add/review a test like:
   - encode 90 frames with intra period 30.
   - extract NAL units with `find_nal_units_annex_b`.
   - count type 5 IDR NAL units.
   - assert there are multiple IDR frames.

## Teaching Style

- Use Vietnamese.
- Keep theory short, then give one concrete exercise.
- Prefer step-by-step guidance.
- Let the learner write code themselves when they ask to learn by coding.
- Review their code instead of taking over, unless they explicitly ask for a fix.
- When reviewing, lead with bugs or conceptual risks, then mention tests.
- For this new OpenH264 section, keep theory short and prefer one concrete code exercise at a time.

## Useful Commands

Run tests:

```bash
cargo test
```

Check formatting:

```bash
cargo fmt --check
```

Capture H.264:

```bash
cargo run -- capture-h264 captures/rust-camera-g30.h264
```

Inspect H.264:

```bash
cargo run -- inspect captures/rust-camera-g30.h264
```

Planned synthetic OpenH264 encode command:

```bash
cargo run -- encode-synthetic-h264 captures/openh264-test.h264
```

Remux raw H.264 to MP4:

```bash
ffmpeg -framerate 30 -i captures/rust-camera-g30.h264 -c copy captures/rust-camera-g30.mp4
```

Compare raw stream and MP4 container:

```bash
ffprobe -hide_banner captures/rust-camera-g30.h264
ffprobe -hide_banner captures/rust-camera-g30.mp4
```

## Important Conceptual State

The learner currently understands:

- `.h264` raw stream is usually Annex B: start code + NAL.
- MP4 stores H.264 samples using AVCC length prefixes.
- `nal_header & 0x1f` gives the H.264 NAL type.
- IDR slice count is not the same as keyframe count.
- Multiple slices can form one frame.
- A simple GOP can be viewed as an IDR frame candidate plus following non-IDR frames.
- `-g 30` at about 30fps creates roughly 1-second GOPs.
- RTP payloading uses NAL unit bytes and does not include Annex B start codes.
- Single NAL RTP packet payload is the original NAL bytes.
- FU-A splits a large NAL across multiple RTP packets.
- FU indicator keeps F/NRI and sets type 28.
- FU header stores Start/End flags and original NAL type.
- Marker bit means the last RTP packet of an access unit/frame, not "this NAL fits in one packet".
- Receiver depacketization reconstructs H.264 NAL units from RTP payloads.
- YUV420P uses separate Y, U, and V planes.
- For no-padding synthetic YUV420P, strides are `(width, width / 2, width / 2)`.
- `YUVSlices::new` can wrap existing Y/U/V buffers for OpenH264 input.
- `Encoder::new()` uses default OpenH264 config and does not create periodic keyframes by default.
- In `openh264 0.9.3`, use `Encoder::with_api_config(OpenH264API::from_source(), config)` to apply custom encoder config.
- `IntraFramePeriod::from_num_frames(30)` creates periodic intra/keyframes roughly every 30 frames.

The learner is now moving from "produce H.264 from synthetic YUV frames" to "make the encoder path configurable and ready for real YUV frame input":

```text
synthetic YUV420P frames
  -> OpenH264 encoder
  -> raw .h264 bitstream
  -> existing H.264 inspector
```

The next conceptual bridge is:

```text
YUV420P frame layout
  -> encoder input frame
  -> OpenH264 output bytes
  -> Annex B / NAL inspection
  -> configurable keyframe interval
  -> real captured YUV frames
```
