import {
  LIMITS,
  StikSoundError,
  type AudioProbeInfo,
  type ImageProbeInfo,
  type VideoFormat
} from "@stiksound/core";

function readU16LE(b: Uint8Array, o: number): number {
  return (b[o] ?? 0) | ((b[o + 1] ?? 0) << 8);
}

function readU16BE(b: Uint8Array, o: number): number {
  return ((b[o] ?? 0) << 8) | (b[o + 1] ?? 0);
}

function readU32LE(b: Uint8Array, o: number): number {
  return ((b[o] ?? 0) | ((b[o + 1] ?? 0) << 8) | ((b[o + 2] ?? 0) << 16) | ((b[o + 3] ?? 0) << 24)) >>> 0;
}

function readU32BE(b: Uint8Array, o: number): number {
  return (((b[o] ?? 0) << 24) | ((b[o + 1] ?? 0) << 16) | ((b[o + 2] ?? 0) << 8) | (b[o + 3] ?? 0)) >>> 0;
}

function ascii(b: Uint8Array, start: number, len: number): string {
  let s = "";
  for (let i = 0; i < len && start + i < b.length; i++) {
    s += String.fromCharCode(b[start + i]!);
  }
  return s;
}

function assertDimensionsSafe(width: number, height: number): void {
  if (width <= 0 || height <= 0) {
    throw new StikSoundError("unsupported-image", "Image has zero dimensions.");
  }
  if (width > LIMITS.MAX_IMAGE_DIMENSION || height > LIMITS.MAX_IMAGE_DIMENSION) {
    throw new StikSoundError("file-too-large", `Image dimension ${width}x${height} exceeds 8000 px.`);
  }
  if (width * height > LIMITS.MAX_IMAGE_PIXELS) {
    throw new StikSoundError("file-too-large", `Image exceeds 40 megapixels (${width}x${height}).`);
  }
}

