use image::{ImageBuffer, Rgb, Rgba};

use super::*;

fn written(picture: &DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    picture.write_to(&mut out, format).unwrap();
    out.into_inner()
}

fn read(bytes: &[u8]) -> DynamicImage {
    image::load_from_memory(bytes).unwrap()
}

/// Noise does not compress, so it stands for a photo that will not fit as it is.
fn noise(wide: u32, tall: u32) -> DynamicImage {
    let mut state = 0x2545_f491_u32;
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(wide, tall, |_, _| {
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state & 0xff) as u8
        };
        Rgb([next(), next(), next()])
    }))
}

fn flat(wide: u32, tall: u32) -> DynamicImage {
    DynamicImage::ImageRgb8(ImageBuffer::from_pixel(wide, tall, Rgb([30, 90, 160])))
}

#[test]
fn a_small_screenshot_stays_a_png_with_its_pixels() {
    let picture = flat(64, 32);
    let (kind, bytes) = recompressed(&written(&picture, ImageFormat::Png)).unwrap();

    assert_eq!(kind, Kind::Png);
    assert_eq!(Kind::of_picture(&bytes), Some(Kind::Png));
    assert_eq!(read(&bytes).to_rgb8(), picture.to_rgb8());
}

#[test]
fn a_photo_stays_a_jpeg() {
    let (kind, bytes) = recompressed(&written(&noise(200, 100), ImageFormat::Jpeg)).unwrap();

    assert_eq!(kind, Kind::Jpeg);
    assert_eq!((read(&bytes).width(), read(&bytes).height()), (200, 100));
}

#[test]
fn what_can_be_seen_through_keeps_its_transparency() {
    let picture = DynamicImage::ImageRgba8(ImageBuffer::from_pixel(10, 10, Rgba([1, 2, 3, 40])));
    let (kind, bytes) = recompressed(&written(&picture, ImageFormat::Png)).unwrap();

    assert_eq!(kind, Kind::Png);
    assert_eq!(
        read(&bytes).to_rgba8().get_pixel(5, 5),
        &Rgba([1, 2, 3, 40])
    );
}

#[test]
fn a_gif_or_a_webp_comes_out_a_png() {
    let picture = flat(16, 16);
    for format in [ImageFormat::Gif, ImageFormat::WebP] {
        let (kind, bytes) = recompressed(&written(&picture, format)).unwrap();
        assert_eq!(kind, Kind::Png, "{format:?}");
        assert_eq!(Kind::of_picture(&bytes), Some(Kind::Png));
    }
}

#[test]
fn a_huge_picture_is_brought_down_to_the_side_a_page_reads() {
    let (_, bytes) = recompressed(&written(&flat(4000, 1000), ImageFormat::Png)).unwrap();
    let back = read(&bytes);

    assert_eq!(
        (back.width(), back.height()),
        (SIDE_AT_MOST, SIDE_AT_MOST / 4)
    );
}

#[test]
fn a_picture_too_heavy_to_fit_shrinks_until_it_does() {
    let heavy = written(&noise(1600, 1200), ImageFormat::Png);
    assert!(heavy.len() > PICTURE_AT_MOST);

    let (kind, bytes) = recompressed(&heavy).unwrap();
    assert!(bytes.len() <= PICTURE_AT_MOST);
    // Nothing in it is see-through, so it may become a JPEG before it loses a pixel.
    assert_eq!(kind, Kind::Jpeg);
}

#[test]
fn a_see_through_picture_too_heavy_for_png_shrinks_and_stays_a_png() {
    let mut state = 7u32;
    let picture = DynamicImage::ImageRgba8(ImageBuffer::from_fn(1400, 1000, |_, _| {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        let [a, b, c, d] = state.to_be_bytes();
        Rgba([a, b, c, d | 1])
    }));
    let (kind, bytes) = recompressed(&written(&picture, ImageFormat::Png)).unwrap();

    assert_eq!(kind, Kind::Png);
    assert!(bytes.len() <= PICTURE_AT_MOST);
    assert!(read(&bytes).width() < 1400);
}

#[test]
fn sixteen_bits_a_channel_come_down_to_eight() {
    let picture = DynamicImage::ImageRgb16(ImageBuffer::from_pixel(8, 8, Rgb([0x1234, 0, 0xffff])));
    let (_, bytes) = recompressed(&written(&picture, ImageFormat::Png)).unwrap();

    assert!(matches!(read(&bytes), DynamicImage::ImageRgb8(_)));
}

#[test]
fn only_pictures_are_taken() {
    assert_eq!(recompressed(b"%PDF-1.7").err(), Some(Refused::NotAPicture));
    assert_eq!(recompressed(b"").err(), Some(Refused::NotAPicture));
    // The right first bytes and nothing behind them.
    assert_eq!(
        recompressed(b"\x89PNG\r\n\x1a\n not really").err(),
        Some(Refused::NotAPicture)
    );
}

#[test]
fn a_file_hidden_after_a_picture_does_not_travel() {
    let mut smuggled = written(&flat(8, 8), ImageFormat::Png);
    smuggled.extend_from_slice(b"PK\x03\x04 a zip file riding along");
    let (_, bytes) = recompressed(&smuggled).unwrap();

    assert!(!bytes.windows(4).any(|one| one == b"PK\x03\x04"));
}
