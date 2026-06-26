# Learn Camera Capture to H.264 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Teach camera capture, H.264 bitstream structure, and H.264 packetization step by step, ending with Rust prototypes that can inspect raw H.264 and prepare H.264 NAL units for RTP transport.

**Architecture:** Learn the pipeline in layers: first observe it with FFmpeg CLI, then inspect the H.264 output, then understand containers and RTP packetization, then implement capture and encoding internals in Rust. Keep each milestone independently testable so the learner can verify understanding before moving on.

**Tech Stack:** Rust, FFmpeg CLI, H.264/AVC concepts, RTP packetization concepts, camera capture APIs, optional macOS AVFoundation and VideoToolbox later.

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

## Stage 4: Build a Rust H.264 Inspector

**Objective:** Move from inspecting H.264 with CLI tools to reading H.264 stream structure from Rust.

- [ ] **Step 1: Parse Annex B start codes**

Implement helpers that recognize both Annex B start code forms:

```text
00 00 00 01
00 00 01
```

Expected result: Rust can detect NAL unit boundaries in a raw `.h264` file.

- [ ] **Step 2: Extract NAL headers and NAL types**

For each NAL unit, read the first byte after the start code:

```text
nal_type = nal_header & 0x1f
```

Learn common NAL types:

```text
1: non-IDR slice
5: IDR slice
6: SEI
7: SPS
8: PPS
```

Expected result: Rust can print NAL unit type summaries.

- [ ] **Step 3: Understand slice vs frame**

Study the fact that one frame can contain multiple slices:

```text
IDR slice count != keyframe count
non-IDR slice count != frame count
```

Expected result: the learner can explain why 10 IDR slices may represent one IDR frame.

- [ ] **Step 4: Summarize GOPs**

Group consecutive IDR slices and following non-IDR slices into simple GOP summaries:

```text
GOP 0: IDR slices: 10, non-IDR slices: 2490
GOP 1: IDR slices: 10, non-IDR slices: 2490
```

Expected result: the learner can relate GOP size, keyframe interval, framerate, and low-latency settings.

- [ ] **Step 5: Experiment with keyframe interval**

Add x264 GOP controls:

```text
-g 30
-keyint_min 30
-sc_threshold 0
```

Expected result: the learner observes GOPs shrink from about 250 frames to about 30 frames at 30fps.

## Stage 5: Raw H.264 Stream vs Container

**Objective:** Understand that raw `.h264` is an elementary stream, while MP4 is a container with timing, metadata, and different H.264 framing.

- [ ] **Step 1: Compare raw H.264 and MP4 with FFprobe**

Remux raw H.264 to MP4:

```bash
ffmpeg -framerate 30 -i captures/rust-camera-g30.h264 -c copy captures/rust-camera-g30.mp4
```

Then compare:

```bash
ffprobe -hide_banner captures/rust-camera-g30.h264
ffprobe -hide_banner captures/rust-camera-g30.mp4
```

Expected result: raw `.h264` has limited timing/container metadata, while MP4 has duration, track metadata, bitrate, and a timeline.

- [ ] **Step 2: Learn Annex B vs AVCC**

Compare first packet bytes:

```text
Annex B raw .h264:
00 00 00 01 67 ...

AVCC inside MP4:
00 00 00 16 67 ...
```

Expected result: the learner can explain that Annex B uses start codes, while AVCC uses length prefixes.

- [ ] **Step 3: Learn mux, demux, and remux**

Definitions:

```text
encode: raw frames -> compressed bitstream
mux: one or more streams -> container
demux: container -> elementary streams
remux: change container/framing without re-encoding
```

Expected result: the learner can explain why `-c copy` remuxes instead of re-encoding.

## Stage 6: H.264 over RTP Concepts

**Objective:** Understand how H.264 NAL units are packetized for RTP transport.

- [ ] **Step 1: Learn the H.264-to-RTP pipeline**

Study this pipeline:

```text
H.264 Annex B or AVCC stream
  -> extract NAL units
  -> remove Annex B start codes or AVCC length prefixes
  -> RTP H.264 payloader
  -> RTP packets over UDP
```

