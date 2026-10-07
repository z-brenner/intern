//! Pages and line crops as the networks take them: planar floats,
//! normalized, at the sizes each network wants.
//!
//! Everything is sampled straight from the page's RGB pixels into the
//! input tensor. Nothing is encoded, and no intermediate image is made for
//! a resize that the sampling can do on the way.

use image::RgbImage;

use super::geometry::{Point, Quad};

/// How large the detector's input may be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetectionSize {
    /// The longer side at most `max`; a page whose longer side is under
    /// `min` is enlarged to it, so the strokes of a low-resolution scan are
    /// a few pixels wide where the detector looks for them.
    LongSide { max: u32, min: u32 },
    /// The page as it is, up to this many pixels on the longer side.
    Native { max_side: u32 },
}

/// The detector's input size for a `width` x `height` page: scaled by the
/// cap, then each side rounded to a multiple of 32, at least 32.
pub fn detection_input_size(width: u32, height: u32, size: DetectionSize) -> (u32, u32) {
    let long = width.max(height) as f32;
    let ratio = match size {
        DetectionSize::LongSide { max, .. } | DetectionSize::Native { max_side: max }
            if long > max as f32 =>
        {
            max as f32 / long
        }
        DetectionSize::LongSide { min, .. } if long < min as f32 => min as f32 / long,
        _ => 1.0,
    };
    let round = |side: u32| (((side as f32 * ratio) / 32.0).round() as u32 * 32).max(32);
    (round(width), round(height))
}

/// Mean and spread the detector and the orientation classifier were
/// trained with, per input channel.
const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

/// The page as the detector takes it: `3 x height x width`, resized by
/// bilinear sampling, channels in the blue-green-red order the models were
/// trained on, each normalized by the ImageNet statistics in that order.
pub fn detection_input(page: &RgbImage, width: u32, height: u32) -> Vec<f32> {
    let plane = (width * height) as usize;
    let mut input = vec![0.0_f32; plane * 3];
    let scale_x = page.width() as f32 / width as f32;
    let scale_y = page.height() as f32 / height as f32;
    let factors: [(f32, f32); 3] = std::array::from_fn(|channel| {
        (
            1.0 / (255.0 * IMAGENET_STD[channel]),
            IMAGENET_MEAN[channel] / IMAGENET_STD[channel],
        )
    });
    let columns: Vec<(u32, u32, f32)> = (0..width)
        .map(|x| source_taps(x, scale_x, page.width()))
        .collect();
    for y in 0..height {
        let (y0, y1, fy) = source_taps(y, scale_y, page.height());
        let row = (y * width) as usize;
        for (x, &(x0, x1, fx)) in columns.iter().enumerate() {
            let pixel = bilinear(page, x0, x1, fx, y0, y1, fy);
            // Blue first: the page is RGB, the models read BGR.
            for (channel, value) in [pixel[2], pixel[1], pixel[0]].into_iter().enumerate() {
                let (scale, offset) = factors[channel];
                input[channel * plane + row + x] = value * scale - offset;
            }
        }
    }
    input
}

/// The two source pixels a destination pixel falls between, and how far
/// along it is, with pixel centres aligned the way OpenCV aligns them.
fn source_taps(destination: u32, scale: f32, limit: u32) -> (u32, u32, f32) {
    let source = ((destination as f32 + 0.5) * scale - 0.5).max(0.0);
    let low = (source.floor() as u32).min(limit - 1);
    let high = (low + 1).min(limit - 1);
    (low, high, source - low as f32)
}

fn bilinear(page: &RgbImage, x0: u32, x1: u32, fx: f32, y0: u32, y1: u32, fy: f32) -> [f32; 3] {
    let a = page.get_pixel(x0, y0).0;
    let b = page.get_pixel(x1, y0).0;
    let c = page.get_pixel(x0, y1).0;
    let d = page.get_pixel(x1, y1).0;
    std::array::from_fn(|channel| {
        let top = f32::from(a[channel]) * (1.0 - fx) + f32::from(b[channel]) * fx;
        let bottom = f32::from(c[channel]) * (1.0 - fx) + f32::from(d[channel]) * fx;
        top * (1.0 - fy) + bottom * fy
    })
}

