# OpenH264 YUV Encode Learning Design

Date: 2026-06-27

## Goal

Teach the learner how to produce an H.264 bitstream from raw YUV frames in Rust using OpenH264.

This is the next learning target after the H.264 Annex B, RTP packetization, and receiver-side depacketization lessons. The first milestone intentionally uses synthetic YUV frames instead of camera frames so the learner can focus on the encoder input layout and output bitstream before debugging camera capture APIs or pixel formats.

## Scope

In scope:

- Add a Rust path that generates synthetic YUV420P frames.
- Encode those frames with the `openh264` crate.
- Write a raw `.h264` elementary stream to `captures/openh264-test.h264`.
- Verify the output with the existing H.264 inspector and standard media tools.
- Keep the implementation learning-oriented and easy to inspect.

Out of scope for this first milestone:

- Native camera capture.
- Camera pixel format negotiation.
- RGB, YUYV, NV12, or MJPEG conversion from real devices.
- RTP streaming of the newly encoded output.
- MP4 muxing.

Those topics come after the learner can explain and verify `YUV420P frame -> OpenH264 encoder -> H.264 bitstream`.

## Proposed CLI

Add a command:

```bash
cargo run -- encode-synthetic-h264 captures/openh264-test.h264
```

The command will:

1. Generate a short synthetic video sequence, such as 60 frames at 320x240.
2. Feed each frame into OpenH264.
3. Write the encoded H.264 bytes into the requested output path.
4. Create the parent output directory when needed.

The output should be a raw `.h264` stream suitable for the existing `inspect` command:

```bash
cargo run -- inspect captures/openh264-test.h264
```

## Architecture

Add a focused encoder module:

```text
src/h264_encode.rs
```

Responsibilities:

- Hold small frame-generation helpers.
- Explain YUV420P plane sizing through simple code.
- Own OpenH264 encoder setup and frame submission.
- Return or write encoded H.264 bytes without mixing in CLI parsing.

Keep `src/main.rs` responsible for:

- CLI command parsing.
- Calling the new encoder module.
- Reporting errors.

This avoids growing `main.rs` into the place where every video concept lives.

## YUV420P Frame Model

Use a simple planar layout:

```text
Y plane: width * height bytes
U plane: (width / 2) * (height / 2) bytes
V plane: (width / 2) * (height / 2) bytes
```

The synthetic frame generator should make the Y plane visibly change over time, for example a moving gradient. U and V can start as constant values so the first lesson stays focused on luma and plane layout.

The code should use even dimensions because YUV420P chroma planes are half-width and half-height.

## Data Flow

```text
generate synthetic YUV420P frame
  -> wrap or convert into the OpenH264 input type
  -> encoder.encode(...)
  -> append encoded H.264 bytes to output file
  -> inspect generated stream
```

The learner should be able to connect this output to previous lessons:

```text
OpenH264 output
  -> raw .h264 bitstream
  -> Annex B / NAL inspection
  -> future RTP packetization
```

## Error Handling

The first version should fail clearly when:

- The output path cannot be created or written.
- Frame dimensions are not even.
- OpenH264 encoder setup or encode calls fail.

It does not need recovery logic or streaming resilience yet.

## Verification

Primary verification:

```bash
cargo test
cargo run -- encode-synthetic-h264 captures/openh264-test.h264
cargo run -- inspect captures/openh264-test.h264
```

Manual media verification:

```bash
ffplay captures/openh264-test.h264
ffprobe -hide_banner captures/openh264-test.h264
```

Expected conceptual result:

- The learner can explain the size and role of Y, U, and V planes.
- The learner can explain why this milestone avoids camera capture.
- The learner can identify SPS/PPS/IDR/non-IDR NAL units in the encoded output.

## Next Milestone After This Spec

After synthetic YUV encoding works, move to a real frame source:

1. Use FFmpeg as a capture helper that outputs raw YUV frames to Rust.
2. Feed those frames into the same OpenH264 encoder path.
3. Only after that consider native camera capture APIs and pixel format conversion.
