use std::io::Cursor;

use image::{GenericImageView, Rgb, RgbImage, Rgba, RgbaImage};

use super::*;

fn encoded(image: DynamicImage, format: ImageFormat) -> Vec<u8> {
    let mut out = Vec::new();
    image.write_to(&mut Cursor::new(&mut out), format).unwrap();
    out
}

fn a_photo(wide: u32, tall: u32) -> Vec<u8> {
    let pixels = RgbImage::from_fn(wide, tall, |x, y| {
        Rgb([(x % 255) as u8, (y % 255) as u8, 90])
    });
    encoded(DynamicImage::ImageRgb8(pixels), ImageFormat::Jpeg)
}

// An APP1 with EXIF that turns a photo a quarter and says where it was taken, right after the JPEG's first marker.
fn with_exif(jpeg: &[u8], orientation: u16) -> Vec<u8> {
    let mut exif = b"Exif\0\0MM\0\x2a\0\0\0\x08\0\x01\x01\x12\0\x03\0\0\0\x01".to_vec();
    exif.extend_from_slice(&orientation.to_be_bytes());
    exif.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
    exif.extend_from_slice(b"GPSLatitude-33.45");
    let mut out = jpeg[..2].to_vec();
    out.extend_from_slice(&[0xFF, 0xE1]);
    out.extend_from_slice(&((exif.len() + 2) as u16).to_be_bytes());
    out.extend_from_slice(&exif);
    out.extend_from_slice(&jpeg[2..]);
    out
}

fn holds(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|one| one == needle)
}

#[test]
fn a_photo_goes_as_jpeg_and_opens_as_what_it_says() {
    let done = made(&a_photo(40, 30)).unwrap();
    assert_eq!(done.kind, Kind::Jpeg);
    assert_eq!((done.width, done.height), (40, 30));
    assert!(done.bytes.starts_with(&[0xFF, 0xD8, 0xFF]));
    assert_eq!(
        image::load_from_memory(&done.bytes).unwrap().dimensions(),
        (40, 30)
    );
}

#[test]
fn nothing_but_pixels_is_left_of_what_came_in() {
    let mut given = with_exif(&a_photo(40, 30), 1);
    // A video or a script after the picture's end is how a motion photo or a disguised file looks.
    given.extend_from_slice(b"ftypmp42 and a hidden payload");
    assert!(holds(&given, b"GPSLatitude"));
    let done = made(&given).unwrap();
    assert!(!holds(&done.bytes, b"Exif"));
    assert!(!holds(&done.bytes, b"GPSLatitude"));
    assert!(!holds(&done.bytes, b"hidden payload"));
    assert!(done.bytes.ends_with(&[0xFF, 0xD9]));
}

#[test]
fn its_orientation_ends_up_in_its_pixels() {
    let done = made(&with_exif(&a_photo(40, 30), 6)).unwrap();
    assert_eq!((done.width, done.height), (30, 40));
}

#[test]
fn its_longest_side_is_at_most_what_the_contract_says() {
    let done = made(&a_photo(3200, 800)).unwrap();
    assert_eq!((done.width, done.height), (1600, 400));
}

#[test]
fn what_is_not_a_photo_goes_as_png() {
    let pixels = RgbImage::from_pixel(10, 10, Rgb([200, 10, 10]));
    let done = made(&encoded(DynamicImage::ImageRgb8(pixels), ImageFormat::Png)).unwrap();
    assert_eq!(done.kind, Kind::Png);
    assert!(
        done.bytes
            .starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A])
    );
}

#[test]
fn alpha_stays_only_where_it_covers_something() {
    let opaque = RgbaImage::from_pixel(4, 4, Rgba([1, 2, 3, 255]));
    let done = made(&encoded(DynamicImage::ImageRgba8(opaque), ImageFormat::Png)).unwrap();
    assert!(
        !image::load_from_memory(&done.bytes)
            .unwrap()
            .color()
            .has_alpha()
    );

    let seen_through = RgbaImage::from_pixel(4, 4, Rgba([1, 2, 3, 10]));
    let done = made(&encoded(
        DynamicImage::ImageRgba8(seen_through),
        ImageFormat::Png,
    ))
    .unwrap();
    assert!(
        image::load_from_memory(&done.bytes)
            .unwrap()
            .color()
            .has_alpha()
    );
}

#[test]
fn sixteen_bits_come_down_to_eight() {
    let deep = image::ImageBuffer::<Rgb<u16>, Vec<u16>>::from_pixel(4, 4, Rgb([60000, 2, 3]));
    let done = made(&encoded(DynamicImage::ImageRgb16(deep), ImageFormat::Png)).unwrap();
    assert_eq!(
        image::load_from_memory(&done.bytes).unwrap().color(),
        image::ColorType::Rgb8
    );
}

