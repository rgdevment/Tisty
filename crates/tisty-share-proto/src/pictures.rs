use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::imageops::FilterType as Resize;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader};

use crate::envelope::{Kind, Refused};
use crate::limits::Limits;

/// A page is read a few hundred pixels wide; anything past this is only weight.
const SIDE_AT_MOST: u32 = 2048;
const QUALITY: u8 = 82;
/// Each try keeps three quarters of the side, so six reach about a sixth of where it began.
const TRIES: usize = 6;
/// Far past any photo, short of what a crafted header could ask a machine to hold.
const DECODED_AT_MOST: u64 = 256 * 1024 * 1024;

/// Decoded and written again, so nothing travels but the pixels: no location, no comments, no hidden file.
pub fn recompressed(bytes: &[u8], limits: &Limits) -> Result<(Kind, Vec<u8>), Refused> {
    let kind = Kind::of_picture(bytes).ok_or(Refused::NotAPicture)?;
    if !limits.takes(Kind::Png) && !limits.takes(Kind::Jpeg) {
        return Err(Refused::KindRefused);
    }
    let mut picture = fitted(decoded(bytes, kind)?, SIDE_AT_MOST);
    for _ in 0..TRIES {
        if let Some(smallest) = smallest_of(&picture, kind, limits)? {
            return Ok(smallest);
        }
        let (wide, tall) = (picture.width(), picture.height());
        picture = picture.resize(
            (wide * 3 / 4).max(1),
            (tall * 3 / 4).max(1),
            Resize::Lanczos3,
        );
    }
    Err(Refused::PictureTooBig)
}

fn decoded(bytes: &[u8], kind: Kind) -> Result<DynamicImage, Refused> {
    let format = match kind {
        Kind::Png => ImageFormat::Png,
        Kind::Jpeg => ImageFormat::Jpeg,
        Kind::Webp => ImageFormat::WebP,
        Kind::Gif => ImageFormat::Gif,
        Kind::Text | Kind::Signature => return Err(Refused::NotAPicture),
    };
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    let mut held = image::Limits::default();
    held.max_alloc = Some(DECODED_AT_MOST);
    reader.limits(held);
    let mut decoder = reader.into_decoder().map_err(|_| Refused::NotAPicture)?;
    // A photo stands as it was taken, since the tag that turned it is not written again.
    let orientation = decoder.orientation().unwrap_or(Orientation::NoTransforms);
    let mut picture = DynamicImage::from_decoder(decoder).map_err(|_| Refused::NotAPicture)?;
    picture.apply_orientation(orientation);
    Ok(plain(picture))
}

/// Eight bits a channel is all a page shows, and half the bytes of sixteen.
fn plain(picture: DynamicImage) -> DynamicImage {
    if picture.color().has_alpha() {
        DynamicImage::ImageRgba8(picture.to_rgba8())
    } else {
        DynamicImage::ImageRgb8(picture.to_rgb8())
    }
}

fn fitted(picture: DynamicImage, side: u32) -> DynamicImage {
    if picture.width() <= side && picture.height() <= side {
        picture
    } else {
        picture.resize(side, side, Resize::Lanczos3)
    }
}

/// A photo stays a photo and the rest a PNG; an opaque PNG may become a JPEG to fit, and a server's one kind wins.
fn smallest_of(
    picture: &DynamicImage,
    kind: Kind,
    limits: &Limits,
) -> Result<Option<(Kind, Vec<u8>)>, Refused> {
    let fits = |bytes: &[u8]| bytes.len() as u64 <= limits.picture;
    let (png, jpeg) = (limits.takes(Kind::Png), limits.takes(Kind::Jpeg));
    if png && (kind != Kind::Jpeg || !jpeg) {
        let written = as_png(picture)?;
        if fits(&written) {
            return Ok(Some((Kind::Png, written)));
        }
        if picture.color().has_alpha() {
            return Ok(None);
        }
    }
    if !jpeg {
        return Ok(None);
    }
    let written = as_jpeg(picture)?;
    Ok(fits(&written).then_some((Kind::Jpeg, written)))
}

fn as_png(picture: &DynamicImage) -> Result<Vec<u8>, Refused> {
    let mut out = Vec::new();
    picture
        .write_with_encoder(PngEncoder::new_with_quality(
            &mut out,
            CompressionType::Best,
            FilterType::Adaptive,
        ))
        .map_err(|_| Refused::NotAPicture)?;
    Ok(out)
}

fn as_jpeg(picture: &DynamicImage) -> Result<Vec<u8>, Refused> {
    let mut out = Vec::new();
    DynamicImage::ImageRgb8(picture.to_rgb8())
        .write_with_encoder(JpegEncoder::new_with_quality(&mut out, QUALITY))
        .map_err(|_| Refused::NotAPicture)?;
    Ok(out)
}

#[cfg(test)]
#[path = "pictures_test.rs"]
mod tests;
