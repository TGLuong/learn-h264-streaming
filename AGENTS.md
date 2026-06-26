# Agent Notes

This repository is being used as a Vietnamese, hands-on learning workspace for H.264 video.

## Read First

At the start of a new session, read these files before guiding the learner:

- `LEARNING_PROGRESS.md`
- `docs/superpowers/plans/2026-06-24-learn-camera-h264.md`
- `src/main.rs`

## Current Learning Goal

The learner's current priority is to understand H.264 stream structure and H.264 over RTP packetization.

Do not drift back into raw camera frame capture, RGB/YUV pixel format conversion, or native camera APIs unless the learner explicitly asks. Those topics are deferred until after the H.264/RTP path is clear.

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

Current next stage:

- Stage 8 Step 4: verify RTP packetization using a real `.h264` file.

Recommended next lesson:

1. Read `src/rtp_packetization.rs`.
2. Check whether the debug `println!("{packets:?}")` in the FU-A test is still present; recommend removing it if the learner asks for cleanup.
3. Guide the learner to add an inspect/summary path that reads `captures/rust-camera-g30.h264`, extracts NAL units, packetizes the first few NAL units with MTU 1200, and prints concise summaries:

```text
NAL 0 type 7 SPS len 22 -> Single RTP packet seq=100
NAL 1 type 8 PPS len 4 -> Single RTP packet seq=101
NAL 2 type 6 SEI len 690 -> Single RTP packet seq=102
NAL 3 type 5 IDR len 50350 -> FU-A packets seq=103..145
```

4. After this verification step, the next conceptual step is receiver-side depacketization/reconstruction.

## Teaching Style

- Use Vietnamese.
- Keep theory short, then give one concrete exercise.
- Prefer step-by-step guidance.
- Let the learner write code themselves when they ask to learn by coding.
- Review their code instead of taking over, unless they explicitly ask for a fix.
- When reviewing, lead with bugs or conceptual risks, then mention tests.

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

The next conceptual bridge is:

```text
Annex B H.264 stream
  -> full NAL unit byte ranges
  -> RTP H.264 payloader summaries on real data
  -> receiver-side depacketization/reconstruction
```
