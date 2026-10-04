//! A window's pixels as the picture the bot gets (spec 24.4): in logical
//! pixels, without Windows' display scale, no larger than a limit, as a
//! JPEG.

use super::Picture;

/// The size `width` × `height` takes to fit within `max`, keeping its
/// shape; never larger than it was, never below one pixel.
pub fn fit(width: u32, height: u32, max: (u32, u32)) -> (u32, u32) {
    let (width, height) = (width.max(1), height.max(1));
    let scale = (f64::from(max.0) / f64::from(width))
        .min(f64::from(max.1) / f64::from(height))
        .min(1.0);
    let side = |length: u32| ((f64::from(length) * scale).round() as u32).max(1);
    (side(width), side(height))
}

/// `rgb`, `width` × `height`, shrunk to `to`: each pixel is the average of
/// the ones it covers.
pub fn shrink(rgb: &[u8], width: u32, height: u32, to: (u32, u32)) -> Vec<u8> {
    let (width, height) = (width as usize, height as usize);
    let (to_width, to_height) = (to.0 as usize, to.1 as usize);
    if (to_width, to_height) == (width, height) {
        return rgb.to_vec();
    }
    let mut out = Vec::with_capacity(to_width * to_height * 3);
    for y in 0..to_height {
        let top = y * height / to_height;
        let bottom = ((y + 1) * height / to_height).max(top + 1);
        for x in 0..to_width {
            let left = x * width / to_width;
            let right = ((x + 1) * width / to_width).max(left + 1);
            let mut sum = [0u64; 3];
            for row in top..bottom {
                for column in left..right {
                    let at = (row * width + column) * 3;
                    for (channel, total) in sum.iter_mut().enumerate() {
                        *total += u64::from(rgb[at + channel]);
                    }
                }
            }
            let count = ((bottom - top) * (right - left)) as u64;
            out.extend(sum.map(|total| (total / count) as u8));
        }
    }
    out
}

/// The picture of a window whose pixels are `rgb`, `width` × `height`, on
/// a display scaled `scale` times (1.5 at 150%).
pub fn picture(
    rgb: &[u8],
    width: u32,
    height: u32,
    scale: f64,
    max: (u32, u32),
    quality: u8,
) -> Picture {
    let logical = |length: u32| ((f64::from(length) / scale.max(1.0)).round() as u32).max(1);
    let (to_width, to_height) = fit(logical(width), logical(height), max);
    let pixels = shrink(rgb, width, height, (to_width, to_height));
    let mut jpeg = Vec::new();
    let encoder = jpeg_encoder::Encoder::new(&mut jpeg, quality);
    // A picture this size always encodes; an empty one means nothing to see.
    if encoder
        .encode(
            &pixels,
            to_width as u16,
            to_height as u16,
            jpeg_encoder::ColorType::Rgb,
        )
        .is_err()
    {
        jpeg.clear();
    }
    Picture {
        jpeg,
        width: to_width,
        height: to_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_fits_the_limit_in_its_shape_and_never_grows() {
        assert_eq!(fit(3200, 1800, (1600, 1200)), (1600, 900));
        assert_eq!(fit(1000, 2400, (1600, 1200)), (500, 1200));
        assert_eq!(fit(800, 600, (1600, 1200)), (800, 600));
        assert_eq!(fit(0, 0, (1600, 1200)), (1, 1));
    }

    #[test]
    fn shrinking_averages_the_pixels_it_covers() {
        // Two pixels wide, one tall: black and white become grey.
        let rgb = [0, 0, 0, 255, 255, 255];
        assert_eq!(shrink(&rgb, 2, 1, (1, 1)), [127, 127, 127]);
        assert_eq!(shrink(&rgb, 2, 1, (2, 1)), rgb);
    }

    #[test]
    fn the_picture_drops_the_display_scale_and_is_a_jpeg() {
        let rgb = vec![200u8; 300 * 150 * 3];
        // A window 300 × 150 on a display at 150%: 200 × 100 to the bot.
        let picture = picture(&rgb, 300, 150, 1.5, (1600, 1200), 70);
        assert_eq!((picture.width, picture.height), (200, 100));
        assert_eq!(&picture.jpeg[..2], [0xFF, 0xD8]);
    }
}
