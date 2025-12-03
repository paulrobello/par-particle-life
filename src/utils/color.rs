//! Color conversion utilities.

/// Convert HSV to RGB.
///
/// # Arguments
/// * `h` - Hue in degrees [0, 360)
/// * `s` - Saturation [0, 1]
/// * `v` - Value/brightness [0, 1]
///
/// # Returns
/// RGB values as [r, g, b] each in [0, 1]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let c = v * s;
    let hp = h / 60.0;
    let x = c * (1.0 - ((hp % 2.0) - 1.0).abs());

    let (r, g, b) = if hp < 1.0 {
        (c, x, 0.0)
    } else if hp < 2.0 {
        (x, c, 0.0)
    } else if hp < 3.0 {
        (0.0, c, x)
    } else if hp < 4.0 {
        (0.0, x, c)
    } else if hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let m = v - c;
    [r + m, g + m, b + m]
}

/// Convert RGB to HSV.
///
/// # Arguments
/// * `r`, `g`, `b` - RGB values each in [0, 1]
///
/// # Returns
/// HSV values as [h, s, v] where h is in [0, 360)
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let v = max;
    let s = if max == 0.0 { 0.0 } else { delta / max };

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    [h, s, v]
}

/// Convert a color from 0-1 float to 0-255 integer.
pub fn color_to_u8(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Convert a color from 0-255 integer to 0-1 float.
pub fn u8_to_color(c: u8) -> f32 {
    c as f32 / 255.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsv_to_rgb_red() {
        let [r, g, b] = hsv_to_rgb(0.0, 1.0, 1.0);
        assert!((r - 1.0).abs() < 0.01);
        assert!(g.abs() < 0.01);
        assert!(b.abs() < 0.01);
    }

    #[test]
    fn test_hsv_to_rgb_green() {
        let [r, g, b] = hsv_to_rgb(120.0, 1.0, 1.0);
        assert!(r.abs() < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!(b.abs() < 0.01);
    }

    #[test]
    fn test_hsv_to_rgb_blue() {
        let [r, g, b] = hsv_to_rgb(240.0, 1.0, 1.0);
        assert!(r.abs() < 0.01);
        assert!(g.abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_rgb_hsv_roundtrip() {
        let original = [0.5, 0.3, 0.8];
        let [h, s, v] = rgb_to_hsv(original[0], original[1], original[2]);
        let [r, g, b] = hsv_to_rgb(h, s, v);
        assert!((r - original[0]).abs() < 0.01);
        assert!((g - original[1]).abs() < 0.01);
        assert!((b - original[2]).abs() < 0.01);
    }
}
