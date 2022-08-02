//! Converts colours[^note] to many common color representations (e.g. RGB, HSL, HSV) and does simple
//! colour manipulation operations. Thanks to [colour](https://pypi.org/project/colour/) and
//! [colors.py](https://pypi.org/project/colors.py/) for inspiring the API (and documentation) of this project.
//!
//! # Note
//!
//! [`Color`] stores colors using the RGB color model, storing R, G, and B values as [`u8`]s. Hence,
//! the result you get back from converting from HSL/HSV to [`Color`] and back will not always be the same
//! due to lack of precision.
//!
//! ```
//! use octarine::Color;
//!
//! let to_hsl = Color::from_hsl(0.0, 0.0, 0.5).to_hsl();
//! assert_ne!((0.0, 0.0, 0.5), to_hsl);
//!
//! let to_hsv = Color::from_hsv(0.0, 0.0, 0.5).to_hsv();
//! assert_ne!((0.0, 0.0, 0.5), to_hsv);
//! ```
//!
//! # Features
//!
//! - Extremely simple API (subjective).
//! - Convert between RGB, HSL, HSV, W3C web colors, and hexadecimal.
//! - One struct ([`Color`]) to rule them all.
//! - Perform arithmetic, blend modes, and generate random colors within boundaries.
//! - [`Octarine`](constants::OCTARINE).
//!
//! # Examples
//!
//! ```
//! use octarine::Color;
//!
//! let color1 = Color::from_web_color("red");
//! let color2 = Color::new(255, 0, 0);
//! assert_eq!(color1, Some(color2.clone()));
//!
//! let hex = Color::new(100, 100, 100).to_hex();
//! assert_eq!(hex, 0x646464);
//!
//! let red = color2.get_red();
//! let green = color2.get_green();
//! let blue = color2.get_blue();
//! assert_eq!(color2.to_rgb(), (red, green, blue));
//!
//! let hue = color2.get_hsl_hue();
//! let saturation = color2.get_hsl_saturation();
//! let luminance = color2.get_hsl_luminance();
//! assert_eq!(color2.to_hsl(), (hue, saturation, luminance));
//!
//! let hue = color2.get_hsv_hue();
//! let saturation = color2.get_hsv_saturation();
//! let value = color2.get_hsv_value();
//! assert_eq!(color2.to_hsv(), (hue, saturation, value));
//! ```
//!
//! [^note]: Bri'ish amirite. Sorry, I use British English myself. But just to keep things consistent,
//! I will be using American English from now on.

use rand::{prelude::SmallRng, Rng, SeedableRng};
use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

pub mod constants;

macro_rules! test_color_value_range {
    ($r:expr, $g:expr, $b:expr) => {
        let mut range_error = false;
        let mut bad_component_string = String::new();
        if $r + f32::EPSILON < 0.0 || $r - f32::EPSILON > 1.0 {
            range_error = true;
            bad_component_string += " Red";
        }
        if $g + f32::EPSILON < 0.0 || $g - f32::EPSILON > 1.0 {
            range_error = true;
            bad_component_string += " Green";
        }
        if $b + f32::EPSILON < 0.0 || $b - f32::EPSILON > 1.0 {
            range_error = true;
            bad_component_string += " Blue";
        }
        if range_error {
            panic!("Color parameter outside of expected range:{bad_component_string}");
        }
    };
    ($s:expr, $x:expr) => {
        let mut range_error = false;
        let mut bad_component_string = String::new();
        if $s + f32::EPSILON < 0.0 || $s - f32::EPSILON > 1.0 {
            range_error = true;
            bad_component_string += " Saturation";
        }
        if $x + f32::EPSILON < 0.0 || $x - f32::EPSILON > 1.0 {
            range_error = true;
            bad_component_string += " Lightness/Value";
        }
        if range_error {
            panic!("Color parameter outside of expected range:{bad_component_string}");
        }
    };
}

/// Color type used to convert and manipulate colors.
#[derive(Clone)]
pub struct Color(u8, u8, u8);

impl Color {
    /// Create a color object from RGB values (0 - 255).
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let color = Color::new(100, 100, 100);
    /// println!("{color:?}");
    /// ```
    #[inline]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }

