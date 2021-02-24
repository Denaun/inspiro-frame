// mod color;

use image::{imageops::ColorMap, Pixel, Rgb, RgbImage};
use imageproc::{
    contrast::otsu_level,
    map::{blue_channel, green_channel, map_colors, red_channel},
};

/// Convert an image to Black, White, and Red.
///
/// The result is still RGB, but only uses three colors.
pub fn to_bwr(mut image: image::RgbImage, dither: bool) -> image::RgbImage {
    let algo = BwrHeuristic::new(&image);
    if dither {
        image::imageops::dither(&mut image, &algo)
    } else {
        for pixel in image.pixels_mut() {
            algo.map_color(pixel);
        }
    }
    image
}
/// Convert an image to Black, White, and Red.
///
/// The result is two gray images, one for the black channel and one for the red
/// one.
///
/// Values of 255 indicate white for both images. Although the images use 8bpp,
/// only two values (0 and 255) are used.
pub fn to_bwr_split(
    mut image: image::RgbImage,
    dither: bool,
) -> (image::GrayImage, image::GrayImage) {
    image = to_bwr(image, dither);
    let (width, height) = image.dimensions();
    let black = image
        .pixels()
        .map(|p| match p.channels() {
            [0, 0, 0] => 0,
            _ => 255,
        })
        .collect::<Vec<_>>();
    let red = image
        .pixels()
        .map(|p| match p.channels() {
            [255, 0, 0] => 0,
            _ => 255,
        })
        .collect::<Vec<_>>();
    (
        image::GrayImage::from_raw(width, height, black).unwrap(),
        image::GrayImage::from_raw(width, height, red).unwrap(),
    )
}

/// Calculate Hue, Saturation and Value of the given color.
///
/// All values are between 0.0 and 1.0: Hue is in turns, Saturation and Value
/// are in percentage.
fn hsv(red: u8, green: u8, blue: u8) -> (f32, f32, f32) {
    let c_max = red.max(green).max(blue);
    let c_min = red.min(green).min(blue);
    let delta = c_max - c_min;
    let hue = if delta == 0 {
        0.0
    } else {
        (if c_max == red {
            ((green as f32 - blue as f32) / delta as f32).rem_euclid(6.0)
        } else if c_max == green {
            ((blue as f32 - red as f32) / delta as f32) + 2.0
        } else {
            assert_eq!(c_max, blue);
            ((red as f32 - green as f32) / delta as f32) + 4.0
        }) / 6.0
    };
    let saturation = if c_max == 0 {
        0.0
    } else {
        delta as f32 / c_max as f32
    };
    let value = from_unorm8(c_max);
    (hue, saturation, value)
}

/// Unsigned Normalized integer conversion.
fn to_unorm8(v: f32) -> u8 {
    if v.is_nan() {
        0
    } else {
        (v.clamp(0.0, 1.0) * u8::MAX as f32).round() as u8
    }
}
/// Unsigned Normalized integer conversion.
fn from_unorm8(v: u8) -> f32 {
    v as f32 / u8::MAX as f32
}

/// Fold a Hue value (red = 0.0) in turns such that reddish values are towards 1.
fn fold_red(h: f32) -> f32 {
    (h - 0.5).abs() * 2.0
}

/// Enumerator to simplify handling 3-color information.
enum Bwr {
    Black,
    White,
    Red,
}
/// Implement ColorMap for a type that can map RGB pixels to [`Bwr`].
macro_rules! bwr_color_map {
    ($t:ident) => {
        impl image::imageops::ColorMap for $t {
            type Color = Rgb<u8>;

            fn index_of(&self, color: &Self::Color) -> usize {
                match self.to_bwr(color) {
                    Bwr::Black => 0,
                    Bwr::White => 1,
                    Bwr::Red => 2,
                }
            }
            fn lookup(&self, index: usize) -> Option<Self::Color> {
                match index {
                    0 => Some([0, 0, 0].into()),
                    1 => Some([255, 255, 255].into()),
                    2 => Some([255, 0, 0].into()),
                    _ => None,
                }
            }
            fn has_lookup(&self) -> bool {
                true
            }
            fn map_color(&self, color: &mut Self::Color) {
                *color = self.lookup(self.index_of(color)).unwrap();
            }
        }
    };
}

/// Try to map colors to Black White and Red based on thresholds on HSV.
///
/// The idea is to use the value to distinguish between black and colored
/// (white/red) pixels, and then use hue and saturation to identify red pixels.
///
/// Hue is folded so that high values correspond to reddish bits. Thresholds are
/// found on each of Hue, Saturation, and Value using Otsu.
#[derive(Debug)]
struct BwrHeuristic {
    h: f32,
    s: f32,
    v: f32,
}
impl BwrHeuristic {
    fn new(image: &RgbImage) -> Self {
        let hsv = map_colors(image, |p| {
            let (h, s, v) = hsv(p[0], p[1], p[2]);
            Rgb([to_unorm8(fold_red(h)), to_unorm8(s), to_unorm8(v)])
        });
        Self {
            h: from_unorm8(otsu_level(&red_channel(&hsv))),
            s: from_unorm8(otsu_level(&green_channel(&hsv))),
            v: from_unorm8(otsu_level(&blue_channel(&hsv))),
        }
    }
    fn to_bwr(&self, color: &Rgb<u8>) -> Bwr {
        let (h, s, v) = hsv(color[0], color[1], color[2]);
        let h = fold_red(h);
        if v > self.v {
            if h > self.h && s > self.s {
                Bwr::Red
            } else {
                Bwr::White
            }
        } else {
            Bwr::Black
        }
    }
}
bwr_color_map!(BwrHeuristic);

#[cfg(test)]
mod tests {
    use float_eq::assert_float_eq;

    #[test]
    fn hsv() {
        let (h, s, v) = super::hsv(255, 71, 99);
        assert_float_eq!(h * 360.0, 350.869_57, abs <= 0.000_01);
        assert_float_eq!(s, 0.722, abs <= 0.001);
        assert_float_eq!(v, 1.0, abs <= 0.000_1);
    }
}
