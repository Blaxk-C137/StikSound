use std::io::Cursor;

use gif::{DecodeOptions, DisposalMethod};
use image::imageops::FilterType;
use image::{DynamicImage, ImageDecoder, ImageFormat as ImgFmt, ImageReader, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;
pub const MAX_IMAGE_DIMENSION: u32 = 8000;
pub const MAX_IMAGE_PIXELS: u64 = 40_000_000;
pub const STICKER_SIZE_PX: u32 = 512;
pub const MAX_STATIC_WEBP_BYTES: usize = 102_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StickerImageFormat {
    Png,
    Jpeg,
    Webp,
    Gif,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageProbeInfo {
    pub format: StickerImageFormat,
    pub width: u32,
    pub height: u32,
    pub has_alpha: bool,
    pub animated: bool,
    pub frame_count: usize,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GifFrame512 {
    pub delay_ms: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum ImageEngineError {
    #[error("unsupported-image: {0}")]
    UnsupportedImage(String),
    #[error("file-too-large: {0}")]
    FileTooLarge(String),
    #[error("sticker-too-large: {0}")]
    StickerTooLarge(String),
}

pub fn sniff_image_format(bytes: &[u8]) -> Result<StickerImageFormat, ImageEngineError> {
    if bytes.len() < 12 {
        return Err(ImageEngineError::UnsupportedImage(
            "Image header too short".to_string(),
        ));
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Ok(StickerImageFormat::Png);
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(StickerImageFormat::Jpeg);
    }
    if bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Ok(StickerImageFormat::Webp);
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Ok(StickerImageFormat::Gif);
    }
    Err(ImageEngineError::UnsupportedImage(
        "Unrecognized image magic bytes".to_string(),
    ))
}

fn check_dimension_limits(width: u32, height: u32) -> Result<(), ImageEngineError> {
    if width == 0 || height == 0 {
        return Err(ImageEngineError::UnsupportedImage(
            "Image has zero dimension".to_string(),
        ));
    }
    if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
        return Err(ImageEngineError::FileTooLarge(format!(
            "Image dimension {width}x{height} exceeds {MAX_IMAGE_DIMENSION}px limit"
        )));
    }
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_IMAGE_PIXELS {
        return Err(ImageEngineError::FileTooLarge(format!(
            "Image pixel count {pixels} exceeds {MAX_IMAGE_PIXELS} limit"
        )));
    }
    Ok(())
}

fn webp_is_animated(bytes: &[u8]) -> bool {
    if bytes.len() >= 21 && &bytes[12..16] == b"VP8X" {
        let flags = bytes[20];
        return (flags & 0x02) != 0;
    }
    false
}

fn inspect_gif(bytes: &[u8]) -> Result<(u32, u32, bool, usize, u32), ImageEngineError> {
    let mut opts = DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = opts
        .read_info(Cursor::new(bytes))
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Invalid GIF header: {e}")))?;

    let width = u32::from(decoder.width());
    let height = u32::from(decoder.height());
    check_dimension_limits(width, height)?;

    let mut frame_count = 0usize;
    let mut total_ms = 0u32;
    let mut has_alpha = false;

    while let Some(frame) = decoder
        .read_next_frame()
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Corrupt GIF frame: {e}")))?
    {
        frame_count += 1;
        // GIF delay is in 10ms units; standard browser/player floor for 0 is 100ms (10cs)
        let delay_cs = if frame.delay == 0 { 10 } else { frame.delay };
        total_ms = total_ms.saturating_add(u32::from(delay_cs) * 10);
        if !has_alpha && frame.buffer.chunks_exact(4).any(|px| px[3] < 255) {
            has_alpha = true;
        }
    }

    if frame_count == 0 {
        return Err(ImageEngineError::UnsupportedImage(
            "GIF contains zero frames".to_string(),
        ));
    }

    Ok((width, height, has_alpha, frame_count, total_ms))
}

pub fn probe_image(bytes: &[u8]) -> Result<ImageProbeInfo, ImageEngineError> {
    if bytes.is_empty() {
        return Err(ImageEngineError::UnsupportedImage("Empty image".to_string()));
    }
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(ImageEngineError::FileTooLarge(
            "Image exceeds 20 MB limit".to_string(),
        ));
    }

    let format = sniff_image_format(bytes)?;
    if format == StickerImageFormat::Png && bytes.len() >= 24 && &bytes[12..16] == b"IHDR" {
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        check_dimension_limits(w, h)?;
    }
    if format == StickerImageFormat::Gif {
        let (width, height, has_alpha, frame_count, duration_ms) = inspect_gif(bytes)?;
        return Ok(ImageProbeInfo {
            format,
            width,
            height,
            has_alpha,
            animated: frame_count > 1,
            frame_count,
            duration_ms: if frame_count > 1 { duration_ms } else { 0 },
        });
    }

    let img_fmt = match format {
        StickerImageFormat::Png => ImgFmt::Png,
        StickerImageFormat::Jpeg => ImgFmt::Jpeg,
        StickerImageFormat::Webp => ImgFmt::WebP,
        StickerImageFormat::Gif => unreachable!(),
    };

    let reader = ImageReader::with_format(Cursor::new(bytes), img_fmt);
    let decoder = reader
        .into_decoder()
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Decoder init failed: {e}")))?;

    let (width, height) = decoder.dimensions();
    check_dimension_limits(width, height)?;

    let color_type = decoder.color_type();
    let img = DynamicImage::from_decoder(decoder)
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Decode failed: {e}")))?;
    let rgba = img.to_rgba8();
    let has_alpha = color_type.has_alpha() || rgba.pixels().any(|p| p[3] < 255);
    let animated = format == StickerImageFormat::Webp && webp_is_animated(bytes);

    Ok(ImageProbeInfo {
        format,
        width,
        height,
        has_alpha,
        animated,
        frame_count: 1,
        duration_ms: 0,
    })
}

/// Computes a 1:1 square crop anchored at normalized (crop_x, crop_y) in [0.0, 1.0].
/// Output resolution is determined solely by the input's smaller side, capped at 512x512.
pub fn compute_square_crop(src_w: u32, src_h: u32, crop_x: f32, crop_y: f32) -> (u32, u32, u32, u32) {
    let crop_side = src_w.min(src_h).max(1);
    let raw_out = crop_side.min(STICKER_SIZE_PX);
    let output_size = if raw_out <= 2 { raw_out } else { raw_out - (raw_out % 2) };
    let nx = if crop_x.is_finite() { crop_x.clamp(0.0, 1.0) } else { 0.5 };
    let ny = if crop_y.is_finite() { crop_y.clamp(0.0, 1.0) } else { 0.5 };
    let src_x = ((src_w.saturating_sub(crop_side)) as f32 * nx).round() as u32;
    let src_y = ((src_h.saturating_sub(crop_side)) as f32 * ny).round() as u32;
    (src_x, src_y, crop_side, output_size)
}

/// Crops a 1:1 square from `src` at `(crop_x, crop_y)` and scales to `output_size x output_size` (<= 512).
pub fn crop_square_rgba_1x1(src: &RgbaImage, crop_x: f32, crop_y: f32) -> RgbaImage {
    let (w, h) = src.dimensions();
    let (src_x, src_y, crop_side, output_size) = compute_square_crop(w, h, crop_x, crop_y);
    let cropped = image::imageops::crop_imm(src, src_x, src_y, crop_side, crop_side).to_image();
    if crop_side == output_size {
        cropped
    } else {
        image::imageops::resize(&cropped, output_size, output_size, FilterType::Lanczos3)
    }
}

/// Scales an RGBA image proportionally so its longest side is <= 512 and centers it
/// on a 512x512 transparent RGBA canvas without premultiplication fringing.
pub fn contain_fit_rgba_512(src: &RgbaImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let scale = (STICKER_SIZE_PX as f64 / w as f64).min(STICKER_SIZE_PX as f64 / h as f64);
    let target_w = ((w as f64 * scale).round() as u32).clamp(1, STICKER_SIZE_PX);
    let target_h = ((h as f64 * scale).round() as u32).clamp(1, STICKER_SIZE_PX);

    let resized = if target_w == w && target_h == h {
        src.clone()
    } else {
        image::imageops::resize(src, target_w, target_h, FilterType::Lanczos3)
    };

    let mut canvas = RgbaImage::from_pixel(STICKER_SIZE_PX, STICKER_SIZE_PX, Rgba([0, 0, 0, 0]));
    let offset_x = (STICKER_SIZE_PX - target_w) / 2;
    let offset_y = (STICKER_SIZE_PX - target_h) / 2;
    image::imageops::overlay(&mut canvas, &resized, i64::from(offset_x), i64::from(offset_y));
    canvas
}

pub fn render_sticker_512(bytes: &[u8]) -> Result<RgbaImage, ImageEngineError> {
    let info = probe_image(bytes)?;
    if info.format == StickerImageFormat::Gif {
        let frames = decode_gif_frames_512(bytes)?;
        let first = frames
            .into_iter()
            .next()
            .ok_or_else(|| ImageEngineError::UnsupportedImage("No GIF frames".to_string()))?;
        return RgbaImage::from_raw(STICKER_SIZE_PX, STICKER_SIZE_PX, first.rgba)
            .ok_or_else(|| ImageEngineError::UnsupportedImage("Invalid RGBA frame".to_string()));
    }

    let img_fmt = match info.format {
        StickerImageFormat::Png => ImgFmt::Png,
        StickerImageFormat::Jpeg => ImgFmt::Jpeg,
        StickerImageFormat::Webp => ImgFmt::WebP,
        StickerImageFormat::Gif => unreachable!(),
    };
    let decoded = ImageReader::with_format(Cursor::new(bytes), img_fmt)
        .decode()
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Decode failed: {e}")))?;
    Ok(contain_fit_rgba_512(&decoded.to_rgba8()))
}

/// Decodes all GIF frames with disposal methods applied so transparency is preserved
/// across frames, and contain-fits every frame onto 512x512 RGBA.
pub fn decode_gif_frames_512(bytes: &[u8]) -> Result<Vec<GifFrame512>, ImageEngineError> {
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(ImageEngineError::FileTooLarge("GIF exceeds 20 MB limit".to_string()));
    }
    let mut opts = DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::RGBA);
    let mut decoder = opts
        .read_info(Cursor::new(bytes))
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Invalid GIF: {e}")))?;

    let canvas_w = u32::from(decoder.width());
    let canvas_h = u32::from(decoder.height());
    check_dimension_limits(canvas_w, canvas_h)?;

    let mut current_canvas = RgbaImage::from_pixel(canvas_w, canvas_h, Rgba([0, 0, 0, 0]));
    let mut out = Vec::new();

    while let Some(frame) = decoder
        .read_next_frame()
        .map_err(|e| ImageEngineError::UnsupportedImage(format!("Corrupt GIF frame: {e}")))?
    {
        let prev_canvas = if frame.dispose == DisposalMethod::Previous {
            Some(current_canvas.clone())
        } else {
            None
        };

        let fx = u32::from(frame.left);
        let fy = u32::from(frame.top);
        let fw = u32::from(frame.width);
        let fh = u32::from(frame.height);

        for y in 0..fh {
            for x in 0..fw {
                let cx = fx + x;
                let cy = fy + y;
                if cx < canvas_w && cy < canvas_h {
                    let idx = ((y * fw + x) * 4) as usize;
                    if idx + 3 < frame.buffer.len() {
                        let a = frame.buffer[idx + 3];
                        if a > 0 {
                            current_canvas.put_pixel(
                                cx,
                                cy,
                                Rgba([
                                    frame.buffer[idx],
                                    frame.buffer[idx + 1],
                                    frame.buffer[idx + 2],
                                    a,
                                ]),
                            );
                        }
                    }
                }
            }
        }

        let fitted = contain_fit_rgba_512(&current_canvas);
        let delay_cs = if frame.delay == 0 { 10 } else { frame.delay };
        out.push(GifFrame512 {
            delay_ms: u32::from(delay_cs) * 10,
            rgba: fitted.into_raw(),
        });

        match frame.dispose {
            DisposalMethod::Background => {
                for y in 0..fh {
                    for x in 0..fw {
                        let cx = fx + x;
                        let cy = fy + y;
                        if cx < canvas_w && cy < canvas_h {
                            current_canvas.put_pixel(cx, cy, Rgba([0, 0, 0, 0]));
                        }
                    }
                }
            }
            DisposalMethod::Previous => {
                if let Some(prev) = prev_canvas {
                    current_canvas = prev;
                }
            }
            DisposalMethod::Any | DisposalMethod::Keep => {}
        }
    }

    if out.is_empty() {
        return Err(ImageEngineError::UnsupportedImage(
            "GIF contained zero frames".to_string(),
        ));
    }
    Ok(out)
}

