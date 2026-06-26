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

Current next stage:

- Stage 6: H.264 over RTP packetization.

Recommended next lesson:

1. Explain H.264 over RTP at a high level.
2. Emphasize that RTP payloading starts from NAL unit bytes, not Annex B start codes.
3. Guide the learner to implement full Annex B NAL unit extraction:

```rust
fn find_nal_units_annex_b(bytes: &[u8]) -> Vec<&[u8]>
```

Each returned slice should start with the NAL header byte and exclude the Annex B start code.

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

The next conceptual bridge is:

```text
Annex B H.264 stream
  -> full NAL unit byte ranges
  -> RTP H.264 payloader
  -> Single NAL Unit packets or FU-A fragmented packets
```
