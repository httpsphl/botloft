//! The bot's cursor on the owner's screen (spec 24.9), drawn in the
//! outline's window: an arrow in the bot's color with its name beside it,
//! gliding to the point of each action, a ring where it clicks and dots
//! while it types. The owner's own mouse never moves for it.

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{COLORREF, POINT, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateFontW, CreatePen, CreateSolidBrush, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DeleteObject,
    DrawTextW, Ellipse, GetStockObject, HDC, HGDIOBJ, NONANTIALIASED_QUALITY, NULL_BRUSH, PS_SOLID,
    Polygon, RoundRect, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
};
use windows::core::w;

use super::super::ScreenCursor;

/// How long the cursor takes to glide to a new point.
const GLIDE: Duration = Duration::from_millis(350);
/// How long the ring of a click grows.
const RIPPLE: Duration = Duration::from_millis(500);
/// The arrow, pointing at (0, 0), at 96 DPI.
const ARROW: [(i32, i32); 7] = [
    (0, 0),
    (0, 17),
    (4, 13),
    (7, 20),
    (10, 19),
    (7, 12),
    (12, 12),
];

/// Where the cursor goes and where it came from, in fractions of the
/// window.
struct Motion {
    from: (f64, f64),
    to: (f64, f64),
    started: Instant,
    /// The action it shows, by when the bot did it.
    at: i64,
    click: bool,
}

static MOTION: Mutex<Option<Motion>> = Mutex::new(None);

fn ease((from, to): ((f64, f64), (f64, f64)), done: f64) -> (f64, f64) {
    let done = 1.0 - (1.0 - done.clamp(0.0, 1.0)).powi(3);
    (
        from.0 + (to.0 - from.0) * done,
        from.1 + (to.1 - from.1) * done,
    )
}

fn now_at(motion: &Motion, now: Instant) -> (f64, f64) {
    let done = now.duration_since(motion.started).as_secs_f64() / GLIDE.as_secs_f64();
    ease((motion.from, motion.to), done)
}

/// Takes in the cursor to show; `true` while it moves or its ring grows,
/// for the window to draw again soon.
pub(super) fn follow(cursor: Option<&ScreenCursor>) -> bool {
    let mut motion = MOTION.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(cursor) = cursor else {
        *motion = None;
        return false;
    };
    let now = Instant::now();
    if motion.as_ref().is_none_or(|motion| motion.at != cursor.at) {
        let to = (cursor.x, cursor.y);
        let from = motion.as_ref().map_or(to, |motion| now_at(motion, now));
        *motion = Some(Motion {
            from,
            to,
            started: now,
            at: cursor.at,
            click: cursor.click,
        });
    }
    moving(now)
}

/// Whether the cursor still glides or its ring still grows.
pub(super) fn moving(now: Instant) -> bool {
    MOTION
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .is_some_and(|motion| now.duration_since(motion.started) < GLIDE + RIPPLE)
}

fn colorref((red, green, blue): (u8, u8, u8)) -> COLORREF {
    COLORREF(u32::from(red) | (u32::from(green) << 8) | (u32::from(blue) << 16))
}

/// Draws the cursor in `inside`, the app's frame within the window.
///
/// # Safety
/// `dc` is the outline window's, between `BeginPaint` and `EndPaint`.
pub(super) unsafe fn paint(
    dc: HDC,
    inside: RECT,
    scale: impl Fn(i32) -> i32,
    color: (u8, u8, u8),
    cursor: &ScreenCursor,
) {
    let now = Instant::now();
    let (point, ripple) = {
        let motion = MOTION.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(motion) = motion.as_ref() else {
            return;
        };
        let since = now.duration_since(motion.started);
        let ripple = (motion.click && since >= GLIDE && since < GLIDE + RIPPLE)
            .then(|| (since - GLIDE).as_secs_f64() / RIPPLE.as_secs_f64());
        (now_at(motion, now), ripple)
    };
    let x = inside.left + (point.0 * f64::from(inside.right - inside.left)) as i32;
    let y = inside.top + (point.1 * f64::from(inside.bottom - inside.top)) as i32;
    // SAFETY: drawing on the caller's paint DC; every object made here is
    // selected out and freed before the end.
    unsafe {
        let fill = CreateSolidBrush(colorref(color));
        let white = CreatePen(PS_SOLID, scale(1).max(1), COLORREF(0x00FF_FFFF));
        let old_brush: HGDIOBJ = SelectObject(dc, fill.into());
        let old_pen: HGDIOBJ = SelectObject(dc, white.into());
        if let Some(done) = ripple {
            let ring = CreatePen(PS_SOLID, scale(3), colorref(color));
            SelectObject(dc, ring.into());
            SelectObject(dc, GetStockObject(NULL_BRUSH));
            let radius = scale(8) + (f64::from(scale(15)) * done) as i32;
            let _ = Ellipse(dc, x - radius, y - radius, x + radius, y + radius);
            SelectObject(dc, white.into());
            SelectObject(dc, fill.into());
            let _ = DeleteObject(ring.into());
        }
        let arrow: Vec<POINT> = ARROW
            .iter()
            .map(|&(dx, dy)| POINT {
                x: x + scale(dx),
                y: y + scale(dy),
            })
            .collect();
        let _ = Polygon(dc, &arrow);
        // The bot's name in a pill beside it, dots while it types.
        let mut label: Vec<u16> = if cursor.typing {
            format!("{} \u{2022}\u{2022}\u{2022}", cursor.name)
        } else {
            cursor.name.clone()
        }
        .encode_utf16()
        .collect();
        let width = scale(14) + scale(7) * label.len().min(40) as i32;
        let (left, top) = (x + scale(14), y + scale(16));
        let pill = scale(20);
        let _ = RoundRect(dc, left, top, left + width, top + pill, pill, pill);
        let font = CreateFontW(
            -scale(12),
            0,
            0,
            0,
            600,
            0,
            0,
            0,
            Default::default(),
            Default::default(),
            Default::default(),
            NONANTIALIASED_QUALITY,
            0,
            w!("Segoe UI"),
        );
        let old_font = SelectObject(dc, font.into());
        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, COLORREF(0x00FF_FFFF));
        let mut place = RECT {
            left,
            top,
            right: left + width,
            bottom: top + pill,
        };
        DrawTextW(
            dc,
            &mut label,
            &mut place,
            DT_CENTER | DT_VCENTER | DT_SINGLELINE,
        );
        SelectObject(dc, old_font);
        SelectObject(dc, old_pen);
        SelectObject(dc, old_brush);
        for object in [font.into(), white.into(), fill.into()] {
            let _ = DeleteObject(object);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cursor_glides_and_lands_where_the_bot_acted() {
        let start = ((0.0, 0.0), (1.0, 0.5));
        assert_eq!(ease(start, 0.0), (0.0, 0.0));
        assert_eq!(ease(start, 1.0), (1.0, 0.5));
        assert_eq!(ease(start, 2.0), (1.0, 0.5), "and stays");
        let (x, _) = ease(start, 0.5);
        assert!(x > 0.5, "fast first, slow at the end: {x}");
    }
}
