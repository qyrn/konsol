use image::{DynamicImage, ImageReader, imageops::FilterType};
use std::fs;
use std::path::Path;

const HUE_BUCKETS: usize = 12;
const ICON_SIZE: u32 = 256;
const FALLBACK_ACCENT: &str = "#4a7bff";

fn decode(path: &Path) -> Option<DynamicImage> {
    ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()
}

pub fn dominant_accent(path: &Path) -> Option<String> {
    let pixels = decode(path)?.thumbnail(48, 48).to_rgb8();
    accent_from_pixels(pixels.pixels().map(|pixel| pixel.0))
}

fn accent_from_pixels(pixels: impl Iterator<Item = [u8; 3]>) -> Option<String> {
    let mut weights = [0.0_f32; HUE_BUCKETS];
    let mut sums = [[0.0_f32; 3]; HUE_BUCKETS];
    for [red, green, blue] in pixels {
        let rgb = [red, green, blue].map(|channel| f32::from(channel) / 255.0);
        let (hue, saturation, value) = to_hsv(rgb);
        if saturation < 0.25 || value < 0.2 {
            continue;
        }
        let bucket = ((hue / 360.0) * HUE_BUCKETS as f32) as usize % HUE_BUCKETS;
        let weight = saturation * saturation * value;
        weights[bucket] += weight;
        for channel in 0..3 {
            sums[bucket][channel] += rgb[channel] * weight;
        }
    }
    let (bucket, weight) = weights
        .iter()
        .enumerate()
        .max_by(|first, second| first.1.total_cmp(second.1))?;
    if *weight <= 0.0 {
        return None;
    }
    let average = sums[bucket].map(|sum| sum / weight);
    let (hue, saturation, value) = to_hsv(average);
    let vivid = from_hsv(hue, saturation.max(0.6), value.clamp(0.78, 0.95));
    Some(to_hex(vivid))
}

pub fn accent_from_title(title: &str) -> String {
    let hash = title.bytes().fold(2166136261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16777619)
    });
    to_hex(from_hsv((hash % 360) as f32, 0.62, 0.9))
}

fn to_hsv([red, green, blue]: [f32; 3]) -> (f32, f32, f32) {
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    let hue = if delta == 0.0 {
        0.0
    } else if max == red {
        60.0 * ((green - blue) / delta).rem_euclid(6.0)
    } else if max == green {
        60.0 * ((blue - red) / delta + 2.0)
    } else {
        60.0 * ((red - green) / delta + 4.0)
    };
    let saturation = if max == 0.0 { 0.0 } else { delta / max };
    (hue, saturation, max)
}

fn from_hsv(hue: f32, saturation: f32, value: f32) -> [f32; 3] {
    let chroma = value * saturation;
    let segment = hue / 60.0;
    let second = chroma * (1.0 - (segment.rem_euclid(2.0) - 1.0).abs());
    let (red, green, blue) = match segment as u32 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };
    let offset = value - chroma;
    [red + offset, green + offset, blue + offset]
}

fn to_hex(rgb: [f32; 3]) -> String {
    let [red, green, blue] = rgb.map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8);
    format!("#{red:02x}{green:02x}{blue:02x}")
}

pub fn write_png_icon(source: &Path, destination: &Path) -> Result<(), String> {
    let image = decode(source).ok_or_else(|| format!("Image illisible : {}", source.display()))?;
    let side = image.width().min(image.height());
    let left = (image.width() - side) / 2;
    let top = (image.height() - side) / 2;
    image
        .crop_imm(left, top, side, side)
        .resize_exact(ICON_SIZE, ICON_SIZE, FilterType::Lanczos3)
        .save(destination)
        .map_err(|error| format!("Impossible d'écrire l'icône : {error}"))
}

pub fn write_letter_icon(
    title: &str,
    accent: Option<&str>,
    destination: &Path,
) -> Result<(), String> {
    fs::write(destination, letter_icon_svg(title, accent))
        .map_err(|error| format!("Impossible d'écrire l'icône : {error}"))
}

fn letter_icon_svg(title: &str, accent: Option<&str>) -> String {
    let initials: String = title
        .split_whitespace()
        .filter_map(|word| word.chars().find(|character| character.is_alphanumeric()))
        .take(2)
        .flat_map(char::to_uppercase)
        .collect();
    let accent = accent
        .filter(|color| is_hex_color(color))
        .unwrap_or(FALLBACK_ACCENT);
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256"><defs><radialGradient id="glow" cx="0.5" cy="0.7" r="0.8"><stop offset="0" stop-color="{accent}"/><stop offset="1" stop-color="#05060c"/></radialGradient></defs><rect width="256" height="256" rx="56" fill="url(#glow)"/><text x="128" y="160" text-anchor="middle" font-family="sans-serif" font-weight="800" font-size="104" fill="#ffffff">{initials}</text></svg>"##
    )
}

fn is_hex_color(color: &str) -> bool {
    color.len() == 7
        && color.starts_with('#')
        && color[1..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_most_vivid_hue_and_ignores_greys() {
        let pixels = [[200, 30, 30]; 10]
            .into_iter()
            .chain([[128, 128, 128]; 50])
            .chain([[20, 20, 20]; 50]);
        let accent = accent_from_pixels(pixels).expect("an accent");
        let red = u8::from_str_radix(&accent[1..3], 16).unwrap();
        let green = u8::from_str_radix(&accent[3..5], 16).unwrap();
        assert!(red > 180 && green < 90, "got {accent}");
    }

    #[test]
    fn gives_each_title_a_stable_color() {
        assert_eq!(
            accent_from_title("Skyrim AE"),
            accent_from_title("Skyrim AE")
        );
        assert_ne!(
            accent_from_title("Skyrim AE"),
            accent_from_title("Rockstar Launcher")
        );
        assert!(is_hex_color(&accent_from_title("Skyrim AE")));
    }

    #[test]
    fn returns_nothing_for_a_grey_image() {
        assert_eq!(accent_from_pixels([[90, 90, 90]; 20].into_iter()), None);
    }

    #[test]
    fn builds_letter_icons_with_escaped_safe_content() {
        let svg = letter_icon_svg("rockstar launcher", Some("#f5a623"));
        assert!(svg.contains(">RL</text>"));
        assert!(svg.contains("#f5a623"));
        let unsafe_svg = letter_icon_svg("<script> x", Some("red\" onload=\""));
        assert!(!unsafe_svg.contains("<script"));
        assert!(unsafe_svg.contains(FALLBACK_ACCENT));
    }
}
