import { LIMITS, StikSoundError } from "@stiksound/core";

export interface GifFrameTiming {
  frameIndex: number;
  delayMs: number;
}

export interface ScheduledVideoFrame {
  frameIndex: number;
  timestampUs: number;
  durationUs: number;
}

/**
 * Loops decoded GIF frames cleanly to cover the target audio duration (capped by the active export limit).
 */
export function scheduleGifFramesForDuration(
  frames: readonly GifFrameTiming[],
  targetDurationMs: number
): ScheduledVideoFrame[] {
  if (frames.length === 0) {
    throw new StikSoundError("unsupported-image", "Cannot schedule zero GIF frames.");
  }
  const cappedMs = Math.max(LIMITS.MIN_TRIM_DURATION_MS, Math.min(Math.round(targetDurationMs), LIMITS.DEFAULT_VIDEO_MAX_AUDIO_MS));
  const targetUs = cappedMs * 1000;
  const schedule: ScheduledVideoFrame[] = [];

  let cursorUs = 0;
  let idx = 0;
  while (cursorUs < targetUs) {
    const f = frames[idx % frames.length]!;
    const frameDurUs = Math.max(10_000, Math.round(f.delayMs * 1000));
    const remainingUs = targetUs - cursorUs;
    const clampedDurUs = Math.min(frameDurUs, remainingUs);

    schedule.push({
      frameIndex: f.frameIndex,
      timestampUs: cursorUs,
      durationUs: clampedDurUs
    });

    cursorUs += clampedDurUs;
    idx++;
  }

  return schedule;
}