    /// Create a color object from RGB floats (0.0 - 1.0).
    ///
    /// # Panics
    ///
    /// Panics when the R, G, or B values are < 0.0 or > 1.0.
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let color = Color::from_rgb_float(0.0, 0.5, 0.0);
    /// println!("{color:?}");
    /// ```
    pub fn from_rgb_float(r: f32, g: f32, b: f32) -> Self {
        test_color_value_range!(r, g, b);
        let r = (r * 255.0 + 0.5) as u8;
        let g = (g * 255.0 + 0.5) as u8;
        let b = (b * 255.0 + 0.5) as u8;
        Self(r, g, b)
    }

    /// Creates a color object from hexadecimal (which is essentially an unsigned integer).
    ///
    /// Use the [`hex` crate](https://crates.io/crates/hex) if you want to convert a hex string
    /// to an integer.
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::{Color, constants};
    ///
    /// let color = Color::from_hex(0xFF0000);
    ///
    /// assert_eq!(color, constants::primary::RED);
    /// ```
    pub fn from_hex(rgb: u32) -> Self {
        let r = ((rgb & 0xFFFFFF) >> 16) as u8;
        let g = ((rgb & 0xFFFF) >> 8) as u8;
        let b = (rgb & 0xFF) as u8;
        Self(r, g, b)
    }

