// Copyright 2025 - Clipper2 Rust port
// Direct port of Colors.h by Angus Johnson
// License: https://www.boost.org/LICENSE_1_0.txt
//
// Purpose: HSL color utilities for SVG visualization

/// HSL color representation with alpha channel.
/// All components are 0-255.
///
/// Direct port from C++ `Hsl` struct.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Hsl {
    pub alpha: u8,
    pub hue: u8,
    pub sat: u8,
    pub lum: u8,
}

impl Hsl {
    pub fn new(alpha: u8, hue: u8, sat: u8, lum: u8) -> Self {
        Self {
            alpha,
            hue,
            sat,
            lum,
        }
    }
}

/// ARGB color packed into a u32.
///
/// Layout: `0xAARRGGBB` (alpha in high byte, blue in low byte).
/// This matches the C++ `Color32` union.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color32 {
    pub color: u32,
}

impl Color32 {
    /// Create from individual ARGB components.
    pub fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Self {
        let color = (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32);
        Self { color }
    }

    /// Extract alpha component.
    pub fn alpha(self) -> u8 {
        (self.color >> 24) as u8
    }

    /// Extract red component.
    pub fn red(self) -> u8 {
        (self.color >> 16) as u8
    }

    /// Extract green component.
    pub fn green(self) -> u8 {
        (self.color >> 8) as u8
    }

    /// Extract blue component.
    pub fn blue(self) -> u8 {
        self.color as u8
    }
}

/// Convert an HSL color to an ARGB Color32.
///
/// Direct port from C++ `HslToRgb()`.
pub fn hsl_to_rgb(hsl: Hsl) -> Color32 {
    let c = ((255 - (2 * hsl.lum as i32 - 255).abs()) * hsl.sat as i32) >> 8;
    let a = 252 - (hsl.hue as i32 % 85) * 6;
    let x = (c * (255 - a.abs())) >> 8;
    let m = hsl.lum as i32 - c / 2;

    let (r, g, b) = match (hsl.hue as i32 * 6) >> 8 {
        0 => (c + m, x + m, m),
        1 => (x + m, c + m, m),
        2 => (m, c + m, x + m),
        3 => (m, x + m, c + m),
        4 => (x + m, m, c + m),
        5 => (c + m, m, x + m),
        _ => (m, m, m),
    };

    Color32::from_argb(
        hsl.alpha,
        r.clamp(0, 255) as u8,
        g.clamp(0, 255) as u8,
        b.clamp(0, 255) as u8,
    )
}

/// Generate a rainbow color for a fractional position.
///
/// Direct port from C++ `RainbowColor()`.
///
/// # Arguments
/// * `frac` - Position in the rainbow (0.0 to 1.0, wraps)
/// * `luminance` - Brightness (0-255, default 128)
/// * `alpha` - Opacity (0-255, default 255)
pub fn rainbow_color(frac: f64, luminance: u8, alpha: u8) -> u32 {
    let frac = frac - frac.floor();
    let hsl = Hsl::new(alpha, (frac * 255.0) as u8, 255, luminance);
    hsl_to_rgb(hsl).color
}

/// Convenience wrapper using default luminance (128) and alpha (255).
pub fn rainbow_color_default(frac: f64) -> u32 {
    rainbow_color(frac, 128, 255)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsl_default() {
        let h = Hsl::default();
        assert_eq!(h.alpha, 0);
        assert_eq!(h.hue, 0);
        assert_eq!(h.sat, 0);
        assert_eq!(h.lum, 0);
    }

    #[test]
    fn test_hsl_new() {
        let h = Hsl::new(255, 128, 200, 100);
        assert_eq!(h.alpha, 255);
        assert_eq!(h.hue, 128);
        assert_eq!(h.sat, 200);
        assert_eq!(h.lum, 100);
    }

    #[test]
    fn test_color32_from_argb() {
        let c = Color32::from_argb(0xFF, 0x12, 0x34, 0x56);
        assert_eq!(c.alpha(), 0xFF);
        assert_eq!(c.red(), 0x12);
        assert_eq!(c.green(), 0x34);
        assert_eq!(c.blue(), 0x56);
        assert_eq!(c.color, 0xFF123456);
    }

    #[test]
    fn test_hsl_to_rgb_zero_sat() {
        // Zero saturation should produce a gray
        let hsl = Hsl::new(255, 0, 0, 128);
        let rgb = hsl_to_rgb(hsl);
        assert_eq!(rgb.alpha(), 255);
        // With zero saturation, r == g == b
        assert_eq!(rgb.red(), rgb.green());
        assert_eq!(rgb.green(), rgb.blue());
    }

    #[test]
    fn test_hsl_to_rgb_full_saturation_red() {
        // Hue 0, full saturation, mid luminance should be red-ish
        let hsl = Hsl::new(255, 0, 255, 128);
        let rgb = hsl_to_rgb(hsl);
        assert_eq!(rgb.alpha(), 255);
        assert!(rgb.red() > rgb.green());
        assert!(rgb.red() > rgb.blue());
    }

    #[test]
    fn test_rainbow_color_returns_opaque() {
        let c = rainbow_color(0.0, 128, 255);
        assert_eq!((c >> 24) & 0xFF, 255);
    }

    #[test]
    fn test_rainbow_color_wraps() {
        // Values > 1.0 should wrap
        let c1 = rainbow_color(0.25, 128, 255);
        let c2 = rainbow_color(1.25, 128, 255);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_rainbow_color_different_positions() {
        let c1 = rainbow_color(0.0, 128, 255);
        let c2 = rainbow_color(0.5, 128, 255);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_rainbow_color_default() {
        let c = rainbow_color_default(0.3);
        assert_eq!((c >> 24) & 0xFF, 255); // alpha = 255
    }
}
