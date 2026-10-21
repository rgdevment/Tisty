use std::path::{Path, PathBuf};

const DRAWN: &str = "drawn";
// Enough for a screen and a page, and a phone's photo shrinks to a fraction of what it weighed.
const LONGEST: u32 = 2048;
const FULL_QUALITY: f32 = 0.95;
const LIGHT_QUALITY: f32 = 0.8;
#[cfg(windows)]
const SEEN_QUALITY: f32 = 0.9;
static TURNS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn folder(cache: &Path) -> PathBuf {
    cache.join(DRAWN)
}

/// WebView2 draws no HEIC, so on Windows it is shown as a JPEG Windows drew once; none when it cannot.
pub fn as_seen(found: PathBuf, cache: &Path) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let heic = found
            .extension()
            .is_some_and(|one| one.eq_ignore_ascii_case("heic"));
        if heic && let Some(stem) = found.file_stem() {
            let into = folder(cache).join(stem).with_extension("jpg");
            let shape = wic::Shape {
                quality: SEEN_QUALITY,
                longest: Some(LONGEST),
            };
            let made = into.is_file()
                || (std::fs::create_dir_all(folder(cache)).is_ok()
                    && wic::written(&found, &into, &shape).is_ok());
            return made.then_some(into);
        }
    }
    #[cfg(not(windows))]
    let _ = cache;
    Some(found)
}

pub fn draws_heic() -> bool {
    #[cfg(windows)]
    let can = wic::reads_heif();
    #[cfg(target_os = "macos")]
    let can = true;
    #[cfg(not(any(windows, target_os = "macos")))]
    let can = false;
    can
}