Expected result: the learner can explain why RTP packets contain NAL payload bytes, not Annex B start codes.

- [ ] **Step 2: Learn RTP packet fields**

Important RTP header fields:

```text
payload type
sequence number
timestamp
marker bit
SSRC
payload bytes
```

Expected result: the learner understands the difference between H.264 payload bytes and RTP metadata.

- [ ] **Step 3: Learn H.264 RTP packetization modes**

Focus first on:

```text
Single NAL Unit Packet: one small NAL in one RTP packet
FU-A: one large NAL fragmented across multiple RTP packets
```

Learn later:

```text
STAP-A: multiple small NAL units aggregated into one RTP packet
```

Expected result: the learner can explain when a NAL can be sent directly and when it must be fragmented.

## Stage 7: Extract Full NAL Units for RTP Payloading

**Objective:** Upgrade the Rust inspector from "NAL headers only" to full NAL unit byte ranges.

- [ ] **Step 1: Extract Annex B NAL unit bytes**

Implement:

```rust
fn find_nal_units_annex_b(bytes: &[u8]) -> Vec<&[u8]>
```

Each returned NAL unit should start with the NAL header byte:

```text
0x67 ... = SPS NAL unit bytes
0x68 ... = PPS NAL unit bytes
0x65 ... = IDR slice NAL unit bytes
```

Do not include Annex B start codes in returned slices.

Expected result: Rust can report NAL type and NAL payload length.

- [ ] **Step 2: Compare extracted NAL lengths with MP4 AVCC length prefixes**

Use the observed MP4 length prefix, such as:

```text
00 00 00 16 67 ...
```

Expected result: the learner understands that RTP payloading starts from NAL unit bytes, regardless of whether source framing was Annex B or AVCC.

## Stage 8: Packetize H.264 NAL Units into RTP Packets

**Objective:** Implement a learning-oriented RTP H.264 payloader for Single NAL and FU-A packets.

- [ ] **Step 1: Define a simple RTP packet model**

Fields:

```text
sequence_number
timestamp
marker
payload_type
ssrc
payload
```

Expected result: Rust can construct inspectable RTP packet structs without sending network traffic yet.

- [ ] **Step 2: Implement Single NAL Unit packetization**

If a NAL unit is small enough for the chosen MTU, put the whole NAL unit into one RTP payload.

Expected result: SPS/PPS and small NAL units can become one RTP packet each.

- [ ] **Step 3: Implement FU-A fragmentation**

If a NAL unit is too large, fragment it into FU-A packets.

Learn FU-A fields:

```text
FU indicator
FU header
Start bit
End bit
original NAL type
```

Expected result: large IDR/non-IDR slices can be split across multiple RTP packets.

- [ ] **Step 4: Verify packetization with printed summaries**

Print examples:

```text
NAL type 7 SPS -> Single RTP packet
NAL type 5 IDR slice, 50350 bytes -> FU-A packets 1..N
```

Expected result: the learner can explain how H.264 byte boundaries map to RTP packet boundaries.

## Stage 9: Low-Latency Streaming Concepts

**Objective:** Understand settings used for real-time capture and packet transport.

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
packet MTU
RTP timestamp clock rate
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

Expected result: you know why "encode H.264", "packetize H.264", and "stream video" are related but separate problems.

## Stage 10: Optional Capture and Encoding Internals

**Objective:** Return to camera frames, pixel conversion, and lower-level encoders after the H.264/RTP path is clear.

### Stage 10A: Capture Frames in Rust

**Objective:** Write a Rust program that opens the camera and receives frames.

Recommended first path:

```text
Use FFmpeg or GStreamer from Rust if the goal is to learn the full pipeline quickly.
Use a Rust camera crate or native API later if the goal is low-level control.
```

Expected result: the learner understands the difference between wrapping FFmpeg and receiving raw camera frames in Rust.

### Stage 10B: Pixel Format Conversion

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

### Stage 10C: Encode H.264 from Rust

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

### Stage 10D: Verify and Debug Output

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

### Stage 10E: Optional macOS Native Path

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