export function sniffImageHeader(bytes: Uint8Array): ImageProbeInfo {
  if (bytes.byteLength < 12) {
    throw new StikSoundError("unsupported-image", "Image buffer too small.");
  }
  if (bytes.byteLength > LIMITS.MAX_IMAGE_BYTES) {
    throw new StikSoundError("file-too-large", "Image exceeds 20 MB limit.");
  }

  // PNG
  if (
    bytes[0] === 0x89 &&
    bytes[1] === 0x50 &&
    bytes[2] === 0x4e &&
    bytes[3] === 0x47 &&
    bytes[4] === 0x0d &&
    bytes[5] === 0x0a &&
    bytes[6] === 0x1a &&
    bytes[7] === 0x0a
  ) {
    if (bytes.byteLength < 29 || ascii(bytes, 12, 4) !== "IHDR") {
      throw new StikSoundError("unsupported-image", "Malformed PNG IHDR chunk.");
    }
    const width = readU32BE(bytes, 16);
    const height = readU32BE(bytes, 20);
    assertDimensionsSafe(width, height);
    const colorType = bytes[25] ?? 0;
    let hasAlpha = colorType === 4 || colorType === 6;

    // Scan chunks briefly for tRNS transparency
    let pos = 8;
    while (pos + 12 <= bytes.byteLength && pos < 65536) {
      const len = readU32BE(bytes, pos);
      const tag = ascii(bytes, pos + 4, 4);
      if (tag === "tRNS") {
        hasAlpha = true;
        break;
      }
      if (tag === "IDAT" || tag === "IEND") break;
      pos += 12 + len;
    }

    return {
      format: "png",
      width,
      height,
      has_alpha: hasAlpha,
      animated: false,
      frame_count: 1,
      duration_ms: 0
    };
  }

  // JPEG
  if (bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[2] === 0xff) {
    let offset = 2;
    while (offset + 4 <= bytes.byteLength) {
      if (bytes[offset] !== 0xff) break;
      const marker = bytes[offset + 1]!;
      if (marker === 0xd9 || marker === 0xda) break;
      if (marker === 0x00 || (marker >= 0xd0 && marker <= 0xd8)) {
        offset += 2;
        continue;
      }
      const segLen = readU16BE(bytes, offset + 2);
      if (segLen < 2 || offset + 2 + segLen > bytes.byteLength) break;
      if ((marker >= 0xc0 && marker <= 0xc3) || (marker >= 0xc5 && marker <= 0xc7)) {
        const height = readU16BE(bytes, offset + 5);
        const width = readU16BE(bytes, offset + 7);
        assertDimensionsSafe(width, height);
        return {
          format: "jpeg",
          width,
          height,
          has_alpha: false,
          animated: false,
          frame_count: 1,
          duration_ms: 0
        };
      }
      offset += 2 + segLen;
    }
    throw new StikSoundError("unsupported-image", "Truncated or invalid JPEG markers.");
  }

  // WebP
  if (ascii(bytes, 0, 4) === "RIFF" && ascii(bytes, 8, 4) === "WEBP") {
    if (bytes.byteLength < 20) {
      throw new StikSoundError("unsupported-image", "Truncated WebP.");
    }
    const chunkTag = ascii(bytes, 12, 4);
    if (chunkTag === "VP8X" && bytes.byteLength >= 30) {
      const flags = bytes[20] ?? 0;
      const hasAlpha = (flags & 0x10) !== 0;
      const animated = (flags & 0x02) !== 0;
      const width = 1 + ((bytes[24]! | (bytes[25]! << 8) | (bytes[26]! << 16)) >>> 0);
      const height = 1 + ((bytes[27]! | (bytes[28]! << 8) | (bytes[29]! << 16)) >>> 0);
      assertDimensionsSafe(width, height);
      return {
        format: "webp",
        width,
        height,
        has_alpha: hasAlpha,
        animated,
        frame_count: animated ? 2 : 1,
        duration_ms: 0
      };
    }
    if (chunkTag === "VP8L" && bytes.byteLength >= 25 && bytes[20] === 0x2f) {
      const bits = readU32LE(bytes, 21);
      const width = (bits & 0x3fff) + 1;
      const height = ((bits >> 14) & 0x3fff) + 1;
      const hasAlpha = ((bits >> 28) & 1) === 1;
      assertDimensionsSafe(width, height);
      return {
        format: "webp",
        width,
        height,
        has_alpha: hasAlpha,
        animated: false,
        frame_count: 1,
        duration_ms: 0
      };
    }
    if (chunkTag === "VP8 " && bytes.byteLength >= 30) {
      const width = readU16LE(bytes, 26) & 0x3fff;
      const height = readU16LE(bytes, 28) & 0x3fff;
      assertDimensionsSafe(width, height);
      return {
        format: "webp",
        width,
        height,
        has_alpha: false,
        animated: false,
        frame_count: 1,
        duration_ms: 0
      };
    }
    throw new StikSoundError("unsupported-image", "Unsupported WebP chunk layout.");
  }

  // GIF87a / GIF89a
  const gifSig = ascii(bytes, 0, 6);
  if (gifSig === "GIF87a" || gifSig === "GIF89a") {
    if (bytes.byteLength < 14) {
      throw new StikSoundError("unsupported-image", "Truncated GIF header.");
    }
    const width = readU16LE(bytes, 6);
    const height = readU16LE(bytes, 8);
    assertDimensionsSafe(width, height);

    const packed = bytes[10] ?? 0;
    let pos = 13;
    if ((packed & 0x80) !== 0) {
      const gctSize = 3 * (1 << ((packed & 0x07) + 1));
      pos += gctSize;
    }

    let frameCount = 0;
    let durationMs = 0;
    let hasAlpha = false;
    let pendingDelayMs = 100;

    while (pos < bytes.byteLength) {
      const block = bytes[pos++];
      if (block === 0x3b) break; // Trailer
      if (block === 0x21) {
        // Extension
        const label = bytes[pos++];
        if (label === 0xf9 && pos + 5 < bytes.byteLength) {
          const blockSize = bytes[pos]!;
          const gcePacked = bytes[pos + 1]!;
          if ((gcePacked & 0x01) !== 0) hasAlpha = true;
          const delayCs = readU16LE(bytes, pos + 2);
          pendingDelayMs = (delayCs === 0 ? 10 : delayCs) * 10;
          pos += 1 + blockSize;
          // Skip sub-block terminator
          while (pos < bytes.byteLength && bytes[pos]! > 0) {
            pos += 1 + bytes[pos]!;
          }
          pos++;
        } else {
          // Other extension sub-blocks
          while (pos < bytes.byteLength) {
            const subLen = bytes[pos++]!;
            if (subLen === 0) break;
            pos += subLen;
          }
        }
      } else if (block === 0x2c) {
        // Image descriptor
        if (pos + 9 > bytes.byteLength) break;
        const imgPacked = bytes[pos + 8]!;
        pos += 9;
        if ((imgPacked & 0x80) !== 0) {
          const lctSize = 3 * (1 << ((imgPacked & 0x07) + 1));
          pos += lctSize;
        }
        pos++; // LZW min code size
        while (pos < bytes.byteLength) {
          const subLen = bytes[pos++]!;
          if (subLen === 0) break;
          pos += subLen;
        }
        frameCount++;
        durationMs += pendingDelayMs;
        pendingDelayMs = 100;
      } else {
        break;
      }
    }

    if (frameCount === 0) {
      throw new StikSoundError("unsupported-image", "GIF contains no frames.");
    }

    return {
      format: "gif",
      width,
      height,
      has_alpha: hasAlpha,
      animated: frameCount > 1,
      frame_count: frameCount,
      duration_ms: frameCount > 1 ? durationMs : 0
    };
  }

  throw new StikSoundError("unsupported-image", "Unrecognized image format.");
}