/// Straight-alpha composite onto a solid RGB matte color (used when producing H.264 MP4).
pub fn composite_on_matte(rgba: &RgbaImage, matte_rgb: [u8; 3]) -> RgbaImage {
    let mut out = RgbaImage::new(rgba.width(), rgba.height());
    for (x, y, px) in rgba.enumerate_pixels() {
        let a = f32::from(px[3]) / 255.0;
        let inv = 1.0 - a;
        let r = (f32::from(px[0]) * a + f32::from(matte_rgb[0]) * inv).round() as u8;
        let g = (f32::from(px[1]) * a + f32::from(matte_rgb[1]) * inv).round() as u8;
        let b = (f32::from(px[2]) * a + f32::from(matte_rgb[2]) * inv).round() as u8;
        out.put_pixel(x, y, Rgba([r, g, b, 255]));
    }
    out
}

pub fn parse_hex_rgb(hex: &str) -> [u8; 3] {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&clean[0..2], 16),
            u8::from_str_radix(&clean[2..4], 16),
            u8::from_str_radix(&clean[4..6], 16),
        ) {
            return [r, g, b];
        }
    }
    [255, 255, 255]
}

fn quantize_rgba_step(img: &RgbaImage, step: u8) -> RgbaImage {
    if step <= 1 {
        return img.clone();
    }
    let mut out = img.clone();
    for px in out.pixels_mut() {
        if px[3] == 0 {
            px.0 = [0, 0, 0, 0];
            continue;
        }
        px[0] = (px[0] / step) * step;
        px[1] = (px[1] / step) * step;
        px[2] = (px[2] / step) * step;
        px[3] = (px[3] / step) * step;
    }
    out
}

