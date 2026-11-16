//! The interface font, IBM Plex Mono, loaded from the repository's own files (`fonts/`) instead of the
//! system's: the page uses the same glyphs, and a machine without the font draws the same picture.
//!
//! DirectWrite reads woff2 only through `UnpackFontFile`, which gives plain font bytes; those are handed
//! to an in-memory loader and gathered in a font set, then a collection that text formats are made from.
//!
//! All three faces declare the family IBM Plex Mono, so the Pi face (arrows and signs) is found by glyph
//! coverage inside that family: no custom font fallback is needed (checked by removing the Pi file, which
//! changes the picture).

use std::ffi::c_void;

use mere_base::observe;
use windows::Win32::Graphics::DirectWrite::*;
use windows::core::Result;

/// The family text is asked for. The collection holds it; the system does not need to.
pub const FAMILY: &str = "IBM Plex Mono";
/// What is drawn when the files cannot be loaded.
pub const FALLBACK_FAMILY: &str = "Consolas";

const LATIN1_REGULAR: &[u8] = include_bytes!("../../../fonts/IBMPlexMono-Regular-Latin1.woff2");
const LATIN1_SEMIBOLD: &[u8] = include_bytes!("../../../fonts/IBMPlexMono-SemiBold-Latin1.woff2");
const PI_REGULAR: &[u8] = include_bytes!("../../../fonts/IBMPlexMono-Regular-Pi.woff2");

pub struct Fonts {
    pub collection: IDWriteFontCollection1,
    // The in-memory loader points into these bytes without copying them, so they live as long as the collection.
    _bytes: Vec<Vec<u8>>,
    _loader: IDWriteInMemoryFontFileLoader,
}

fn unpack(factory: &IDWriteFactory5, woff2: &[u8]) -> Result<Vec<u8>> {
    unsafe {
        let stream = factory.UnpackFontFile(DWRITE_CONTAINER_TYPE_WOFF2, woff2.as_ptr() as *const c_void, woff2.len() as u32)?;
        let size = stream.GetFileSize()?;
        let mut start: *mut c_void = std::ptr::null_mut();
        let mut context: *mut c_void = std::ptr::null_mut();
        stream.ReadFileFragment(&mut start, 0, size, &mut context)?;
        // Copied out: the stream's memory is only valid until the fragment is released.
        let bytes = std::slice::from_raw_parts(start as *const u8, size as usize).to_vec();
        stream.ReleaseFileFragment(context);
        Ok(bytes)
    }
}

fn strings(list: &IDWriteLocalizedStrings) -> String {
    unsafe {
        let n = list.GetCount();
        let mut out = Vec::new();
        for i in 0..n {
            let len = list.GetStringLength(i).unwrap_or(0) as usize;
            let mut buf = vec![0u16; len + 1];
            if list.GetString(i, &mut buf).is_ok() {
                out.push(String::from_utf16_lossy(&buf[..len]));
            }
        }
        out.join("/")
    }
}

pub fn load(factory: &IDWriteFactory5) -> Result<Fonts> {
    unsafe {
        let loader = factory.CreateInMemoryFontFileLoader()?;
        factory.RegisterFontFileLoader(&loader)?;
        let builder = factory.CreateFontSetBuilder()?;
        let mut bytes = Vec::new();
        for (name, woff2) in [("regular", LATIN1_REGULAR), ("semibold", LATIN1_SEMIBOLD), ("pi", PI_REGULAR)] {
            let raw = unpack(factory, woff2)?;
            observe!(Info, "font {name}: {} bytes woff2, {} bytes unpacked", woff2.len(), raw.len());
            // Heap buffers do not move when the Vec of them grows, so the pointer given to the loader stays valid.
            let file = loader.CreateInMemoryFontFileReference(factory, raw.as_ptr() as *const c_void, raw.len() as u32, None)?;
            builder.AddFontFile(&file)?;
            bytes.push(raw);
        }
        let set = builder.CreateFontSet()?;
        let collection = factory.CreateFontCollectionFromFontSet(&set)?;

        let n = collection.GetFontFamilyCount();
        for i in 0..n {
            let family = collection.GetFontFamily(i)?;
            let names = strings(&family.GetFamilyNames()?);
            let faces = family.GetFontCount();
            observe!(Info, "font collection family {i}: \"{names}\" with {faces} face(s)");
            for j in 0..faces {
                let font = family.GetFont(j)?;
                let face_names = strings(&font.GetFaceNames()?);
                observe!(Info, "  face {j}: \"{face_names}\" weight {}", font.GetWeight().0);
            }
        }
        Ok(Fonts { collection, _bytes: bytes, _loader: loader })
    }
}
