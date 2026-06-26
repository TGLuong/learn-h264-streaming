# Learn Camera Capture to H.264 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Teach camera capture to H.264 bitstream step by step, ending with a Rust prototype that writes playable H.264 output.

**Architecture:** Learn the pipeline in layers: first observe it with FFmpeg CLI, then inspect the H.264 output, then implement capture and encoding in Rust. Keep each milestone independently testable so the learner can verify understanding before moving on.

**Tech Stack:** Rust, FFmpeg CLI, H.264/AVC concepts, camera capture APIs, optional macOS AVFoundation and VideoToolbox later.

---

## File Structure

- `LEARNING_PROGRESS.md`: persistent learning memory for future sessions.
- `docs/superpowers/plans/2026-06-24-learn-camera-h264.md`: this learning roadmap.
- `src/main.rs`: later Rust prototype entry point.
- `Cargo.toml`: later dependencies for capture and encoding experiments.
- `captures/`: later generated local output files such as `.h264`, `.mp4`, or sample frame dumps.

## Stage 1: Video Pipeline Concepts

**Objective:** Understand what happens between camera hardware and an H.264 bitstream.

- [ ] **Step 1: Learn the basic pipeline**

Read and explain this pipeline:

```text
Camera sensor
  -> camera driver/backend
  -> frames in a pixel format such as RGB, YUYV, NV12, or MJPEG
  -> optional pixel format conversion
  -> H.264 encoder
  -> H.264 NAL units
  -> raw .h264 file, MP4 muxer, or network stream
```

Key terms to understand:

```text
frame
resolution
framerate
pixel format
colorspace
encoder
bitrate
keyframe
NAL unit
SPS
PPS
IDR frame
```

- [ ] **Step 2: Confirm understanding**

Answer these questions in your own words:

```text
1. What is the difference between a raw camera frame and an encoded H.264 packet?
2. Why does an encoder often require YUV420P or NV12 instead of RGB?
3. What is a keyframe?
4. Why do SPS and PPS matter?
```

Expected result: you can describe the pipeline without code.

## Stage 2: Capture and Encode with FFmpeg CLI

**Objective:** Use FFmpeg as a working reference before writing Rust code.

- [ ] **Step 1: List cameras on macOS**

Run:

```bash
ffmpeg -f avfoundation -list_devices true -i ""
```

Expected result: FFmpeg prints available video and audio devices. Choose the camera index, usually `0`.

- [ ] **Step 2: Record a short raw H.264 stream**

Run:

```bash
mkdir -p captures
ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i "0" -c:v libx264 -preset ultrafast -tune zerolatency -f h264 captures/camera.h264
```

Stop after a few seconds with `q`.

Expected result: `captures/camera.h264` exists and contains raw H.264 Annex B data.

- [ ] **Step 3: Play the raw H.264 file**

Run:

```bash
ffplay captures/camera.h264
```

Expected result: the recorded camera video plays back.

## Stage 3: Inspect H.264 Bitstream

**Objective:** See that H.264 output is not a sequence of images, but a sequence of encoded NAL units.

- [ ] **Step 1: Inspect packets**

Run:

```bash
ffprobe -show_packets -select_streams v captures/camera.h264
```

Expected result: FFprobe prints video packets with timestamps or packet sizes.

- [ ] **Step 2: Look for Annex B start codes**

Run:

```bash
xxd -l 128 captures/camera.h264
```

Expected result: the file contains byte sequences similar to:

```text
00 00 00 01
00 00 01
```

Those are Annex B start codes that separate NAL units.

- [ ] **Step 3: Learn important NAL unit types**

Memorize:

```text
Type 7: SPS
Type 8: PPS
Type 5: IDR keyframe
Type 1: non-IDR video slice
```

Expected result: you can explain why a decoder needs SPS and PPS before decoding video frames.

## Stage 4: Capture Frames in Rust

**Objective:** Write a Rust program that opens the camera and receives frames.

- [ ] **Step 1: Choose first capture approach**

Recommended first path:

```text
Use FFmpeg or GStreamer from Rust if the goal is to learn the full pipeline quickly.
Use a Rust camera crate or native API later if the goal is low-level control.
```

For this repo, start simple:

```text
Rust launches or wraps a known-good capture pipeline, then later moves lower-level.
```

- [ ] **Step 2: Add a small Rust command structure**

Planned commands:

```text
cargo run -- capture-h264 captures/rust-camera.h264
cargo run -- inspect captures/rust-camera.h264
```

Expected result: the app has obvious learning-oriented commands.

- [ ] **Step 3: Verify output**

Run:

```bash
ffplay captures/rust-camera.h264
```

Expected result: playback works.

