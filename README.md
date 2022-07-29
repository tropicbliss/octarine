# octarine

Converts colors to many common color representations (e.g. RGB, HSL, HSV) and does simple
colour manipulation operations. Thanks to [colour](https://pypi.org/project/colour/) and
[colors.py](https://pypi.org/project/colors.py/) for inspiring the API (and documentation) of this project.

# Features:

- Extremely simple API (subjective).
- Convert between RGB, HSL, HSV, W3C web colors, and hexadecimal.
- One struct `Color` to rule them all.
- Perform arithmetic, blend modes, and generate random colors within boundaries.
- [Octarine](https://discworld.fandom.com/wiki/Octarine).

# Examples

```rs
use octarine::Color;

let color1 = Color::from_web_color("red");
let color2 = Color::new(255, 0, 0);
assert_eq!(color1, Some(color2));

let hex = color2.to_hex();
assert_eq!(hex, 0xFF0000);

let red = color2.get_red();
let green = color2.get_green();
let blue = color2.get_blue();
assert_eq!(color2.to_rgb(), (red, green, blue));

let hue = color2.get_hsl_hue();
let saturation = color2.get_hsl_saturation();
let luminance = color2.get_hsl_luminance();
assert_eq!(color2.to_hsl(), (hue, saturation, luminance));

let hue = color2.get_hsv_hue();
let saturation = color.get_hsv_saturation();
let value = color.get_hsv_value();
assert_eq!(color2.to_hsv(), (hue, saturation, value));
```