/// The page's colour at any point, bilinear, with the edge pixels repeated
/// beyond the page.
fn sample(page: &RgbImage, point: Point) -> [f32; 3] {
    let x = point.x.clamp(0.0, (page.width() - 1) as f32);
    let y = point.y.clamp(0.0, (page.height() - 1) as f32);
    let (x0, y0) = (x.floor() as u32, y.floor() as u32);
    let (x1, y1) = (
        (x0 + 1).min(page.width() - 1),
        (y0 + 1).min(page.height() - 1),
    );
    bilinear(page, x0, x1, x - x0 as f32, y0, y1, y - y0 as f32)
}

/// The height every line is recognized at.
pub const RECOGNITION_HEIGHT: u32 = 48;

/// A line cut out of the page along its own axis, `height` pixels tall and
/// as wide as its proportions make it, in RGB.
///
/// A line taller than it is long by half again is read as running down the
/// page: it is cut out turned a quarter counter-clockwise, which is how the
/// reference pipeline reads vertical text. Each output pixel averages a
/// small grid of samples when the line is being shrunk, so thin strokes in
/// large type are not lost between samples.
#[derive(Clone, Debug, PartialEq)]
pub struct LineCrop {
    pub width: u32,
    pub height: u32,
    /// Row-major RGB, `width * height * 3` values, 0-255.
    pub pixels: Vec<f32>,
}

/// The longest crop, in pixels at recognition height, a line is cut to.
/// A line longer than this is squeezed to fit; a full-width line of body
/// text at 300 DPI is about a third of it.
pub const MAX_LINE_WIDTH: u32 = 3_200;

/// Where a line's crop starts, the direction its rows and its columns run,
/// and its long and short sides in page pixels.
fn line_frame(quad: &Quad) -> (Point, Point, Point, f32, f32) {
    let [a, b, c, d] = quad.0;
    let along = |p: Point, q: Point| ((q.x - p.x).powi(2) + (q.y - p.y).powi(2)).sqrt();
    let source_width = along(a, b).max(along(d, c)).max(1.0);
    let source_height = along(a, d).max(along(b, c)).max(1.0);
    if source_height >= source_width * 1.5 {
        (b, c, a, source_height, source_width)
    } else {
        (a, b, d, source_width, source_height)
    }
}

/// How wide [`crop_line`] cuts a line at `height`, without cutting it.
pub fn line_width(quad: &Quad, height: u32) -> u32 {
    let (_, _, _, long, short) = line_frame(quad);
    ((height as f32 * long / short).ceil() as u32).clamp(1, MAX_LINE_WIDTH)
}

pub fn crop_line(page: &RgbImage, quad: &Quad, height: u32) -> LineCrop {
    // Corner the crop starts from, the direction its rows run, and the
    // direction its columns run, in page pixels per crop pixel.
    let (origin, row_end, column_end, long, short) = line_frame(quad);
    let width = line_width(quad, height);
    let step_x = Point::new(
        (row_end.x - origin.x) / width as f32,
        (row_end.y - origin.y) / width as f32,
    );
    let step_y = Point::new(
        (column_end.x - origin.x) / height as f32,
        (column_end.y - origin.y) / height as f32,
    );
    let shrink = (short / height as f32).max(long / width as f32);
    let taps = (shrink.ceil() as u32).clamp(1, 4);
    let mut pixels = Vec::with_capacity((width * height * 3) as usize);
    let weight = 1.0 / (taps * taps) as f32;
    for y in 0..height {
        for x in 0..width {
            let mut sum = [0.0_f32; 3];
            for ty in 0..taps {
                for tx in 0..taps {
                    let u = x as f32 + (tx as f32 + 0.5) / taps as f32;
                    let v = y as f32 + (ty as f32 + 0.5) / taps as f32;
                    let point = Point::new(
                        origin.x + step_x.x * u + step_y.x * v - 0.5,
                        origin.y + step_x.y * u + step_y.y * v - 0.5,
                    );
                    let value = sample(page, point);
                    for channel in 0..3 {
                        sum[channel] += value[channel];
                    }
                }
            }
            pixels.extend(sum.map(|value| value * weight));
        }
    }
    LineCrop {
        width,
        height,
        pixels,
    }
}

