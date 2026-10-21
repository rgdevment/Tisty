use std::path::{Path, PathBuf};

const DRAWN: &str = "drawn";

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
                    && wic::png(&found, &into).is_ok());
            return made.then_some(into);
        }
    }
    #[cfg(not(windows))]
    let _ = cache;
    Some(found)
}

// COM is reached only through unsafe calls; nothing here outlives the call that made it.
#[cfg(windows)]
#[allow(unsafe_code)]
mod wic {
    use std::path::Path;

    use windows::Win32::Foundation::GENERIC_READ;
    use windows::Win32::Graphics::Imaging::{
        CLSID_WICImagingFactory, GUID_ContainerFormatPng, GUID_WICPixelFormat32bppBGRA,
        IWICBitmapFrameEncode, IWICImagingFactory, WICBitmapDitherTypeNone,
        WICBitmapEncoderNoCache, WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
    };
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
    };
    use windows::core::HSTRING;

    // Written beside and moved in whole, so a picture cut short is never taken for a drawn one.
    pub fn png(from: &Path, into: &Path) -> windows::core::Result<()> {
        let part = into.with_extension("png.part");
        let drawn = drawn(from, &part);
        match drawn.and_then(|()| std::fs::rename(&part, into).map_err(|e| e.into())) {
            Ok(()) => Ok(()),
            Err(why) => {
                let _ = std::fs::remove_file(&part);
                Err(why)
            }
        }
    }

    fn drawn(from: &Path, into: &Path) -> windows::core::Result<()> {
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
            let converter = factory.CreateFormatConverter()?;
            converter.Initialize(
                &frame,
                &GUID_WICPixelFormat32bppBGRA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeCustom,
            )?;
            let stream = factory.CreateStream()?;
            stream.InitializeFromFilename(&HSTRING::from(into.as_os_str()), 0x4000_0000)?;
            let encoder = factory.CreateEncoder(&GUID_ContainerFormatPng, std::ptr::null())?;
            encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;
            let mut page: Option<IWICBitmapFrameEncode> = None;
            encoder.CreateNewFrame(&mut page, std::ptr::null_mut())?;
            let Some(page) = page else {
                return Err(windows::core::Error::empty());
            };
            page.Initialize(None)?;
            let (mut wide, mut high) = (0, 0);
            converter.GetSize(&mut wide, &mut high)?;
            page.SetSize(wide, high)?;
            let mut format = GUID_WICPixelFormat32bppBGRA;
            page.SetPixelFormat(&mut format)?;
            page.WriteSource(&converter, std::ptr::null())?;
            page.Commit()?;
            encoder.Commit()
        }
    }
}