    /// Creates a color object from HSL values.
    ///
    /// # Panics
    ///
    /// Panics when S and L values are < 0.0 or > 1.0.
    ///
    /// # Note
    ///
    /// Hue can be set to any value but as it is a rotation
    /// around the chromatic circle, any value above 1 or below 0 can
    /// be expressed by a value between 0 and 1 (Note that `h = 0` is equivalent
    /// to `h = 1`).
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let color = Color::from_hsl(0.0, 0.5, 0.0);
    /// println!("{color:?}");
    /// ```
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        test_color_value_range!(s, l);
        if s == 0.0 {
            return Self::from_rgb_float(l, l, l);
        }
        let v2 = if l < 0.5 {
            l * (1.0 + s)
        } else {
            (l + s) - (s * l)
        };
        let v1 = 2.0 * l - v2;
        let r = hue_to_rgb(v1, v2, h + (1.0 / 3.0));
        let g = hue_to_rgb(v1, v2, h);
        let b = hue_to_rgb(v1, v2, h - (1.0 / 3.0));
        Self::from_rgb_float(r, g, b)
    }

    /// Creates a color object from HSV values.
    ///
    /// # Panics
    ///
    /// Panics when S and V values are < 0.0 or > 1.0.
    ///
    /// # Note
    ///
    /// Hue can be set to any value but as it is a rotation
    /// around the chromatic circle, any value above 1 or below 0 can
    /// be expressed by a value between 0 and 1 (Note that `h = 0` is equivalent
    /// to `h = 1`).
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let color = Color::from_hsv(0.0, 0.5, 0.0);
    /// println!("{color:?}");
    /// ```
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        test_color_value_range!(s, v);
        if s == 0.0 {
            return Self::from_rgb_float(v, v, v);
        }
        let mut v_h = h * 6.0;
        if v_h == 6.0 {
            v_h = 0.0;
        }
        let v_i = v_h.floor();
        let v1 = v * (1.0 - s);
        let v2 = v * (1.0 - s * (v_h - v_i));
        let v3 = v * (1.0 - s * (1.0 - (v_h - v_i)));
        let (r, g, b) = if v_i == 0.0 {
            (v, v3, v1)
        } else if v_i == 1.0 {
            (v2, v, v1)
        } else if v_i == 2.0 {
            (v1, v, v3)
        } else if v_i == 3.0 {
            (v1, v2, v)
        } else if v_i == 4.0 {
            (v3, v1, v)
        } else {
            (v, v1, v2)
        };
        Self::from_rgb_float(r, g, b)
    }

    /// Creates a color object from web colors. Returns `None` when the color cannot be found.
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::{Color, constants};
    ///
    /// let color = Color::from_web_color("red");
    ///
    /// assert_eq!(color, Some(constants::primary::RED));
    /// ```
    pub fn from_web_color(name: &str) -> Option<Self> {
        let name = name.to_lowercase();
        constants::RGB_TO_COLOR_NAMES.get(&name).cloned()
    }

    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let color1 = Color::from_hex(0xFF9D9D);
    /// let color2 = Color::from_hex(0xFF9999).screen(Color::new(10, 10, 10));
    ///
    /// assert_eq!(color1, color2);
    /// ```
    pub fn screen(&self, other: Self) -> Self {
        let r = 255 - (((255 - self.0) as usize * (255 - other.0) as usize) / 255) as u8;
        let g = 255 - (((255 - self.1) as usize * (255 - other.1) as usize) / 255) as u8;
        let b = 255 - (((255 - self.2) as usize * (255 - other.2) as usize) / 255) as u8;
        Self(r, g, b)
    }

    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let color1 = Color::from_hex(0xF58F8F);
    /// let color2 = Color::from_hex(0xFF9999).difference(Color::new(10, 10, 10));
    ///
    /// assert_eq!(color1, color2);
    /// ```
    pub fn difference(&self, other: Self) -> Self {
        let r = self.0.abs_diff(other.0);
        let g = self.1.abs_diff(other.1);
        let b = self.2.abs_diff(other.2);
        Self(r, g, b)
    }

    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let color1 = Color::new(255, 156, 156);
    /// let color2 = Color::from_hex(0xFF9999).overlay(Color::new(10, 10, 10));
    ///
    /// assert_eq!(color1, color2);
    /// ```
    #[inline]
    pub fn overlay(&self, other: Self) -> Self {
        self.screen(self.clone() * other)
    }

    /// # Example
    ///
    /// ```
    /// use octarine::constants;
    ///
    /// let color = constants::primary::BLACK.invert();
    ///
    /// assert_eq!(color, constants::primary::WHITE);
    /// ```
    #[inline]
    pub fn invert(&self) -> Self {
        self.difference(Self(255, 255, 255))
    }

    /// Get a random color.
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let random_color = Color::random_color();
    /// println!("{random_color:?}");
    /// ```
    pub fn random_color() -> Self {
        let mut rng = rand::rngs::SmallRng::from_entropy();
        let r: u8 = rng.gen();
        let g: u8 = rng.gen();
        let b: u8 = rng.gen();
        Self(r, g, b)
    }

    /// Get the hexadecimal representation of a color.
    ///
    /// Use the [`hex` crate](https://crates.io/crates/hex) if you want to convert an integer
    /// to a hex string.
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let hex = Color::new(100, 100, 100).to_hex();
    ///
    /// assert_eq!(hex, 0x646464);
    /// ```
    pub fn to_hex(&self) -> u32 {
        let b = (self.2 as u32) << 16;
        let g = (self.1 as u32) << 8;
        let r = self.0 as u32;
        r | g | b
    }

    /// Converts a color to HSL.
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let hsl = Color::new(100, 100, 100).to_hsl();
    ///
    /// assert_eq!(hsl, (0.0, 0.0, 0.39215687));
    /// ```
    pub fn to_hsl(&self) -> (f32, f32, f32) {
        let v_min = std::cmp::min(std::cmp::min(self.0, self.1), self.2) as f32 / 255.0;
        let v_max = std::cmp::max(std::cmp::max(self.0, self.1), self.2) as f32 / 255.0;
        let diff = v_max - v_min;
        let v_sum = v_min + v_max;
        let l = v_sum / 2.0;
        if diff < f32::EPSILON {
            return (0.0, 0.0, l);
        }
        let s = if l < 0.5 {
            diff / v_sum
        } else {
            diff / (2.0 - v_sum)
        };
        let (r, g, b) = self.to_rgb_float();
        let dr = (((v_max - r) / 6.0) + (diff / 2.0)) / diff;
        let dg = (((v_max - g) / 6.0) + (diff / 2.0)) / diff;
        let db = (((v_max - b) / 6.0) + (diff / 2.0)) / diff;
        let mut h = if r == v_max {
            db - dg
        } else if g == v_max {
            (1.0 / 3.0) + dr - db
        } else {
            (2.0 / 3.0) + dg - dr
        };
        if h < 0.0 {
            h += 1.0;
        }
        if h > 1.0 {
            h -= 1.0;
        }
        (h, s, l)
    }

    /// Converts a color to HSV.
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let hsl = Color::from_hex(0x646464).to_hsv();
    ///
    /// assert_eq!(hsl, (0.0, 0.0, 0.392156862745));
    /// ```
    pub fn to_hsv(&self) -> (f32, f32, f32) {
        let v_min = std::cmp::min(std::cmp::min(self.0, self.1), self.2) as f32 / 255.0;
        let v_max = std::cmp::max(std::cmp::max(self.0, self.1), self.2) as f32 / 255.0;
        let diff = v_max - v_min;
        let v = v_max;
        if diff < f32::EPSILON {
            return (0.0, 0.0, v);
        }
        let s = diff / v_max;
        let (r, g, b) = self.to_rgb_float();
        let dr = (((v_max - r) / 6.0) + (diff / 2.0)) / diff;
        let dg = (((v_max - g) / 6.0) + (diff / 2.0)) / diff;
        let db = (((v_max - b) / 6.0) + (diff / 2.0)) / diff;
        let mut h = if r == v_max {
            db - dg
        } else if g == v_max {
            (1.0 / 3.0) + dr - db
        } else {
            (2.0 / 3.0) + dg - dr
        };
        if h < 0.0 {
            h += 1.0;
        }
        if h > 1.0 {
            h -= 1.0;
        }
        (h, s, v)
    }

    /// Converts a color back to its RGB representation.
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let color = Color::new(100, 100, 100).to_rgb();
    ///
    /// assert_eq!(color, (100, 100, 100));
    /// ```
    #[inline]
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        (self.0, self.1, self.2)
    }

    /// Converts a color back to its RGB float representation.
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let color = Color::new(0, 255, 0).to_rgb_float();
    ///
    /// assert_eq!(color, (0.0, 1.0, 0.0));
    /// ```
    #[inline]
    pub fn to_rgb_float(&self) -> (f32, f32, f32) {
        (
            self.0 as f32 / 255.0,
            self.1 as f32 / 255.0,
            self.2 as f32 / 255.0,
        )
    }

    /// Get the red value of RGB.
    #[inline]
    pub fn get_red(&self) -> u8 {
        self.0
    }

    /// Get the green value of RGB.
    #[inline]
    pub fn get_green(&self) -> u8 {
        self.1
    }

    /// Get the blue value of RGB.
    #[inline]
    pub fn get_blue(&self) -> u8 {
        self.2
    }

    /// Get the hue (H) of HSL.
    #[inline]
    pub fn get_hsl_hue(&self) -> f32 {
        self.to_hsl().0
    }

    /// Get the saturation (S) of HSL.
    #[inline]
    pub fn get_hsl_saturation(&self) -> f32 {
        self.to_hsl().1
    }

    /// Get the luminance (L) of HSL.
    #[inline]
    pub fn get_hsl_luminance(&self) -> f32 {
        self.to_hsl().2
    }

    /// Get the hue (H) of HSV.
    #[inline]
    pub fn get_hsv_hue(&self) -> f32 {
        self.to_hsv().0
    }

    /// Get the saturation (S) of HSV.
    #[inline]
    pub fn get_hsv_saturation(&self) -> f32 {
        self.to_hsv().1
    }

    /// Get the value (V) of HSV.
    #[inline]
    pub fn get_hsv_value(&self) -> f32 {
        self.to_hsv().2
    }

    /// Gets the W3C web color. Returns `None` if no web color matches the current color.
    ///
    /// # Example
    /// ```
    /// use octarine::{Color, constants};
    ///
    /// let web_color = constants::primary::RED.get_web_color();
    ///
    /// assert_eq!(web_color, Some("red"));
    /// ```
    #[inline]
    pub fn get_web_color(&self) -> Option<&'static str> {
        constants::RGB_TO_COLOR_NAMES
            .into_iter()
            .find_map(|(name, color)| if self == color { Some(*name) } else { None })
    }

    /// Sets the red value of RGB.
    pub fn set_red(&mut self, red: u8) {
        self.0 = red;
    }

    /// Sets the green value of RGB.
    pub fn set_green(&mut self, green: u8) {
        self.1 = green;
    }

    /// Sets the blue value of RGB.
    pub fn set_blue(&mut self, blue: u8) {
        self.2 = blue;
    }

    /// Sets the hue (H) of HSL.
    pub fn set_hsl_hue(&mut self, hue: f32) {
        let hsl = self.to_hsl();
        let color = Self::from_hsl(hue, hsl.1, hsl.2);
        self.0 = color.0;
        self.1 = color.1;
        self.2 = color.2;
    }

    /// Sets the saturation (S) of HSL.
    pub fn set_hsl_saturation(&mut self, saturation: f32) {
        let hsl = self.to_hsl();
        let color = Self::from_hsl(hsl.0, saturation, hsl.2);
        self.0 = color.0;
        self.1 = color.1;
        self.2 = color.2;
    }

    /// Sets the luminance (L) of HSL.
    pub fn set_hsl_luminance(&mut self, luminance: f32) {
        let hsl = self.to_hsl();
        let color = Self::from_hsl(hsl.0, hsl.1, luminance);
        self.0 = color.0;
        self.1 = color.1;
        self.2 = color.2;
    }

    /// Sets the hue (H) of HSV.
    pub fn set_hsv_hue(&mut self, hue: f32) {
        let hsv = self.to_hsv();
        let color = Self::from_hsv(hue, hsv.1, hsv.2);
        self.0 = color.0;
        self.1 = color.1;
        self.2 = color.2;
    }

    /// Sets the saturation (S) of HSV.
    pub fn set_hsv_saturation(&mut self, saturation: f32) {
        let hsv = self.to_hsv();
        let color = Self::from_hsv(hsv.0, saturation, hsv.2);
        self.0 = color.0;
        self.1 = color.1;
        self.2 = color.2;
    }

    /// Sets the value (V) of HSV.
    pub fn set_hsv_value(&mut self, value: f32) {
        let hsv = self.to_hsv();
        let color = Self::from_hsv(hsv.0, hsv.1, value);
        self.0 = color.0;
        self.1 = color.1;
        self.2 = color.2;
    }

    /// Returns a [`ColorRange`] which is an iterator that returns some color scales
    /// of variation between the current color and another color specified. Refer to
    /// [`ColorRange`] for more information on how it works.
    ///
    /// # Panics
    ///
    /// Panics when steps is 0.
    ///
    /// # Example
    ///
    /// ```
    /// use octarine::Color;
    ///
    /// let c0 = Color::from_web_color("red").unwrap();
    /// let c1 = Color::from_hex(0xFF7F00);
    /// let c2 = Color::from_web_color("yellow").unwrap();
    /// let c3 = Color::new(128, 255, 0);
    /// let c4 = Color::from_web_color("lime").unwrap();
    /// let mut range_to = c0.range_to(c4.clone(), 5);
    ///
    /// assert_eq!(Some(c0), range_to.next());
    /// assert_eq!(Some(c1), range_to.next());
    /// assert_eq!(Some(c2), range_to.next());
    /// assert_eq!(Some(c3), range_to.next());
    /// assert_eq!(Some(c4), range_to.next());
    /// assert_eq!(None, range_to.next());
    /// ```
    #[inline]
    pub fn range_to(&self, value: Self, steps: usize) -> ColorRange {
        ColorRange::new(self.clone(), value, steps)
    }

    /// This method offers a way to equate colors using [`Equivalence`], in which a color is
    /// equated using its RGB, HSL, or HSV values.
    ///
    /// # Note
    ///
    /// By default octarine uses `[Equivalence::RGB]` for its [`PartialEq`] representation, so
    /// if such equality operations are not required, you can simply
    /// equate two color objects using `==`.
    ///
    /// # Example
    /// ```
    /// use octarine::{Color, Equivalence};
    ///
    /// let color = Color::new(100, 100, 100);
    /// let color2 = color.clone();
    ///
    /// assert_eq!(color == color2, color.complex_eq(&color2, Equivalence::RGB));
    /// ```
    pub fn complex_eq(&self, other: &Self, equivalence: Equivalence) -> bool {
        match equivalence {
            Equivalence::RGB => self.0 == other.0 && self.1 == other.1 && self.2 == other.2,
            Equivalence::HSL => self.to_hsl() == other.to_hsl(),
            Equivalence::HSV => self.to_hsv() == other.to_hsv(),
        }
    }
}

