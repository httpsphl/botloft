//! A picture of the owner's whole screen for a bot with the whole desktop
//! (spec 24.4, D6): every window as the owner sees it, the ones that are
//! never granted (spec 24.3) covered in black.

use super::{DesktopError, Picture};

/// Paints each of `covered` (left, top, width and height on the screen)
/// black in `rgb`, a picture `width` × `height` of the screen from
/// `origin`; what falls outside the picture is left out.
pub fn cover(rgb: &mut [u8], width: u32, height: u32, origin: (i32, i32), covered: &[[i32; 4]]) {
    let (width, height) = (i64::from(width), i64::from(height));
    for &[left, top, across, down] in covered {
        let from_x = (i64::from(left) - i64::from(origin.0)).clamp(0, width);
        let to_x = (i64::from(left) + i64::from(across) - i64::from(origin.0)).clamp(0, width);
        let from_y = (i64::from(top) - i64::from(origin.1)).clamp(0, height);
        let to_y = (i64::from(top) + i64::from(down) - i64::from(origin.1)).clamp(0, height);
        for row in from_y..to_y {
            let start = ((row * width + from_x) * 3) as usize;
            let end = ((row * width + to_x) * 3) as usize;
            rgb[start..end].fill(0);
        }
    }
}

/// The whole screen, with every window never granted covered, in logical
/// pixels up to [`super::PICTURE_MAX`].
pub fn screen_picture() -> Result<Picture, DesktopError> {
    #[cfg(windows)]
    {
        use super::{QUALITY, never, shrink, windows};
        let covered: Vec<[i32; 4]> = windows::every_shown()?
            .iter()
            .filter(|window| !window.minimized && never(window).is_some())
            .filter_map(|window| windows::frame(window.id).ok())
            .collect();
        let (rgb, width, height, scale) = windows::capture_screen(&covered)?;
        Ok(shrink::picture(
            &rgb,
            width,
            height,
            scale,
            super::PICTURE_MAX,
            QUALITY,
        ))
    }
    #[cfg(not(windows))]
    {
        Err(DesktopError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::cover;

    #[test]
    fn covered_windows_turn_black_and_only_inside_the_picture() {
        // A 4 × 3 screen from (-2, 10), all white.
        let mut rgb = vec![255u8; 4 * 3 * 3];
        // A window at (-1, 11), 2 × 5: columns 1 and 2, rows 1 and 2.
        cover(&mut rgb, 4, 3, (-2, 10), &[[-1, 11, 2, 5]]);
        let black: Vec<(usize, usize)> = (0..3)
            .flat_map(|row| (0..4).map(move |column| (row, column)))
            .filter(|&(row, column)| rgb[(row * 4 + column) * 3] == 0)
            .collect();
        assert_eq!(black, vec![(1, 1), (1, 2), (2, 1), (2, 2)]);
        // Off the screen, nothing.
        let mut rgb = vec![255u8; 4 * 3 * 3];
        cover(&mut rgb, 4, 3, (0, 0), &[[10, 10, 5, 5], [-9, -9, 2, 2]]);
        assert!(rgb.iter().all(|&byte| byte == 255));
    }
}