export function sniffAudioContainer(bytes: Uint8Array): AudioProbeInfo {
  if (bytes.byteLength < 12) {
    throw new StikSoundError("unsupported-audio", "Audio buffer too small.");
  }
  if (bytes.byteLength > LIMITS.MAX_AUDIO_BYTES) {
    throw new StikSoundError("file-too-large", "Audio exceeds 25 MB limit.");
  }

  // WAV PCM
  if (ascii(bytes, 0, 4) === "RIFF" && ascii(bytes, 8, 4) === "WAVE") {
    let pos = 12;
    let channels = 1;
    let sampleRate = 48000;
    let bitsPerSample = 16;
    let dataSize = 0;

    while (pos + 8 <= bytes.byteLength) {
      const chunkId = ascii(bytes, pos, 4);
      const chunkSize = readU32LE(bytes, pos + 4);
      if (chunkId === "fmt " && pos + 24 <= bytes.byteLength) {
        channels = readU16LE(bytes, pos + 10);
        sampleRate = readU32LE(bytes, pos + 12);
        bitsPerSample = readU16LE(bytes, pos + 22);
      } else if (chunkId === "data") {
        dataSize = chunkSize;
        break;
      }
      pos += 8 + chunkSize + (chunkSize % 2);
    }

    const bytesPerSec = Math.max(1, sampleRate * Math.max(1, channels) * Math.max(1, bitsPerSample / 8));
    const durationMs = Math.round((dataSize / bytesPerSec) * 1000);
    if (durationMs < LIMITS.MIN_AUDIO_DURATION_MS) {
      throw new StikSoundError("unsupported-audio", "WAV audio shorter than 100 ms or corrupt.");
    }
    return {
      format: "wav",
      duration_ms: durationMs,
      sample_rate: sampleRate,
      channels: Math.max(1, channels)
    };
  }

  if (ascii(bytes, 0, 4) === "OggS") {
    return { format: "ogg", duration_ms: 1000, sample_rate: 48000, channels: 1 };
  }

  if (ascii(bytes, 0, 3) === "ID3" || (bytes[0] === 0xff && (bytes[1]! & 0xe0) === 0xe0)) {
    return { format: "mp3", duration_ms: 1000, sample_rate: 44100, channels: 2 };
  }

  if (ascii(bytes, 4, 4) === "ftyp") {
    return { format: "m4a", duration_ms: 1000, sample_rate: 44100, channels: 2 };
  }

  if (bytes[0] === 0xff && (bytes[1]! & 0xf6) === 0xf0) {
    return { format: "aac", duration_ms: 1000, sample_rate: 44100, channels: 2 };
  }

  throw new StikSoundError("unsupported-audio", "Unrecognized audio container.");
}

/**
 * Recognise short video clips by their container signature (no decoding).
 * MP4 / MOV: an ISO-BMFF "ftyp" box ("qt  " brand = QuickTime) or a bare QuickTime atom.
 * WebM / Matroska: EBML magic 1A 45 DF A3.
 * Audio-only MP4s ("M4A ") are excluded so they keep going through the audio path.
 */
export function sniffVideoContainer(bytes: Uint8Array): VideoFormat | null {
  if (bytes.byteLength < 12) return null;
  const box = ascii(bytes, 4, 4);
  if (box === "ftyp") {
    const brand = ascii(bytes, 8, 4);
    if (brand === "M4A " || brand === "M4B " || brand === "M4P ") return null;
    return brand === "qt  " ? "mov" : "mp4";
  }
  if (box === "moov" || box === "mdat" || box === "wide" || box === "free") return "mov";
  if (bytes[0] === 0x1a && bytes[1] === 0x45 && bytes[2] === 0xdf && bytes[3] === 0xa3) return "webm";
  return null;
}
