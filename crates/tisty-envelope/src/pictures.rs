use std::io::Cursor;

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::error::ImageError;
use image::imageops::FilterType;
use image::{DynamicImage, ImageDecoder, ImageEncoder, ImageFormat, ImageReader, Limits};

const LONGEST: u32 = 1600;
const JPEG_QUALITY: u8 = 82;
const PIXELS_AT_MOST: u64 = 50_000_000;
const MEMORY_AT_MOST: u64 = 512 * 1024 * 1024;
// A colour profile is a few KB; one this large carries something else.
const PROFILE_AT_MOST: usize = 64 * 1024;
// What the HEIF family opens with, which only the machine may be able to read.
const FAMILY_BRANDS: [&[u8; 4]; 9] = [
    b"heic", b"heix", b"heim", b"heis", b"hevc", b"hevx", b"mif1", b"msf1", b"avif",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Png,
    Jpeg,
}

impl Kind {
    pub fn said(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
        }
    }

    pub fn ending(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Made {
    pub bytes: Vec<u8>,
    pub kind: Kind,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Unmade {
    #[error("these bytes are no picture this machine can read")]
    NotAPicture,
    #[error("this picture has more pixels than can be opened safely")]
    TooLarge,
    #[error("this picture could not be written again")]
    Unwritten,
}

fn unread(wrong: ImageError) -> Unmade {
    match wrong {
        ImageError::Limits(_) => Unmade::TooLarge,
        _ => Unmade::NotAPicture,
    }
}

fn reader(given: &[u8]) -> Result<ImageReader<Cursor<&[u8]>>, Unmade> {
    let mut reader = ImageReader::new(Cursor::new(given))
        .with_guessed_format()
        .map_err(|_| Unmade::NotAPicture)?;
    let mut limits = Limits::default();
    limits.max_alloc = Some(MEMORY_AT_MOST);
    reader.limits(limits);
    Ok(reader)
}

fn is_of_the_heif_family(given: &[u8]) -> bool {
    given.get(4..8) == Some(b"ftyp".as_slice())
        && given
            .get(8..12)
            .is_some_and(|brand| FAMILY_BRANDS.iter().any(|one| one.as_slice() == brand))
}

/// A picture made anew from its pixels, so only pixels travel: metadata, a location and anything hidden in the file stay behind.
pub fn made(given: &[u8]) -> Result<Made, Unmade> {
    let read = reader(given)?;
    let Some(format) = read.format() else {
        return Err(Unmade::NotAPicture);
    };
    written(given, format == ImageFormat::Jpeg)
}

/// The same, for what only the machine can read, as a HEIC: `turned` asks it for bytes this crate reads, and it goes on from those pixels.
pub fn made_or_turned(
    given: &[u8],
    turned: impl FnOnce() -> Option<Vec<u8>>,
) -> Result<Made, Unmade> {
    if !is_of_the_heif_family(given) {
        return made(given);
    }
    let bytes = turned().ok_or(Unmade::NotAPicture)?;
    written(&bytes, true)
}

fn written(given: &[u8], photo: bool) -> Result<Made, Unmade> {
    let (wide, tall) = reader(given)?.into_dimensions().map_err(unread)?;
    if u64::from(wide) * u64::from(tall) > PIXELS_AT_MOST {
        return Err(Unmade::TooLarge);
    }
    let mut decoder = reader(given)?.into_decoder().map_err(unread)?;
    let turn = decoder.orientation().map_err(unread)?;
    let profile = decoder
        .icc_profile()
        .ok()
        .flatten()
        .filter(|one| one.len() <= PROFILE_AT_MOST);
    let mut pixels = DynamicImage::from_decoder(decoder).map_err(unread)?;
    pixels.apply_orientation(turn);
    if pixels.width().max(pixels.height()) > LONGEST {
        pixels = pixels.resize(LONGEST, LONGEST, FilterType::Lanczos3);
    }
    let (width, height) = (pixels.width(), pixels.height());
    let rgba = pixels.to_rgba8();
    let see_through = rgba.pixels().any(|one| one.0[3] < u8::MAX);
    let mut bytes = Vec::new();
    let kind = if photo && !see_through {
        let mut encoder = JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY);
        if let Some(profile) = profile {
            let _ = encoder.set_icc_profile(profile);
        }
        encoder
            .write_image(
                pixels.to_rgb8().as_raw(),
                width,
                height,
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|_| Unmade::Unwritten)?;
        Kind::Jpeg
    } else {
        let mut encoder =
            PngEncoder::new_with_quality(&mut bytes, CompressionType::Best, PngFilter::Adaptive);
        if let Some(profile) = profile {
            let _ = encoder.set_icc_profile(profile);
        }
        let written = if see_through {
            encoder.write_image(
                rgba.as_raw(),
                width,
                height,
                image::ExtendedColorType::Rgba8,
            )
        } else {
            encoder.write_image(
                pixels.to_rgb8().as_raw(),
                width,
                height,
                image::ExtendedColorType::Rgb8,
            )
        };
        written.map_err(|_| Unmade::Unwritten)?;
        Kind::Png
    };
    Ok(Made {
        bytes,
        kind,
        width,
        height,
    })
}

#[cfg(test)]
#[path = "pictures_test.rs"]
mod tests;