## Stage 5: Pixel Format Conversion

**Objective:** Understand why captured frames often need conversion before encoding.

- [ ] **Step 1: Learn common camera formats**

Study:

```text
RGB24: easy to understand, large, often not encoder-native
YUYV422: common USB camera format
NV12: common hardware encoder format
YUV420P: common software encoder format
MJPEG: compressed frames from camera, not H.264
```

- [ ] **Step 2: Learn conversion examples**

Conceptual conversions:

```text
RGB24 -> YUV420P -> libx264
YUYV422 -> YUV420P -> libx264
NV12 -> VideoToolbox H.264 encoder
MJPEG -> decode to raw frame -> convert -> H.264 encoder
```

Expected result: you know that "camera frame" and "encoder input frame" may be different formats.

## Stage 6: Encode H.264 from Rust

**Objective:** Move from shelling out to a pipeline toward controlling encode from Rust.

- [ ] **Step 1: Pick encoder backend**

Recommended order:

```text
1. FFmpeg CLI wrapper from Rust: easiest to verify.
2. ffmpeg-next bindings: more control, more setup.
3. GStreamer Rust bindings: good pipeline model.
4. OpenH264: focused H.264 encoder library.
5. VideoToolbox on macOS: native hardware encoding.
```

- [ ] **Step 2: Encode a known test source first**

Before camera input, encode generated frames:

```text
Generate synthetic RGB frames
Convert to YUV420P or feed through FFmpeg
Encode to H.264
Verify with ffplay
```

Expected result: separate encoder learning from camera learning.

- [ ] **Step 3: Connect camera input to encoder**

Pipeline:

```text
camera frame
  -> timestamp
  -> pixel conversion
  -> encoder input
  -> H.264 packet bytes
  -> write bytes to file
```

Expected result: `captures/rust-camera.h264` can be played by FFplay.

## Stage 7: Verify and Debug Output

**Objective:** Learn how to tell whether problems are from capture, conversion, encoding, or container/bitstream format.

- [ ] **Step 1: Verify raw bitstream**

Run:

```bash
ffprobe -hide_banner captures/rust-camera.h264
```

Expected result: FFprobe recognizes H.264 video with width and height.

- [ ] **Step 2: Convert raw H.264 to MP4**

Run:

```bash
ffmpeg -r 30 -i captures/rust-camera.h264 -c copy captures/rust-camera.mp4
```

Expected result: MP4 plays in common video players.

- [ ] **Step 3: Debug common failures**

Use this checklist:

```text
No playback: missing SPS/PPS or malformed Annex B/AVCC format.
Wrong colors: pixel format conversion problem.
Stutter: timestamp/framerate mismatch.
Huge latency: encoder B-frames, buffering, or non-low-latency settings.
File only opens in ffplay: raw .h264 has no container metadata.
```

## Stage 8: Low-Latency Streaming Concepts

**Objective:** Understand settings used for real-time capture.

- [ ] **Step 1: Learn encoder knobs**

Study:

```text
preset ultrafast
tune zerolatency
bitrate
GOP/keyframe interval
B-frames disabled
VBV buffer size
Annex B output
SPS/PPS repeat before keyframes
```

- [ ] **Step 2: Learn transport options**

Compare:

```text
Raw H.264 over TCP: simple, custom framing needed
RTP: common for real-time media
RTMP: common for ingest
WebRTC: browser-friendly, more complex
MPEG-TS: practical streaming container
```

Expected result: you know why "encode H.264" and "stream video" are related but separate problems.

## Stage 9: Optional macOS Native Path

**Objective:** Learn the production-style macOS path after the basics are clear.

- [ ] **Step 1: Study AVFoundation capture**

Conceptual pipeline:

```text
AVCaptureSession
  -> AVCaptureDeviceInput
  -> AVCaptureVideoDataOutput
  -> CMSampleBuffer
  -> CVPixelBuffer
```

- [ ] **Step 2: Study VideoToolbox encoding**

Conceptual pipeline:

```text
CVPixelBuffer
  -> VTCompressionSessionEncodeFrame
  -> encoded CMSampleBuffer
  -> extract SPS/PPS and NAL units
  -> write Annex B or AVCC
```

Expected result: you understand how macOS hardware encoding maps to the same general pipeline.

## Review Cadence

After every stage, update `LEARNING_PROGRESS.md`:

```markdown
- [x] Stage N: ...
```

Also add one short note:

```markdown
## Notes From Stage N

- What I understood:
- What confused me:
- Command or code that worked:
```

## First Next Action

Start Stage 1. Explain the pipeline in Vietnamese using concrete examples, then ask the learner to answer the four Stage 1 questions.
