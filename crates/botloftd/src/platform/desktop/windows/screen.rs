//! A picture of the owner's whole screen (spec 24.4): every monitor, as the
//! owner sees it, with the windows that are never granted covered in black.
//! The daemon's own outline and notice are kept out of it (they are
//! excluded from capture).

use std::ffi::c_void;

use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CAPTUREBLT, CreateCompatibleBitmap,
    CreateCompatibleDC, DIB_RGB_COLORS, GetDC, GetDIBits, ReleaseDC, SRCCOPY, SelectObject,
};
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

use super::super::{DesktopError, cover};
use super::capture::{Canvas, RealPixels, system};

/// Windows' unscaled DPI.
const BASE_DPI: f64 = 96.0;

/// The whole screen's pixels as RGB, its width and height, and the display
/// scale, with each of `covered` (left, top, width and height, in real
/// pixels on the screen) painted black.
pub fn capture_screen(covered: &[[i32; 4]]) -> Result<(Vec<u8>, u32, u32, f64), DesktopError> {
    let _real = RealPixels::new();
    // SAFETY: plain queries.
    let (left, top, width, height) = unsafe {
        (
            GetSystemMetrics(SM_XVIRTUALSCREEN),
            GetSystemMetrics(SM_YVIRTUALSCREEN),
            GetSystemMetrics(SM_CXVIRTUALSCREEN),
            GetSystemMetrics(SM_CYVIRTUALSCREEN),
        )
    };
    if width <= 0 || height <= 0 {
        return Err(DesktopError::Locked);
    }
    // SAFETY: each object made here is freed by `Canvas`.
    let canvas = unsafe {
        let screen = GetDC(None);
        let memory = CreateCompatibleDC(Some(screen));
        let bitmap = CreateCompatibleBitmap(screen, width, height);
        if memory.is_invalid() || bitmap.is_invalid() {
            ReleaseDC(None, screen);
            return Err(system("no memory for the picture"));
        }
        let previous = SelectObject(memory, bitmap.into());
        Canvas {
            screen,
            memory,
            bitmap,
            previous,
        }
    };
    // SAFETY: copies the screen into our bitmap of the same size. A locked
    // screen or the secure desktop refuses.
    let copied = unsafe {
        BitBlt(
            canvas.memory,
            0,
            0,
            width,
            height,
            Some(canvas.screen),
            left,
            top,
            SRCCOPY | CAPTUREBLT,
        )
    };
    if copied.is_err() {
        return Err(DesktopError::Locked);
    }
    let mut info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            // Negative: rows from the top down.
            biHeight: -height,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bgra = vec![0u8; width as usize * height as usize * 4];
    // SAFETY: the buffer holds the whole bitmap at 32 bits a pixel.
    let rows = unsafe {
        GetDIBits(
            canvas.memory,
            canvas.bitmap,
            0,
            height as u32,
            Some(bgra.as_mut_ptr().cast::<c_void>()),
            &mut info,
            DIB_RGB_COLORS,
        )
    };
    drop(canvas);
    if rows == 0 {
        return Err(system("the picture could not be read"));
    }
    let (pixels, _) = bgra.as_chunks::<4>();
    let mut rgb = Vec::with_capacity(pixels.len() * 3);
    for [blue, green, red, _] in pixels {
        rgb.extend([*red, *green, *blue]);
    }
    let (width, height) = (width as u32, height as u32);
    cover(&mut rgb, width, height, (left, top), covered);
    // SAFETY: a plain query.
    let dpi = unsafe { GetDpiForSystem() };
    let scale = if dpi == 0 {
        1.0
    } else {
        f64::from(dpi) / BASE_DPI
    };
    Ok((rgb, width, height, scale))
}