/// Specifies the methods in which [`complex_eq()`](Color#method.complex_eq) compares colors.
pub enum Equivalence {
    RGB,
    HSL,
    HSV,
}

fn hue_to_rgb(v1: f32, v2: f32, mut v_h: f32) -> f32 {
    while v_h < 0.0 {
        v_h += 1.0;
    }
    while v_h > 1.0 {
        v_h -= 1.0;
    }
    if 6.0 * v_h < 1.0 {
        v1 + (v2 - v1) * 6.0 * v_h
    } else if 2.0 * v_h < 1.0 {
        v2
    } else if 3.0 * v_h < 2.0 {
        v1 + (v2 - v1) * ((2.0 / 3.0) - v_h) * 6.0
    } else {
        v1
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.complex_eq(other, Equivalence::RGB)
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let r = (self.0 as usize * rhs.0 as usize / 255) as u8;
        let g = (self.1 as usize * rhs.1 as usize / 255) as u8;
        let b = (self.2 as usize * rhs.2 as usize / 255) as u8;
        Self::new(r, g, b)
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let r = self.0.saturating_add(rhs.0);
        let g = self.1.saturating_add(rhs.1);
        let b = self.2.saturating_add(rhs.2);
        Self::new(r, g, b)
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let r = self.0.saturating_sub(rhs.0);
        let g = self.1.saturating_sub(rhs.1);
        let b = self.2.saturating_sub(rhs.2);
        Self::new(r, g, b)
    }
}