// Its own folder, so the caller can take it away whole once the photo is kept.
pub fn reshaped(from: &Path, named: &str, light: bool) -> Option<PathBuf> {
    let room = std::env::temp_dir().join(format!(
        "tisty-reshaped-{}-{}",
        std::process::id(),
        TURNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&room).ok()?;
    let into = room.join(named);
    let quality = if light { LIGHT_QUALITY } else { FULL_QUALITY };
    let longest = light.then_some(LONGEST);
    #[cfg(windows)]
    let made = wic::written(from, &into, &wic::Shape { quality, longest }).is_ok();
    #[cfg(target_os = "macos")]
    let made = sips(from, &into, quality, longest);
    #[cfg(not(any(windows, target_os = "macos")))]
    let made = {
        let _ = (from, quality, longest);
        false
    };
    if made && into.is_file() {
        return Some(into);
    }
    let _ = std::fs::remove_dir_all(&room);
    None
}

// The one that comes with every Mac, which reads whatever Preview reads and keeps the photo upright.
#[cfg(target_os = "macos")]
fn sips(from: &Path, into: &Path, quality: f32, longest: Option<u32>) -> bool {
    let mut asked = std::process::Command::new("/usr/bin/sips");
    asked
        .args(["-s", "format", "jpeg", "-s", "formatOptions"])
        .arg(format!("{}", (quality * 100.0).round() as u32));
    if let Some(longest) = longest {
        asked.arg("-Z").arg(longest.to_string());
    }
    asked
        .arg(from)
        .arg("--out")
        .arg(into)
        .output()
        .is_ok_and(|done| done.status.success())
}

// COM is reached only through unsafe calls; nothing here outlives the call that made it.
#[cfg(windows)]
#[allow(unsafe_code)]
mod wic {
    use std::path::Path;

    use windows::Win32::Foundation::GENERIC_READ;
    use windows::Win32::Graphics::Imaging::{
        CLSID_WICImagingFactory, GUID_ContainerFormatHeif, GUID_ContainerFormatJpeg,
        GUID_WICPixelFormat24bppBGR, IWICBitmapFrameDecode, IWICBitmapFrameEncode,
        IWICBitmapSource, IWICImagingFactory, WICBitmapDitherTypeNone, WICBitmapEncoderNoCache,
        WICBitmapInterpolationModeFant, WICBitmapPaletteTypeCustom,
        WICBitmapTransformFlipHorizontal, WICBitmapTransformFlipVertical,
        WICBitmapTransformOptions, WICBitmapTransformRotate0, WICBitmapTransformRotate90,
        WICBitmapTransformRotate180, WICBitmapTransformRotate270, WICDecodeMetadataCacheOnDemand,
    };
    use windows::Win32::System::Com::StructuredStorage::{IPropertyBag2, PROPBAG2, PROPVARIANT};
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
    };
    use windows::Win32::System::Variant::VARIANT;
    use windows::core::{HSTRING, Interface, PWSTR};

    // A HEIF decoder registered with Windows is what its extension from the Store installs.
    pub fn reads_heif() -> bool {
        let container = unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            CoCreateInstance::<_, IWICImagingFactory>(
                &CLSID_WICImagingFactory,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .and_then(|factory| factory.CreateDecoder(&GUID_ContainerFormatHeif, std::ptr::null()))
            .is_ok()
        };
        container && reads_hevc()
    }

    // The HEIF container comes with Windows; the HEVC pictures inside it need a decoder of their own.
    fn reads_hevc() -> bool {
        use windows::Win32::Media::MediaFoundation::{
            IMFActivate, MFMediaType_Video, MFT_CATEGORY_VIDEO_DECODER, MFT_ENUM_FLAG_ALL,
            MFT_REGISTER_TYPE_INFO, MFTEnumEx, MFVideoFormat_HEVC,
        };
        use windows::Win32::System::Com::CoTaskMemFree;
        let asked = MFT_REGISTER_TYPE_INFO {
            guidMajorType: MFMediaType_Video,
            guidSubtype: MFVideoFormat_HEVC,
        };
        let mut found: *mut Option<IMFActivate> = std::ptr::null_mut();
        let mut count = 0u32;
        unsafe {
            let listed = MFTEnumEx(
                MFT_CATEGORY_VIDEO_DECODER,
                MFT_ENUM_FLAG_ALL,
                Some(&asked),
                None,
                &mut found,
                &mut count,
            );
            if !found.is_null() {
                for at in 0..count as usize {
                    drop(found.add(at).read());
                }
                CoTaskMemFree(Some(found.cast()));
            }
            listed.is_ok() && count > 0
        }
    }

    pub struct Shape {
        pub quality: f32,
        pub longest: Option<u32>,
    }

    // Written beside under a name of its own and moved in whole, so two asking at once never share one.
    pub fn written(from: &Path, into: &Path, shape: &Shape) -> windows::core::Result<()> {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        let part = into.with_extension(format!("{stamp}.part"));
        let done = drawn(from, &part, shape).and_then(|()| {
            if into.is_file() {
                return Ok(());
            }
            std::fs::rename(&part, into).map_err(windows::core::Error::from)
        });
        let _ = std::fs::remove_file(&part);
        done
    }

    // EXIF's eight ways a photo may lie, as the turn and flip that set it upright again.
    fn upright(said: u16) -> WICBitmapTransformOptions {
        let turn = |a: WICBitmapTransformOptions, b: WICBitmapTransformOptions| {
            WICBitmapTransformOptions(a.0 | b.0)
        };
        match said {
            2 => WICBitmapTransformFlipHorizontal,
            3 => WICBitmapTransformRotate180,
            4 => WICBitmapTransformFlipVertical,
            5 => turn(WICBitmapTransformRotate90, WICBitmapTransformFlipHorizontal),
            6 => WICBitmapTransformRotate90,
            7 => turn(
                WICBitmapTransformRotate270,
                WICBitmapTransformFlipHorizontal,
            ),
            8 => WICBitmapTransformRotate270,
            _ => WICBitmapTransformRotate0,
        }
    }

    fn orientation(frame: &IWICBitmapFrameDecode) -> u16 {
        let mut said = PROPVARIANT::default();
        let read = unsafe {
            frame.GetMetadataQueryReader().and_then(|reader| {
                reader.GetMetadataByName(&HSTRING::from("System.Photo.Orientation"), &mut said)
            })
        };
        read.ok()
            .and_then(|()| u16::try_from(&said).ok())
            .unwrap_or(1)
    }

    fn drawn(from: &Path, into: &Path, shape: &Shape) -> windows::core::Result<()> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let factory: IWICImagingFactory =
                CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
            let decoder = factory.CreateDecoderFromFilename(
                &HSTRING::from(from.as_os_str()),
                None,
                GENERIC_READ,
                WICDecodeMetadataCacheOnDemand,
            )?;
            let frame = decoder.GetFrame(0)?;
            // HEIF carries its own turn inside the container; reading EXIF on top of it would turn it twice.
            let heif = from.extension().is_some_and(|one| {
                one.eq_ignore_ascii_case("heic") || one.eq_ignore_ascii_case("heif")
            });
            let turned = match heif {
                true => WICBitmapTransformRotate0,
                false => upright(orientation(&frame)),
            };
            let mut source: IWICBitmapSource = frame.cast()?;
            if turned != WICBitmapTransformRotate0 {
                let rotator = factory.CreateBitmapFlipRotator()?;
                rotator.Initialize(&source, turned)?;
                source = rotator.cast()?;
            }
            let (mut wide, mut high) = (0, 0);
            source.GetSize(&mut wide, &mut high)?;
            if let Some(longest) = shape.longest.filter(|most| wide.max(high) > *most) {
                let ratio = f64::from(longest) / f64::from(wide.max(high));
                wide = ((f64::from(wide) * ratio).round() as u32).max(1);
                high = ((f64::from(high) * ratio).round() as u32).max(1);
                let scaler = factory.CreateBitmapScaler()?;
                scaler.Initialize(&source, wide, high, WICBitmapInterpolationModeFant)?;
                source = scaler.cast()?;
            }
            let converter = factory.CreateFormatConverter()?;
            converter.Initialize(
                &source,
                &GUID_WICPixelFormat24bppBGR,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeCustom,
            )?;
            let stream = factory.CreateStream()?;
            stream.InitializeFromFilename(&HSTRING::from(into.as_os_str()), 0x4000_0000)?;
            let encoder = factory.CreateEncoder(&GUID_ContainerFormatJpeg, std::ptr::null())?;
            encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;
            let mut page: Option<IWICBitmapFrameEncode> = None;
            let mut options: Option<IPropertyBag2> = None;
            encoder.CreateNewFrame(&mut page, &mut options)?;
            let Some(page) = page else {
                return Err(windows::core::Error::empty());
            };
            if let Some(options) = &options {
                let name = HSTRING::from("ImageQuality");
                let named = PROPBAG2 {
                    pstrName: PWSTR(name.as_ptr().cast_mut()),
                    ..Default::default()
                };
                options.Write(1, &named, &VARIANT::from(shape.quality))?;
            }
            page.Initialize(options.as_ref())?;
            page.SetSize(wide, high)?;
            let mut written_as = GUID_WICPixelFormat24bppBGR;
            page.SetPixelFormat(&mut written_as)?;
            page.WriteSource(&converter, std::ptr::null())?;
            page.Commit()?;
            encoder.Commit()
        }
    }
}
