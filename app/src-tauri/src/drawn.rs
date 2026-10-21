use std::path::{Path, PathBuf};

const DRAWN: &str = "drawn";
// Long enough for a screen and a page, and a phone's photo shrinks to a fraction of what it weighed.
const LIGHT_LONGEST: u32 = 2048;
const FULL_QUALITY: f32 = 0.95;
const LIGHT_QUALITY: f32 = 0.8;

pub fn folder(cache: &Path) -> PathBuf {
    cache.join(DRAWN)
}

/// WebView2 draws no HEIC, so on Windows the picture shown is a PNG Windows itself drew once from it;
/// none when Windows has nothing to draw it with.
pub fn as_seen(found: PathBuf, cache: &Path) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let heic = found
            .extension()
            .is_some_and(|one| one.eq_ignore_ascii_case("heic"));
        if heic && let Some(stem) = found.file_stem() {
            let into = folder(cache).join(stem).with_extension("png");
            let made = into.is_file()
                || (std::fs::create_dir_all(folder(cache)).is_ok()
                    && wic::written(&found, &into, wic::Shape::Seen).is_ok());
            return made.then_some(into);
        }
    }
    #[cfg(not(windows))]
    let _ = cache;
    Some(found)
}

/// A photo kept as JPEG instead, whole or lighter, written beside the system's temporary files.
pub fn reshaped(from: &Path, light: bool) -> Option<PathBuf> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    let into = std::env::temp_dir().join(format!("tisty-reshaped-{stamp}.jpg"));
    let quality = if light { LIGHT_QUALITY } else { FULL_QUALITY };
    let longest = light.then_some(LIGHT_LONGEST);
    #[cfg(windows)]
    let made = wic::written(from, &into, wic::Shape::Jpeg { quality, longest }).is_ok();
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
    let _ = std::fs::remove_file(&into);
    None
}

// The one that comes with every Mac, which reads whatever Preview reads.
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
        CLSID_WICImagingFactory, GUID_ContainerFormatJpeg, GUID_ContainerFormatPng,
        GUID_WICPixelFormat24bppBGR, GUID_WICPixelFormat32bppBGRA, IWICBitmapFrameEncode,
        IWICBitmapSource, IWICImagingFactory, WICBitmapDitherTypeNone, WICBitmapEncoderNoCache,
        WICBitmapInterpolationModeFant, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
    };
    use windows::Win32::System::Com::StructuredStorage::{IPropertyBag2, PROPBAG2};
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
    };
    use windows::Win32::System::Variant::VARIANT;
    use windows::core::{GUID, HSTRING, Interface, PWSTR};

    pub enum Shape {
        Seen,
        Jpeg { quality: f32, longest: Option<u32> },
    }

    // Written beside and moved in whole, so a picture cut short is never taken for a finished one.
    pub fn written(from: &Path, into: &Path, shape: Shape) -> windows::core::Result<()> {
        let part = into.with_extension("part");
        let done = drawn(from, &part, &shape)
            .and_then(|()| std::fs::rename(&part, into).map_err(windows::core::Error::from));
        if done.is_err() {
            let _ = std::fs::remove_file(&part);
        }
        done
    }

    fn drawn(from: &Path, into: &Path, shape: &Shape) -> windows::core::Result<()> {
        let (container, format, quality, longest): (GUID, GUID, Option<f32>, Option<u32>) =
            match shape {
                Shape::Seen => (
                    GUID_ContainerFormatPng,
                    GUID_WICPixelFormat32bppBGRA,
                    None,
                    None,
                ),
                Shape::Jpeg { quality, longest } => (
                    GUID_ContainerFormatJpeg,
                    GUID_WICPixelFormat24bppBGR,
                    Some(*quality),
                    *longest,
                ),
            };
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
            let mut source: IWICBitmapSource = decoder.GetFrame(0)?.cast()?;
            let (mut wide, mut high) = (0, 0);
            source.GetSize(&mut wide, &mut high)?;
            if let Some(longest) = longest.filter(|most| wide.max(high) > *most) {
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
                &format,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeCustom,
            )?;
            let stream = factory.CreateStream()?;
            stream.InitializeFromFilename(&HSTRING::from(into.as_os_str()), 0x4000_0000)?;
            let encoder = factory.CreateEncoder(&container, std::ptr::null())?;
            encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;
            let mut page: Option<IWICBitmapFrameEncode> = None;
            let mut options: Option<IPropertyBag2> = None;
            encoder.CreateNewFrame(&mut page, &mut options)?;
            let Some(page) = page else {
                return Err(windows::core::Error::empty());
            };
            if let (Some(quality), Some(options)) = (quality, &options) {
                let name = HSTRING::from("ImageQuality");
                let named = PROPBAG2 {
                    pstrName: PWSTR(name.as_ptr().cast_mut()),
                    ..Default::default()
                };
                options.Write(1, &named, &VARIANT::from(quality))?;
            }
            page.Initialize(options.as_ref())?;
            page.SetSize(wide, high)?;
            let mut written_as = format;
            page.SetPixelFormat(&mut written_as)?;
            page.WriteSource(&converter, std::ptr::null())?;
            page.Commit()?;
            encoder.Commit()
        }
    }
}