/// Encodes a 512x512 RGBA image to WebP preserving alpha and staying <= 100 KB.
/// Tries lossless first, then steps down palette/channel quantization until <= 102,400 bytes.
pub fn encode_webp_under_100kb(frame: &RgbaImage) -> Result<Vec<u8>, ImageEngineError> {
    if frame.width() != STICKER_SIZE_PX || frame.height() != STICKER_SIZE_PX {
        return Err(ImageEngineError::UnsupportedImage(
            "WebP frame must be 512x512".to_string(),
        ));
    }

    for step in [1u8, 4, 8, 16, 32] {
        let candidate = quantize_rgba_step(frame, step);
        let mut buf = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(candidate)
            .write_to(&mut buf, ImgFmt::WebP)
            .map_err(|e| ImageEngineError::UnsupportedImage(format!("WebP encode failed: {e}")))?;
        let bytes = buf.into_inner();
        if bytes.len() <= MAX_STATIC_WEBP_BYTES {
            return Ok(bytes);
        }
    }

    Err(ImageEngineError::StickerTooLarge(
        "Could not compress WebP under 100 KB".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gif::{Encoder, Frame, Repeat};

    fn make_transparent_png(w: u32, h: u32) -> Vec<u8> {
        let mut img = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
        for y in (h / 4)..(3 * h / 4) {
            for x in (w / 4)..(3 * w / 4) {
                img.put_pixel(x, y, Rgba([255, 64, 32, 255]));
            }
        }
        let mut buf = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(img)
            .write_to(&mut buf, ImgFmt::Png)
            .unwrap();
        buf.into_inner()
    }

    fn make_animated_transparent_gif() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let palette = &[0, 0, 0, 255, 0, 0, 0, 255, 0];
            let mut encoder = Encoder::new(&mut buf, 64, 64, palette).unwrap();
            encoder.set_repeat(Repeat::Infinite).unwrap();
            for idx in 0..2 {
                let mut pixels = vec![0u8; 64 * 64];
                for y in 16..48 {
                    for x in 16..48 {
                        pixels[y * 64 + x] = (idx + 1) as u8;
                    }
                }
                let mut frame = Frame::from_indexed_pixels(64, 64, pixels, Some(0));
                frame.delay = 50; // 500ms
                frame.dispose = DisposalMethod::Background;
                encoder.write_frame(&frame).unwrap();
            }
        }
        buf
    }

    #[test]
    fn tc001_png_preserves_alpha_and_encodes_webp_under_100kb() {
        let png = make_transparent_png(640, 480);
        let probe = probe_image(&png).unwrap();
        assert_eq!(probe.format, StickerImageFormat::Png);
        assert!(probe.has_alpha);
        assert!(!probe.animated);

        let fitted = render_sticker_512(&png).unwrap();
        assert_eq!(fitted.dimensions(), (512, 512));
        // Corner pixel must remain fully transparent
        assert_eq!(fitted.get_pixel(0, 0)[3], 0);
        // Center pixel must remain opaque
        assert_eq!(fitted.get_pixel(256, 256)[3], 255);

        let webp = encode_webp_under_100kb(&fitted).unwrap();
        assert!(webp.len() <= MAX_STATIC_WEBP_BYTES);

        let re_probe = probe_image(&webp).unwrap();
        assert_eq!(re_probe.format, StickerImageFormat::Webp);
        assert_eq!((re_probe.width, re_probe.height), (512, 512));
        assert!(re_probe.has_alpha);
    }

    #[test]
    fn gif_animated_preserves_alpha_and_frame_durations() {
        let gif_bytes = make_animated_transparent_gif();
        let probe = probe_image(&gif_bytes).unwrap();
        assert_eq!(probe.format, StickerImageFormat::Gif);
        assert!(probe.animated);
        assert!(probe.has_alpha);
        assert_eq!(probe.frame_count, 2);
        assert_eq!(probe.duration_ms, 1000);

        let frames = decode_gif_frames_512(&gif_bytes).unwrap();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].delay_ms, 500);
        // Corner alpha is 0 (transparent)
        assert_eq!(frames[0].rgba[3], 0);
    }

    #[test]
    fn matte_composition_replaces_transparent_pixels_cleanly() {
        let png = make_transparent_png(128, 128);
        let fitted = render_sticker_512(&png).unwrap();
        let on_white = composite_on_matte(&fitted, parse_hex_rgb("#FFFFFF"));
        assert_eq!(on_white.get_pixel(0, 0).0, [255, 255, 255, 255]);
        assert_eq!(on_white.get_pixel(256, 256).0, [255, 64, 32, 255]);
    }

    #[test]
    fn sec009_and_tc006_reject_bomb_and_corrupt_bytes() {
        // Fake PNG IHDR claiming 9000x9000 pixels
        let mut fake_png = vec![
            0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, // signature
            0x00, 0x00, 0x00, 0x0D, b'I', b'H', b'D', b'R', // IHDR len + tag
        ];
        fake_png.extend_from_slice(&9000u32.to_be_bytes());
        fake_png.extend_from_slice(&9000u32.to_be_bytes());
        fake_png.extend_from_slice(&[8, 6, 0, 0, 0, 0, 0, 0, 0]);
        let err = probe_image(&fake_png).unwrap_err();
        assert!(matches!(err, ImageEngineError::FileTooLarge(_)));

        let corrupt = b"not an image at all";
        let err2 = probe_image(corrupt).unwrap_err();
        assert!(matches!(err2, ImageEngineError::UnsupportedImage(_)));
    }

    #[test]
    fn crops_1x1_center_and_hover_pan_preserving_input_resolution() {
        let mut wide = RgbaImage::from_pixel(800, 400, Rgba([0, 0, 0, 0]));
        // Put blue pixel at center (400, 200) and red pixel at right edge (750, 200)
        wide.put_pixel(400, 200, Rgba([0, 0, 255, 255]));
        wide.put_pixel(750, 200, Rgba([255, 0, 0, 255]));

        let centered = crop_square_rgba_1x1(&wide, 0.5, 0.5);
        assert_eq!(centered.dimensions(), (400, 400));
        assert_eq!(centered.get_pixel(200, 200).0, [0, 0, 255, 255]);

        let panned_right = crop_square_rgba_1x1(&wide, 1.0, 0.5);
        assert_eq!(panned_right.dimensions(), (400, 400));
        assert_eq!(panned_right.get_pixel(350, 200).0, [255, 0, 0, 255]);
    }
}