/// A batch of crops as the recognizer takes them: `n x 3 x height x width`,
/// each crop left-aligned and the rest zero, channels blue-green-red,
/// scaled to -1..1. `width` is the widest crop, and never less than
/// `min_width`.
pub fn recognition_batch(crops: &[&LineCrop], min_width: u32) -> (Vec<f32>, u32) {
    let height = crops.first().map_or(RECOGNITION_HEIGHT, |crop| crop.height);
    let width = crops
        .iter()
        .map(|crop| crop.width)
        .max()
        .unwrap_or(0)
        .max(min_width);
    let plane = (width * height) as usize;
    let mut batch = vec![0.0_f32; crops.len() * plane * 3];
    for (index, crop) in crops.iter().enumerate() {
        let base = index * plane * 3;
        for y in 0..crop.height {
            for x in 0..crop.width {
                let source = ((y * crop.width + x) * 3) as usize;
                let destination = (y * width + x) as usize;
                for channel in 0..3 {
                    let value = crop.pixels[source + 2 - channel];
                    batch[base + channel * plane + destination] = value / 127.5 - 1.0;
                }
            }
        }
    }
    (batch, width)
}

/// How many times longer than wide a page may be before only its central
/// square is shrunk for the orientation classifier.
const MAX_ORIENTATION_ASPECT: u32 = 8;

/// The square the orientation classifier looks at: the page shrunk so its
/// shorter side is 256, by area averaging, then its centre 224 x 224, RGB,
/// ImageNet-normalized, `3 x 224 x 224`.
pub fn orientation_input(page: &RgbImage) -> Vec<f32> {
    const SHORT: u32 = 256;
    const CROP: u32 = 224;
    let (width, height) = page.dimensions();
    // A page many times longer than it is wide - a till roll, or a crafted
    // strip of one pixel by a million - would be shrunk to a thumbnail as
    // long as itself: 196 GB for that strip. Only its centre reaches the
    // classifier, so only its central square is shrunk. Every page of an
    // ordinary shape is read exactly as before.
    let short = width.min(height);
    if short > 0 && width.max(height) / short > MAX_ORIENTATION_ASPECT {
        let square = image::imageops::crop_imm(
            page,
            (width - short) / 2,
            (height - short) / 2,
            short,
            short,
        )
        .to_image();
        return orientation_input(&square);
    }
    let scale = SHORT as f32 / width.min(height) as f32;
    let (resized_width, resized_height) = (
        ((width as f32 * scale).round() as u32).max(CROP),
        ((height as f32 * scale).round() as u32).max(CROP),
    );
    let shrunk = image::imageops::thumbnail(page, resized_width, resized_height);
    let left = (resized_width - CROP) / 2;
    let top = (resized_height - CROP) / 2;
    let plane = (CROP * CROP) as usize;
    let mut input = vec![0.0_f32; plane * 3];
    for y in 0..CROP {
        for x in 0..CROP {
            let pixel = shrunk.get_pixel(left + x, top + y).0;
            for channel in 0..3 {
                input[channel * plane + (y * CROP + x) as usize] =
                    (f32::from(pixel[channel]) / 255.0 - IMAGENET_MEAN[channel])
                        / IMAGENET_STD[channel];
            }
        }
    }
    input
}

/// A crop with its contrast stretched so its darkest and lightest few per
/// cent of pixels span the whole range. A faint line's strokes then differ
/// from the paper as much as a crisp one's do.
pub fn stretch_contrast(crop: &LineCrop) -> LineCrop {
    let mut lightness: Vec<f32> = crop
        .pixels
        .chunks_exact(3)
        .map(|rgb| (rgb[0] + rgb[1] + rgb[2]) / 3.0)
        .collect();
    if lightness.is_empty() {
        return crop.clone();
    }
    lightness.sort_by(f32::total_cmp);
    let low = lightness[lightness.len() * 2 / 100];
    let high = lightness[(lightness.len() * 98 / 100).min(lightness.len() - 1)];
    if high - low < 1.0 {
        return crop.clone();
    }
    let scale = 255.0 / (high - low);
    LineCrop {
        pixels: crop
            .pixels
            .iter()
            .map(|value| ((value - low) * scale).clamp(0.0, 255.0))
            .collect(),
        ..crop.clone()
    }
}

/// A crop made black and white at the threshold that best separates its
/// two populations of grey (Otsu's method). Speckle lighter than the ink
/// and shading darker than the paper both fall to the side they are
/// nearer.
pub fn binarize(crop: &LineCrop) -> LineCrop {
    let grey: Vec<u8> = crop
        .pixels
        .chunks_exact(3)
        .map(|rgb| ((rgb[0] + rgb[1] + rgb[2]) / 3.0).round().clamp(0.0, 255.0) as u8)
        .collect();
    let threshold = otsu_threshold(&grey);
    LineCrop {
        pixels: grey
            .iter()
            .flat_map(|&value| {
                let level = if value > threshold { 255.0 } else { 0.0 };
                [level; 3]
            })
            .collect(),
        ..crop.clone()
    }
}

