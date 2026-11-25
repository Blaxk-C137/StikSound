# Third-Party Notices

StikSound itself is dual-licensed under **MIT OR Apache-2.0** (see [LICENSE-MIT](../../LICENSE-MIT) and [LICENSE-APACHE](../../LICENSE-APACHE)).

This document records third-party software, codecs, and libraries bundled or used by StikSound. All release builds must verify these license notices before distribution.

## Desktop & Native Media Policy

- **FFmpeg (LGPL build, no `--enable-gpl`)**: Used as a dynamically linked sidecar binary for video muxing/encoding. Licensed under LGPLv2.1+.
- **Native OS Encoders**: Windows Media Foundation (`h264_mf`) and Apple VideoToolbox (`h264_videotoolbox`) use system-provided OS frameworks.
- **OpenH264 (`libopenh264`)**: BSD-2-Clause licensed software fallback for H.264.
- **libvpx (`libvpx-vp9`) & libopus**: BSD-3-Clause licensed codecs for WebM Alpha (`yuva420p` + Opus) and Ogg Opus.

## Rust Crates (`crates/*` & `apps/desktop/src-tauri`)

| Crate | License | Purpose |
| --- | --- | --- |
| `serde` / `serde_json` | MIT OR Apache-2.0 | JSON manifest serialization |
| `thiserror` | MIT OR Apache-2.0 | Structured domain error types |
| `image` | MIT OR Apache-2.0 | PNG, JPEG, WebP decoding & resampling |
| `gif` | MIT OR Apache-2.0 | Animated GIF frame decoding & transparency handling |
| `webp` | MIT OR Apache-2.0 | Static WebP encoding with alpha |
| `symphonia` | MPL-2.0 | Pure-Rust audio probing & decoding (MP3, WAV, OGG, AAC) |
| `zip` | MIT | Portable `.stiksound.zip` reading/writing |
| `tauri` | MIT OR Apache-2.0 | Desktop application shell |
| `reqwest` | MIT OR Apache-2.0 | Allowlisted HTTPS client for MyInstants |
| `url` | MIT OR Apache-2.0 | Strict URL parsing and validation |

## TypeScript & Browser Packages (`packages/*` & `apps/extension`)

| Package | License | Purpose |
| --- | --- | --- |
| `react` / `react-dom` | MIT | Shared UI components |
| `ajv` / `ajv-formats` | MIT | JSON Schema Draft 2020-12 manifest validation |
| `fflate` | MIT | Pure-JS ZIP compression/decompression |
| `mp4-muxer` | MIT | Pure-TS ISO-BMFF MP4 muxing for WebCodecs |
| `webm-muxer` | MIT | Pure-TS WebM muxing for WebCodecs VP9/Opus |
| `vitest` | MIT | Unit & integration test runner |

## Content Rights Notice

Sounds fetched from MyInstants remain the property of their respective owners. StikSound does not grant any redistribution license for downloaded sounds.
