use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::imageops::FilterType as Resize;
use image::metadata::Orientation;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits};

use crate::envelope::{Kind, PICTURE_AT_MOST, Refused};

/// A page is read a few hundred pixels wide; anything past this is only weight.
const SIDE_AT_MOST: u32 = 2048;
const QUALITY: u8 = 82;
/// Each try keeps three quarters of the side, so six reach about a sixth of where it began.
const TRIES: usize = 6;
/// Far past any photo, short of what a crafted header could ask a machine to hold.
const DECODED_AT_MOST: u64 = 256 * 1024 * 1024;

/// Decoded and written again, so nothing travels but the pixels: no location, no comments, no hidden file.
pub fn recompressed(bytes: &[u8]) -> Result<(Kind, Vec<u8>), Refused> {
    let kind = Kind::of_picture(bytes).ok_or(Refused::NotAPicture)?;
    let mut picture = fitted(decoded(bytes, kind)?, SIDE_AT_MOST);
    for _ in 0..TRIES {
        if let Some(smallest) = smallest_of(&picture, kind)? {
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
    let mut limits = Limits::default();
    limits.max_alloc = Some(DECODED_AT_MOST);
    reader.limits(limits);
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

/// A photo stays a photo and anything else a PNG; a PNG with nothing see-through may become a JPEG to fit.
fn smallest_of(picture: &DynamicImage, kind: Kind) -> Result<Option<(Kind, Vec<u8>)>, Refused> {
    let see_through = picture.color().has_alpha();
    if kind != Kind::Jpeg {
        let png = as_png(picture)?;
        if png.len() <= PICTURE_AT_MOST {
            return Ok(Some((Kind::Png, png)));
        }
    }
    if see_through {
        return Ok(None);
    }
    let jpeg = as_jpeg(picture)?;
    Ok((jpeg.len() <= PICTURE_AT_MOST).then_some((Kind::Jpeg, jpeg)))
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