impl Div for Color {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let r = self.0 / rhs.0;
        let g = self.1 / rhs.1;
        let b = self.2 / rhs.2;
        Self::new(r, g, b)
    }
}

impl ToString for Color {
    fn to_string(&self) -> String {
        format!("{}, {}, {}", self.0, self.1, self.2)
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Color red: {}, green: {}, blue: {}",
            self.0, self.1, self.2
        )
    }
}

/// The color wheel allows you to randomly choose colors while keeping the colors relatively evenly distributed. Think
/// generating random colors without pooling in one hue, e.g., not 50 green, and 1 red.
///
/// ColorWheel is an iterable, but be careful if using inside any type of loop. It will iterate forever until you interject.
#[derive(Debug, Clone)]
pub struct ColorWheel {
    phase: f32,
    rng: SmallRng,
}

impl ColorWheel {
    /// Creates a new [`ColorWheel`] using the default parameters.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`ColorWheel`] with a custom starting point.
    pub fn with_starting_point(mut start: f32) -> Self {
        if start >= 1.0 {
            start -= 1.0;
        }
        let rng = rand::rngs::SmallRng::from_entropy();
        Self { phase: start, rng }
    }
}

impl Iterator for ColorWheel {
    type Item = Color;

    fn next(&mut self) -> Option<Self::Item> {
        let shift: f32 = self.rng.gen_range(0.1..0.2);
        self.phase += shift;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        Some(Color::from_hsv(self.phase, 1.0, 0.8))
    }
}

