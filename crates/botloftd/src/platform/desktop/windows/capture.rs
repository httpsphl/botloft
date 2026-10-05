//! A picture of one window with `PrintWindow` (spec 24.4): only the
//! window, even behind others, without the invisible resize border Windows
//! keeps around it.

use std::ffi::c_void;

use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HBITMAP, HDC, HGDIOBJ, ReleaseDC,
    SelectObject,
};
use windows::Win32::Storage::Xps::{PRINT_WINDOW_FLAGS, PrintWindow};
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow,
    SetThreadDpiAwarenessContext,
};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, IsIconic, IsWindow};

use super::super::DesktopError;
use super::handle;

/// Asks the window to draw itself fully, as DirectX and browser windows
/// need.
const RENDER_FULL_CONTENT: PRINT_WINDOW_FLAGS = PRINT_WINDOW_FLAGS(2);
/// Windows' unscaled DPI.
const BASE_DPI: f64 = 96.0;

/// Sizes in real pixels on this thread while it lives, then back.
struct RealPixels(DPI_AWARENESS_CONTEXT);

impl RealPixels {
    fn new() -> Self {
        // SAFETY: changes only this thread's DPI awareness, restored on drop.
        Self(unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) })
    }
}

impl Drop for RealPixels {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: puts back the awareness this thread had.
            unsafe { SetThreadDpiAwarenessContext(self.0) };
        }
    }
}

/// Frees the drawing objects in the right order when it goes.
struct Canvas {
    screen: HDC,
    memory: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
}

impl Drop for Canvas {
    fn drop(&mut self) {
        // SAFETY: each object was made by `capture` and is freed once.
        unsafe {
            SelectObject(self.memory, self.previous);
            let _ = DeleteObject(self.bitmap.into());
            let _ = DeleteDC(self.memory);
            ReleaseDC(None, self.screen);
        }
    }
}

fn system(what: &str) -> DesktopError {
    DesktopError::System(what.to_owned())
}

/// Where window `id`'s frame is on the screen, in real pixels, as the
/// controls' places are: left, top, width and height.
pub fn frame(id: u64) -> Result<[i32; 4], DesktopError> {
    let hwnd = handle(id);
    let _real = RealPixels::new();
    let mut frame = RECT::default();
    // SAFETY: `frame` is a valid out-pointer.
    unsafe { GetWindowRect(hwnd, &mut frame) }.map_err(|_| DesktopError::Gone)?;
    // SAFETY: `frame` is a RECT of the size given. Without DWM the window
    // rectangle is the frame.
    let _ = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            std::ptr::from_mut(&mut frame).cast::<c_void>(),
            size_of::<RECT>() as u32,
        )
    };
    Ok([
        frame.left,
        frame.top,
        frame.right - frame.left,
        frame.bottom - frame.top,
    ])
}

/// The window `id`'s pixels as RGB, its width and height, and the display
/// scale it is on (1.5 at 150%).
pub fn capture(id: u64) -> Result<(Vec<u8>, u32, u32, f64), DesktopError> {
    let hwnd = handle(id);
    // SAFETY: plain queries on a window handle.
    unsafe {
        if !IsWindow(Some(hwnd)).as_bool() {
            return Err(DesktopError::Gone);
        }
        if IsIconic(hwnd).as_bool() {
            return Err(DesktopError::Minimized);
        }
    }
    let _real = RealPixels::new();
    let mut outer = RECT::default();
    // SAFETY: `outer` is a valid out-pointer.
    unsafe { GetWindowRect(hwnd, &mut outer) }.map_err(|_| DesktopError::Gone)?;
    let mut frame = outer;
    // SAFETY: `frame` is a RECT of the size given. Without DWM the window
    // rectangle is the frame.
    let _ = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            std::ptr::from_mut(&mut frame).cast::<c_void>(),
            size_of::<RECT>() as u32,
        )
    };
    let full = (outer.right - outer.left, outer.bottom - outer.top);
    if full.0 <= 0 || full.1 <= 0 {
        return Err(DesktopError::Minimized);
    }
    // SAFETY: plain query on a window handle.
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    let scale = if dpi == 0 {
        1.0
    } else {
        f64::from(dpi) / BASE_DPI
    };

    // SAFETY: each object made here is freed by `Canvas`.
    let canvas = unsafe {
        let screen = GetDC(None);
        let memory = CreateCompatibleDC(Some(screen));
        let bitmap = CreateCompatibleBitmap(screen, full.0, full.1);
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
    // SAFETY: the window draws into our bitmap.
    if !unsafe { PrintWindow(hwnd, canvas.memory, RENDER_FULL_CONTENT) }.as_bool() {
        return Err(system("the window did not draw itself"));
    }
    let mut info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: full.0,
            // Negative: rows from the top down.
            biHeight: -full.1,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut bgra = vec![0u8; full.0 as usize * full.1 as usize * 4];
    // SAFETY: the buffer holds the whole bitmap at 32 bits a pixel.
    let rows = unsafe {
        GetDIBits(
            canvas.memory,
            canvas.bitmap,
            0,
            full.1 as u32,
            Some(bgra.as_mut_ptr().cast::<c_void>()),
            &mut info,
            DIB_RGB_COLORS,
        )
    };
    drop(canvas);
    if rows == 0 {
        return Err(system("the picture could not be read"));
    }
    // Only the frame, without the invisible border.
    let left = (frame.left - outer.left).clamp(0, full.0) as usize;
    let top = (frame.top - outer.top).clamp(0, full.1) as usize;
    let right = (frame.right - outer.left).clamp(left as i32 + 1, full.0) as usize;
    let bottom = (frame.bottom - outer.top).clamp(top as i32 + 1, full.1) as usize;
    let mut rgb = Vec::with_capacity((right - left) * (bottom - top) * 3);
    for row in top..bottom {
        let start = (row * full.0 as usize + left) * 4;
        let (pixels, _) = bgra[start..start + (right - left) * 4].as_chunks::<4>();
        for [blue, green, red, _] in pixels {
            rgb.extend([*red, *green, *blue]);
        }
    }
    Ok((rgb, (right - left) as u32, (bottom - top) as u32, scale))
}