/// The grey level that maximizes the between-class variance of `values`.
pub fn otsu_threshold(values: &[u8]) -> u8 {
    let mut histogram = [0_u64; 256];
    for &value in values {
        histogram[value as usize] += 1;
    }
    let total = values.len() as f64;
    let sum_all: f64 = histogram
        .iter()
        .enumerate()
        .map(|(level, &count)| level as f64 * count as f64)
        .sum();
    let (mut weight_low, mut sum_low) = (0.0_f64, 0.0_f64);
    let (mut best, mut best_variance) = (0_u8, -1.0_f64);
    for (level, &count) in histogram.iter().enumerate() {
        weight_low += count as f64;
        if weight_low == 0.0 {
            continue;
        }
        let weight_high = total - weight_low;
        if weight_high == 0.0 {
            break;
        }
        sum_low += level as f64 * count as f64;
        let mean_low = sum_low / weight_low;
        let mean_high = (sum_all - sum_low) / weight_high;
        let variance = weight_low * weight_high * (mean_low - mean_high).powi(2);
        if variance > best_variance {
            best_variance = variance;
            best = level as u8;
        }
    }
    best
}

/// The page turned by `degrees` about its centre, counter-clockwise for a
/// positive angle, on a canvas of the same size with white where the page
/// no longer reaches.
pub fn rotate_page(page: &RgbImage, degrees: f32) -> RgbImage {
    let (width, height) = page.dimensions();
    let centre = Point::new((width as f32 - 1.0) / 2.0, (height as f32 - 1.0) / 2.0);
    // Each output pixel looks up where it came from: the inverse turn.
    let radians = -degrees.to_radians();
    RgbImage::from_fn(width, height, |x, y| {
        let source = Point::new(x as f32, y as f32).rotated_about(centre, radians);
        if source.x < -0.5
            || source.y < -0.5
            || source.x > width as f32 - 0.5
            || source.y > height as f32 - 0.5
        {
            image::Rgb([255, 255, 255])
        } else {
            image::Rgb(sample(page, source).map(|value| value.round().clamp(0.0, 255.0) as u8))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A strip a pixel wide and a hundred thousand long reads as its central
    /// square: the classifier's input is the usual size, and the thumbnail
    /// is never as long as the strip.
    #[test]
    fn a_very_long_strip_is_classified_by_its_centre() {
        let strip = RgbImage::from_pixel(1, 100_000, image::Rgb([255, 255, 255]));
        let input = orientation_input(&strip);
        assert_eq!(input.len(), 3 * 224 * 224);
        // A page of an ordinary shape is read as it always was.
        let page = RgbImage::from_pixel(850, 1100, image::Rgb([200, 200, 200]));
        assert_eq!(orientation_input(&page).len(), 3 * 224 * 224);
    }
    use image::Rgb;

    #[test]
    fn detection_sizes_are_capped_and_rounded_to_32() {
        // A letter page at 300 DPI.
        let cap = |max| DetectionSize::LongSide { max, min: 0 };
        assert_eq!(detection_input_size(2550, 3300, cap(960)), (736, 960));
        assert_eq!(detection_input_size(2550, 3300, cap(1600)), (1248, 1600));
        assert_eq!(
            detection_input_size(2550, 3300, DetectionSize::Native { max_side: 4000 }),
            (2560, 3296)
        );
        // A small image is not enlarged without a floor, only rounded...
        assert_eq!(detection_input_size(500, 300, cap(960)), (512, 288));
        assert_eq!(detection_input_size(5, 5, cap(960)), (32, 32));
        // ...and with one, its longer side is brought up to it.
        let floored = DetectionSize::LongSide {
            max: 1600,
            min: 1024,
        };
        assert_eq!(detection_input_size(300, 560, floored), (544, 1024));
        assert_eq!(detection_input_size(2550, 3300, floored), (1248, 1600));
    }

    #[test]
    fn detection_input_is_planar_bgr_and_normalized() {
        // A pure red page: R=255, G=0, B=0.
        let page = RgbImage::from_pixel(64, 32, Rgb([255, 0, 0]));
        let input = detection_input(&page, 32, 32);
        assert_eq!(input.len(), 3 * 32 * 32);
        let plane = 32 * 32;
        // Blue plane first: blue is 0.
        assert!((input[0] - (0.0 - 0.485) / 0.229).abs() < 1e-5);
        assert!((input[plane] - (0.0 - 0.456) / 0.224).abs() < 1e-5);
        assert!((input[2 * plane] - (1.0 - 0.406) / 0.225).abs() < 1e-5);
    }

    #[test]
    fn a_line_is_cut_along_its_axis_at_recognition_height() {
        // A white page with a black bar from x 100..300, y 50..90.
        let mut page = RgbImage::from_pixel(400, 200, Rgb([255, 255, 255]));
        for y in 50..90 {
            for x in 100..300 {
                page.put_pixel(x, y, Rgb([0, 0, 0]));
            }
        }
        let quad = Quad([
            Point::new(100.0, 50.0),
            Point::new(300.0, 50.0),
            Point::new(300.0, 90.0),
            Point::new(100.0, 90.0),
        ]);
        let crop = crop_line(&page, &quad, RECOGNITION_HEIGHT);
        assert_eq!(crop.height, 48);
        assert_eq!(crop.width, 240, "200 x 40 at height 48");
        // Inside the bar throughout, away from the anti-aliased edges.
        let at = |x: u32, y: u32| crop.pixels[((y * crop.width + x) * 3) as usize];
        assert!(at(120, 24) < 1.0);
        assert!(at(5, 5) < 30.0 && at(234, 42) < 30.0);
    }

    #[test]
    fn a_tall_line_is_read_turned_a_quarter_counter_clockwise() {
        // Left half dark, right half light, in a 20 x 100 vertical box.
        let mut page = RgbImage::from_pixel(100, 200, Rgb([255, 255, 255]));
        for y in 50..150 {
            for x in 40..50 {
                page.put_pixel(x, y, Rgb([0, 0, 0]));
            }
        }
        let quad = Quad([
            Point::new(40.0, 50.0),
            Point::new(60.0, 50.0),
            Point::new(60.0, 150.0),
            Point::new(40.0, 150.0),
        ]);
        let crop = crop_line(&page, &quad, 48);
        assert_eq!(crop.width, 240);
        // Turned counter-clockwise, the dark left half becomes the bottom.
        let at = |x: u32, y: u32| crop.pixels[((y * crop.width + x) * 3) as usize];
        assert!(at(120, 40) < 10.0, "bottom {}", at(120, 40));
        assert!(at(120, 8) > 245.0, "top {}", at(120, 8));
    }

    #[test]
    fn a_batch_pads_with_zero_and_reverses_the_channels() {
        let crop = LineCrop {
            width: 2,
            height: 1,
            pixels: vec![255.0, 0.0, 0.0, 0.0, 0.0, 255.0],
        };
        let (batch, width) = recognition_batch(&[&crop], 4);
        assert_eq!(width, 4);
        assert_eq!(batch.len(), 3 * 4);
        // Blue plane: the first pixel has none, the second is all blue.
        assert_eq!(&batch[0..4], &[-1.0, 1.0, 0.0, 0.0]);
        // Red plane.
        assert_eq!(&batch[8..12], &[1.0, -1.0, 0.0, 0.0]);
    }

    #[test]
    fn otsu_splits_two_populations() {
        let mut values = vec![40_u8; 100];
        values.extend(vec![200_u8; 300]);
        let threshold = otsu_threshold(&values);
        assert!((40..200).contains(&threshold), "{threshold}");
    }

    #[test]
    fn contrast_stretch_spans_the_range() {
        let crop = LineCrop {
            width: 100,
            height: 1,
            pixels: (0..100).flat_map(|i| [100.0 + i as f32 * 0.5; 3]).collect(),
        };
        let stretched = stretch_contrast(&crop);
        let min = stretched.pixels.iter().copied().fold(f32::MAX, f32::min);
        let max = stretched.pixels.iter().copied().fold(f32::MIN, f32::max);
        assert_eq!((min, max), (0.0, 255.0));
    }

    #[test]
    fn rotating_a_page_keeps_its_size_and_fills_with_white() {
        let page = RgbImage::from_pixel(100, 50, Rgb([0, 0, 0]));
        let turned = rotate_page(&page, 10.0);
        assert_eq!(turned.dimensions(), (100, 50));
        assert_eq!(turned.get_pixel(0, 0).0, [255, 255, 255]);
        assert_eq!(turned.get_pixel(50, 25).0, [0, 0, 0]);
    }
}