impl Default for ColorWheel {
    fn default() -> Self {
        let rng = rand::rngs::SmallRng::from_entropy();
        Self { phase: 0.0, rng }
    }
}

/// This is an iterator that allows you to iterate over minor color variation between a starting
/// color and an ending color. You can only initialize this with [`range_to()`](Color#method.range_to).
///
/// # Example
///
/// Here is the color scale between red and lime:
///
/// ```
/// use octarine::Color;
///
/// let c0 = Color::from_web_color("red").unwrap();
/// let c1 = Color::from_hex(0xFF7F00);
/// let c2 = Color::from_web_color("yellow").unwrap();
/// let c3 = Color::new(128, 255, 0);
/// let c4 = Color::from_web_color("lime").unwrap();
/// let mut range_to = c0.range_to(c4.clone(), 5);
///
/// assert_eq!(Some(c0), range_to.next());
/// assert_eq!(Some(c1), range_to.next());
/// assert_eq!(Some(c2), range_to.next());
/// assert_eq!(Some(c3), range_to.next());
/// assert_eq!(Some(c4), range_to.next());
/// assert_eq!(None, range_to.next());
/// ```
///
/// You can also iterate through different amounts of gray between black and white.
///
/// ```
/// use octarine::Color;
///
/// let c0 = Color::from_web_color("black").unwrap();
/// let c1 = Color::from_hex(0x333333);
/// let c2 = Color::from_hex(0x666666);
/// let c3 = Color::from_hex(0x999999);
/// let c4 = Color::from_hex(0xCCCCCC);
/// let c5 = Color::from_web_color("white").unwrap();
/// let mut range_to = c0.range_to(c5.clone(), 6);
///
/// assert_eq!(Some(c0), range_to.next());
/// assert_eq!(Some(c1), range_to.next());
/// assert_eq!(Some(c2), range_to.next());
/// assert_eq!(Some(c3), range_to.next());
/// assert_eq!(Some(c4), range_to.next());
/// assert_eq!(Some(c5), range_to.next());
/// assert_eq!(None, range_to.next());
/// ```
#[derive(Debug, Clone)]
pub struct ColorRange {
    total_steps: usize,
    current_step: usize,
    step: (f32, f32, f32),
    start_color_hsl: (f32, f32, f32),
}

impl ColorRange {
    fn new(start_color: Color, end_color: Color, steps: usize) -> Self {
        let nb = steps.checked_sub(1).expect(&format!(
            "Unsupported negative number of colors: {steps} - 1"
        ));
        let start_color = start_color.to_hsl();
        let end_color = end_color.to_hsl();
        let s0 = (end_color.0 - start_color.0) / nb as f32;
        let s1 = (end_color.1 - start_color.1) / nb as f32;
        let s2 = (end_color.2 - start_color.2) / nb as f32;
        let step = if nb > 0 {
            (s0, s1, s2)
        } else {
            (0.0, 0.0, 0.0)
        };
        Self {
            total_steps: steps,
            current_step: 0,
            step,
            start_color_hsl: start_color,
        }
    }
}