#[test]
fn an_animated_picture_stays_its_first_frame() {
    use image::codecs::gif::GifEncoder;
    use image::{Delay, Frame};
    let mut gif = Vec::new();
    {
        let mut encoder = GifEncoder::new(&mut gif);
        for shade in [200, 10] {
            let frame = Frame::from_parts(
                RgbaImage::from_pixel(4, 4, Rgba([shade, shade, shade, 255])),
                0,
                0,
                Delay::from_numer_denom_ms(100, 1),
            );
            encoder.encode_frame(frame).unwrap();
        }
    }
    let done = made(&gif).unwrap();
    assert_eq!(done.kind, Kind::Png);
    let first = image::load_from_memory(&done.bytes).unwrap().to_rgb8();
    assert!(first.get_pixel(0, 0).0[0] > 150);
}

#[test]
fn a_name_says_nothing_only_the_bytes_decide() {
    assert_eq!(
        made(b"%PDF-1.7 a document named foto.png"),
        Err(Unmade::NotAPicture)
    );
    assert_eq!(
        made(b"<svg xmlns='http://www.w3.org/2000/svg'><script>x()</script></svg>"),
        Err(Unmade::NotAPicture)
    );
    assert_eq!(
        made(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 1, 2, 3]),
        Err(Unmade::NotAPicture)
    );
}

#[test]
fn a_header_that_claims_too_many_pixels_is_never_decoded() {
    // A real header saying 20000 by 20000, the shape of a decode bomb, with nothing behind it.
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13];
    let mut chunk = b"IHDR".to_vec();
    chunk.extend_from_slice(&20_000u32.to_be_bytes());
    chunk.extend_from_slice(&20_000u32.to_be_bytes());
    chunk.extend_from_slice(&[8, 2, 0, 0, 0]);
    let crc = crc_of(&chunk);
    png.extend_from_slice(&chunk);
    png.extend_from_slice(&crc.to_be_bytes());
    png.extend_from_slice(&[0, 0, 0, 0]);
    png.extend_from_slice(b"IDAT");
    png.extend_from_slice(&crc_of(b"IDAT").to_be_bytes());
    assert_eq!(made(&png), Err(Unmade::TooLarge));
}

fn crc_of(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[test]
fn what_only_the_machine_reads_goes_on_from_what_it_turned_it_into() {
    let heic = b"\0\0\0\x18ftypheic\0\0\0\0mif1heic";
    let done = made_or_turned(heic, || Some(a_photo(20, 10))).unwrap();
    assert_eq!(done.kind, Kind::Jpeg);
    assert_eq!((done.width, done.height), (20, 10));
    assert_eq!(made_or_turned(heic, || None), Err(Unmade::NotAPicture));
}

#[test]
fn a_heic_named_only_among_its_other_brands_is_still_one() {
    let told_in_its_list = b"\0\0\0\x18ftypiso8\0\0\0\0mif1heic";
    let done = made_or_turned(told_in_its_list, || Some(a_photo(20, 10))).unwrap();
    assert_eq!(done.kind, Kind::Jpeg);
}

#[test]
fn a_file_cut_short_is_no_picture() {
    let whole = a_photo(40, 30);
    assert_eq!(made(&whole[..whole.len() / 2]), Err(Unmade::NotAPicture));
    let png = encoded(
        DynamicImage::ImageRgb8(RgbImage::from_pixel(40, 30, Rgb([1, 2, 3]))),
        ImageFormat::Png,
    );
    assert_eq!(made(&png[..png.len() / 2]), Err(Unmade::NotAPicture));
}

#[test]
fn only_a_real_profile_for_these_pixels_stays() {
    let mut real = vec![0u8; 128];
    real[16..20].copy_from_slice(b"RGB ");
    real[36..40].copy_from_slice(b"acsp");
    assert!(is_a_profile_for_these_pixels(&real));
    let mut gray = real.clone();
    gray[16..20].copy_from_slice(b"GRAY");
    assert!(!is_a_profile_for_these_pixels(&gray));
    let mut cmyk = real.clone();
    cmyk[16..20].copy_from_slice(b"CMYK");
    assert!(!is_a_profile_for_these_pixels(&cmyk));
    assert!(!is_a_profile_for_these_pixels(
        b"a long text about where this was taken, not a profile at all"
    ));
    real.resize(PROFILE_AT_MOST + 1, 0);
    assert!(!is_a_profile_for_these_pixels(&real));
}

#[test]
fn what_it_reads_itself_never_asks_the_machine() {
    let done = made_or_turned(&a_photo(20, 10), || panic!("asked the machine")).unwrap();
    assert_eq!(done.kind, Kind::Jpeg);
}