impl Iterator for ColorRange {
    type Item = Color;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_step == self.total_steps {
            return None;
        }
        let m0 = self.step.0 * self.current_step as f32;
        let m1 = self.step.1 * self.current_step as f32;
        let m2 = self.step.2 * self.current_step as f32;
        let v0 = self.start_color_hsl.0 + m0;
        let v1 = self.start_color_hsl.1 + m1;
        let v2 = self.start_color_hsl.2 + m2;
        let color = Color::from_hsl(v0, v1, v2);
        self.current_step += 1;
        Some(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_rgb_float() {
        let canonical = constants::primary::RED;
        let from_float = Color::from_rgb_float(1.0, 0.0, 0.0);
        assert_eq!(canonical, from_float);
    }

    #[test]
    fn from_hex() {
        let canonical = constants::primary::RED;
        let from_hex = Color::from_hex(0xFF0000);
        assert_eq!(canonical, from_hex);
    }

    #[test]
    fn from_hsl() {
        let canonical = constants::primary::RED;
        let from_hsl = Color::from_hsl(0.0, 1.0, 0.5);
        assert_eq!(canonical, from_hsl);
    }

    #[test]
    fn from_hsv() {
        let canonical = constants::primary::RED;
        let from_hsv = Color::from_hsv(0.0, 1.0, 1.0);
        assert_eq!(canonical, from_hsv);
    }

    #[test]
    fn from_web_color() {
        let canonical = constants::primary::RED;
        let from_web_color = Color::from_web_color("red");
        assert_eq!(Some(canonical), from_web_color);
    }

    #[test]
    fn screen() {
        let canonical = Color::from_hex(0xFF9D9D);
        let screen = Color::from_hex(0xFF9999).screen(Color::new(10, 10, 10));
        assert_eq!(canonical, screen);
    }

    #[test]
    fn difference() {
        let canonical = Color::from_hex(0xF58F8F);
        let difference = Color::from_hex(0xFF9999).difference(Color::new(10, 10, 10));
        assert_eq!(canonical, difference);
    }

    #[test]
    fn overlay() {
        let canonical = Color::new(255, 156, 156);
        let overlay = Color::from_hex(0xFF9999).overlay(Color::new(10, 10, 10));
        assert_eq!(canonical, overlay);
    }

    #[test]
    fn invert() {
        let canonical = constants::primary::BLACK;
        let invert = constants::primary::WHITE.invert();
        assert_eq!(canonical, invert);
    }

    #[test]
    fn random_color() {
        Color::random_color();
    }

    #[test]
    fn to_hex() {
        let canonical = 0x646464;
        let to_hex = Color::new(100, 100, 100).to_hex();
        assert_eq!(canonical, to_hex);
    }

    #[test]
    fn to_hsl() {
        let canonical = (0.0, 0.0, 0.39215687);
        let to_hsl = Color::new(100, 100, 100).to_hsl();
        assert_eq!(canonical, to_hsl);
    }

    #[test]
    fn to_hsv() {
        let canonical = (0.0, 0.0, 0.392156862745);
        let to_hsv = Color::from_hex(0x646464).to_hsv();
        assert_eq!(canonical, to_hsv);
    }

    #[test]
    fn getters() {
        let color = Color::new(100, 100, 100);
        let canonical_rgb = color.to_rgb();
        let rgb = (color.get_red(), color.get_green(), color.get_blue());
        assert_eq!(canonical_rgb, rgb);
        let canonical_hsl = color.to_hsl();
        let hsl = (
            color.get_hsl_hue(),
            color.get_hsl_saturation(),
            color.get_hsl_luminance(),
        );
        assert_eq!(canonical_hsl, hsl);
        let canonical_hsv = color.to_hsv();
        let hsv = (
            color.get_hsv_hue(),
            color.get_hsv_saturation(),
            color.get_hsv_value(),
        );
        assert_eq!(canonical_hsv, hsv);
        let canonical_web_color = color.get_web_color();
        assert_eq!(canonical_web_color, None);
    }

    #[test]
    fn range_to() {
        let c0 = Color::from_web_color("red").unwrap();
        let c1 = Color::from_hex(0xFF7F00);
        let c2 = Color::from_web_color("yellow").unwrap();
        let c3 = Color::new(128, 255, 0);
        let c4 = Color::from_web_color("lime").unwrap();
        let mut range_to = c0.range_to(c4.clone(), 5);
        assert_eq!(Some(c0), range_to.next());
        assert_eq!(Some(c1), range_to.next());
        assert_eq!(Some(c2), range_to.next());
        assert_eq!(Some(c3), range_to.next());
        assert_eq!(Some(c4), range_to.next());
        assert_eq!(None, range_to.next());
        let c0 = Color::from_web_color("black").unwrap();
        let c1 = Color::from_hex(0x333333);
        let c2 = Color::from_hex(0x666666);
        let c3 = Color::from_hex(0x999999);
        let c4 = Color::from_hex(0xCCCCCC);
        let c5 = Color::from_web_color("white").unwrap();
        let mut range_to = c0.range_to(c5.clone(), 6);
        assert_eq!(Some(c0), range_to.next());
        assert_eq!(Some(c1), range_to.next());
        assert_eq!(Some(c2), range_to.next());
        assert_eq!(Some(c3), range_to.next());
        assert_eq!(Some(c4), range_to.next());
        assert_eq!(Some(c5), range_to.next());
        assert_eq!(None, range_to.next());
    }

    #[test]
    fn equality() {
        let color = Color::new(100, 100, 100);
        assert_eq!(color, Color::from_hex(0x646464));
        assert_eq!(Color::from_hsv(0.0, 1.0, 1.0), Color::new(255, 0, 0));
        assert!(color.complex_eq(&color, Equivalence::HSL));
        assert!(color.complex_eq(&color, Equivalence::HSV));
    }

    #[test]
    fn multiply() {
        let canonical = Color::new(204, 122, 122);
        let multiply = Color::from_hex(0xFF9999) * Color::from_hex(0xCCCCCC);
        assert_eq!(canonical, multiply);
        let canonical = Color::from_hex(0x640000);
        let multiply = Color::new(100, 100, 100) * Color::from_hsv(0.0, 1.0, 1.0);
        assert_eq!(canonical, multiply);
    }

    #[test]
    fn add() {
        let canonical = Color::new(255, 163, 163);
        let add = Color::from_hex(0xFF9999) + Color::new(10, 10, 10);
        assert_eq!(canonical, add);
        let canonical = Color::new(180, 255, 214);
        let add = Color::from_hex(0xAAFFCC) + Color::new(10, 10, 10);
        assert_eq!(canonical, add);
    }

    #[test]
    fn subtract() {
        let canonical = Color::new(245, 143, 143);
        let subtract = Color::from_hex(0xFF9999) - Color::new(10, 10, 10);
        assert_eq!(canonical, subtract);
        let canonical = Color::new(160, 245, 194);
        let subtract = Color::from_hex(0xAAFFCC) - Color::new(10, 10, 10);
        assert_eq!(canonical, subtract);
    }

    #[test]
    fn divide() {
        let canonical = Color::new(25, 15, 15);
        let divide = Color::from_hex(0xFF9999) / Color::new(10, 10, 10);
        assert_eq!(canonical, divide);
        let canonical = Color::new(17, 25, 20);
        let divide = Color::from_hex(0xAAFFCC) / Color::new(10, 10, 10);
        assert_eq!(canonical, divide);
    }

    #[test]
    fn color_wheel() {
        let mut color_wheel = ColorWheel::new();
        color_wheel.next();
        color_wheel = ColorWheel::with_starting_point(3.0);
        color_wheel.next();
    }

    #[test]
    fn setter() {
        let mut color = Color::random_color();
        let canonical = (3, 3, 3);
        color.set_red(3);
        color.set_green(3);
        color.set_blue(3);
        assert_eq!(canonical, color.to_rgb());
    }

    #[test]
    fn sanity() {
        let color = Color::from_rgb_float(0.5, 0.5, 0.5);
        assert_eq!(color.to_rgb(), (128, 128, 128));
        let color = Color::new(100, 100, 100);
        assert_eq!(color.to_rgb_float(), (0.39215687, 0.39215687, 0.39215687));
        let color = Color::from_hsl(0.5, 0.5, 0.5);
        assert_eq!(color.to_hsl(), (0.5, 0.49803922, 0.5));
        let color = Color::from_hsv(0.5, 0.5, 0.5);
        assert_eq!(color.to_hsv(), (0.5, 0.5, 0.5019608));
        let color = Color::from_hex(0x8F8F8F).to_hex();
        assert_eq!(0x8F8F8F, color);
    }
}
