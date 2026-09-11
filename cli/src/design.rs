//! `design lint`: colours and type in frontend files resolve to
//! `brand/tokens.json`.
//!
//! The token file flattens to `TOKEN-<group>-<leaf>`, and the CSS custom
//! property of the same leaf is `--<group>-<leaf>`. Every rule below is
//! textual: nothing parses CSS or JSX.

use crate::errors::CliError;
use std::collections::BTreeMap;
use std::path::Path;

/// The six token groups. `meta` is the seventh top-level key and is not one.
pub const GROUPS: &[&str] = &["color", "type", "spacing", "radius", "elevation", "motion"];

/// The values a colour or type property may carry instead of a token.
pub const ALLOWED_KEYWORDS: &[&str] = &[
    "transparent",
    "currentcolor",
    "inherit",
    "none",
    "normal",
    "unset",
    "initial",
];

/// The 148 CSS named colours of CSS Color 4, lowercased. `transparent` is a
/// keyword rather than a named colour and is absent.
pub const NAMED_COLOURS: &[&str] = &[
    "aliceblue",
    "antiquewhite",
    "aqua",
    "aquamarine",
    "azure",
    "beige",
    "bisque",
    "black",
    "blanchedalmond",
    "blue",
    "blueviolet",
    "brown",
    "burlywood",
    "cadetblue",
    "chartreuse",
    "chocolate",
    "coral",
    "cornflowerblue",
    "cornsilk",
    "crimson",
    "cyan",
    "darkblue",
    "darkcyan",
    "darkgoldenrod",
    "darkgray",
    "darkgreen",
    "darkgrey",
    "darkkhaki",
    "darkmagenta",
    "darkolivegreen",
    "darkorange",
    "darkorchid",
    "darkred",
    "darksalmon",
    "darkseagreen",
    "darkslateblue",
    "darkslategray",
    "darkslategrey",
    "darkturquoise",
    "darkviolet",
    "deeppink",
    "deepskyblue",
    "dimgray",
    "dimgrey",
    "dodgerblue",
    "firebrick",
    "floralwhite",
    "forestgreen",
    "fuchsia",
    "gainsboro",
    "ghostwhite",
    "gold",
    "goldenrod",
    "gray",
    "green",
    "greenyellow",
    "grey",
    "honeydew",
    "hotpink",
    "indianred",
    "indigo",
    "ivory",
    "khaki",
    "lavender",
    "lavenderblush",
    "lawngreen",
    "lemonchiffon",
    "lightblue",
    "lightcoral",
    "lightcyan",
    "lightgoldenrodyellow",
    "lightgray",
    "lightgreen",
    "lightgrey",
    "lightpink",
    "lightsalmon",
    "lightseagreen",
    "lightskyblue",
    "lightslategray",
    "lightslategrey",
    "lightsteelblue",
    "lightyellow",
    "lime",
    "limegreen",
    "linen",
    "magenta",
    "maroon",
    "mediumaquamarine",
    "mediumblue",
    "mediumorchid",
    "mediumpurple",
    "mediumseagreen",
    "mediumslateblue",
    "mediumspringgreen",
    "mediumturquoise",
    "mediumvioletred",
    "midnightblue",
    "mintcream",
    "mistyrose",
    "moccasin",
    "navajowhite",
    "navy",
    "oldlace",
    "olive",
    "olivedrab",
    "orange",
    "orangered",
    "orchid",
    "palegoldenrod",
    "palegreen",
    "paleturquoise",
    "palevioletred",
    "papayawhip",
    "peachpuff",
    "peru",
    "pink",
    "plum",
    "powderblue",
    "purple",
    "rebeccapurple",
    "red",
    "rosybrown",
    "royalblue",
    "saddlebrown",
    "salmon",
    "sandybrown",
    "seagreen",
    "seashell",
    "sienna",
    "silver",
    "skyblue",
    "slateblue",
    "slategray",
    "slategrey",
    "snow",
    "springgreen",
    "steelblue",
    "tan",
    "teal",
    "thistle",
    "tomato",
    "turquoise",
    "violet",
    "wheat",
    "white",
    "whitesmoke",
    "yellow",
    "yellowgreen",
];

/// The flattened token file.
#[derive(Debug, Clone, Default)]
pub struct Tokens {
    /// `TOKEN-<group>-<leaf>` to the values it carries: one for every group
    /// but `color`, which carries the `light` and the `dark` value.
    pub values: BTreeMap<String, Vec<String>>,
    /// The group each token belongs to.
    pub group: BTreeMap<String, String>,
}

impl Tokens {
    /// True when the flattened name is defined.
    pub fn has(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    /// The token of a CSS custom property `--<group>-<leaf>`.
    pub fn by_property(&self, property: &str) -> Option<&String> {
        let name = format!("TOKEN-{property}");
        self.values.get_key_value(&name).map(|(k, _)| k)
    }

    /// Every token of one group.
    pub fn of_group<'a>(&'a self, group: &str) -> Vec<(&'a String, &'a Vec<String>)> {
        self.values
            .iter()
            .filter(|(k, _)| self.group.get(*k).map(String::as_str) == Some(group))
            .collect()
    }
}

/// One violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// The project-relative path.
    pub path: String,
    /// The one-based line.
    pub line: u32,
    /// The `DFA-` code.
    pub code: &'static str,
    /// The offending value.
    pub value: String,
    /// The token named in the message.
    pub nearest: String,
    /// The rendered message.
    pub message: String,
}

/// Read and flatten the token file.
pub fn load_tokens(root: &Path, rel: &str) -> Result<(Tokens, serde_json::Value), CliError> {
    let path = crate::project::resolve_doc_path(root, rel.trim_start_matches(".devforgeai/"));
    let path = if path.exists() { path } else { root.join(rel) };
    if !path.exists() {
        return Err(CliError::at(
            "DFA-E120",
            format!("{rel} not found; the Design skill produces it"),
            rel.to_string(),
        ));
    }
    let text = crate::project::read_doc(&path)?;
    let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
        CliError::at(
            "DFA-E121",
            format!("brand/tokens.json is not valid JSON: {e}"),
            rel.to_string(),
        )
    })?;
    Ok((flatten(&value), value))
}

/// Flatten the six groups, ignoring `meta` and any leaf of the wrong shape.
pub fn flatten(value: &serde_json::Value) -> Tokens {
    let mut t = Tokens::default();
    for group in GROUPS {
        let Some(obj) = value.get(group).and_then(|g| g.as_object()) else {
            continue;
        };
        for (leaf, v) in obj {
            let name = format!("TOKEN-{group}-{leaf}");
            let values: Vec<String> = if *group == "color" {
                match (
                    v.get("light").and_then(|x| x.as_str()),
                    v.get("dark").and_then(|x| x.as_str()),
                ) {
                    (Some(l), Some(d)) => vec![l.to_string(), d.to_string()],
                    _ => continue,
                }
            } else {
                match v.as_str() {
                    Some(s) => vec![s.to_string()],
                    None => continue,
                }
            };
            t.values.insert(name.clone(), values);
            t.group.insert(name, (*group).to_string());
        }
    }
    t
}

// ------------------------------------------------------------------ colours

/// A colour value as sRGB, `None` when it is not one this comparison reads.
pub fn to_rgb(value: &str) -> Option<(f64, f64, f64)> {
    let v = value.trim().to_lowercase();
    if let Some(hex) = v.strip_prefix('#') {
        return from_hex(hex);
    }
    if let Some(rest) = v.strip_prefix("rgba(").or_else(|| v.strip_prefix("rgb(")) {
        let nums: Vec<f64> = rest
            .trim_end_matches(')')
            .split([',', ' ', '/'])
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.trim().trim_end_matches('%').parse::<f64>().ok())
            .collect();
        if nums.len() >= 3 {
            return Some((nums[0], nums[1], nums[2]));
        }
        return None;
    }
    named_rgb(&v)
}

fn from_hex(hex: &str) -> Option<(f64, f64, f64)> {
    let h: String = hex.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
    let full = match h.len() {
        3 | 4 => h[..3]
            .chars()
            .map(|c| format!("{c}{c}"))
            .collect::<String>(),
        6 | 8 => h[..6].to_string(),
        _ => return None,
    };
    let n = u32::from_str_radix(&full, 16).ok()?;
    Some((
        ((n >> 16) & 0xff) as f64,
        ((n >> 8) & 0xff) as f64,
        (n & 0xff) as f64,
    ))
}

/// The sRGB of one of the named colours, from a small table of the ones a
/// distance comparison is likely to reach; every other name resolves through
/// its own hex when the token file carries one.
fn named_rgb(name: &str) -> Option<(f64, f64, f64)> {
    match name {
        "black" => Some((0.0, 0.0, 0.0)),
        "white" => Some((255.0, 255.0, 255.0)),
        "red" => Some((255.0, 0.0, 0.0)),
        "lime" | "green" if name == "lime" => Some((0.0, 255.0, 0.0)),
        "green" => Some((0.0, 128.0, 0.0)),
        "blue" => Some((0.0, 0.0, 255.0)),
        "yellow" => Some((255.0, 255.0, 0.0)),
        "cyan" | "aqua" => Some((0.0, 255.0, 255.0)),
        "magenta" | "fuchsia" => Some((255.0, 0.0, 255.0)),
        "gray" | "grey" => Some((128.0, 128.0, 128.0)),
        "silver" => Some((192.0, 192.0, 192.0)),
        "maroon" => Some((128.0, 0.0, 0.0)),
        "olive" => Some((128.0, 128.0, 0.0)),
        "navy" => Some((0.0, 0.0, 128.0)),
        "purple" => Some((128.0, 0.0, 128.0)),
        "teal" => Some((0.0, 128.0, 128.0)),
        "orange" => Some((255.0, 165.0, 0.0)),
        _ => None,
    }
}

/// The `color` token nearest `value` in sRGB, or `no token defined`.
pub fn nearest_colour(tokens: &Tokens, value: &str) -> String {
    let Some(want) = to_rgb(value) else {
        return nearest_any_colour(tokens);
    };
    let mut best: Option<(f64, &String)> = None;
    for (name, values) in tokens.of_group("color") {
        for v in values {
            let Some(have) = to_rgb(v) else { continue };
            let d =
                (want.0 - have.0).powi(2) + (want.1 - have.1).powi(2) + (want.2 - have.2).powi(2);
            if best.as_ref().map(|(bd, _)| d < *bd).unwrap_or(true) {
                best = Some((d, name));
            }
        }
    }
    match best {
        Some((_, name)) => name.clone(),
        None => nearest_any_colour(tokens),
    }
}

fn nearest_any_colour(tokens: &Tokens) -> String {
    tokens
        .of_group("color")
        .first()
        .map(|(n, _)| (*n).clone())
        .unwrap_or_else(|| "no token defined".to_string())
}

// --------------------------------------------------------------------- type

/// A length in pixels, at 16px per rem and per em.
pub fn to_px(value: &str) -> Option<f64> {
    let v = value.trim().to_lowercase();
    let number: String = v
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let n: f64 = number.parse().ok()?;
    let unit = v[number.len()..].trim();
    Some(match unit {
        "rem" | "em" => n * 16.0,
        "pt" => n * 96.0 / 72.0,
        "px" | "" => n,
        _ => return None,
    })
}

/// The `type` token numerically nearest `value`, or `no token defined`.
pub fn nearest_type(tokens: &Tokens, value: &str) -> String {
    let Some(want) = to_px(value) else {
        return first_type(tokens);
    };
    let mut best: Option<(f64, &String)> = None;
    for (name, values) in tokens.of_group("type") {
        for v in values {
            let Some(have) = to_px(v) else { continue };
            let d = (want - have).abs();
            if best.as_ref().map(|(bd, _)| d < *bd).unwrap_or(true) {
                best = Some((d, name));
            }
        }
    }
    match best {
        Some((_, name)) => name.clone(),
        None => first_type(tokens),
    }
}

fn first_type(tokens: &Tokens) -> String {
    tokens
        .of_group("type")
        .first()
        .map(|(n, _)| (*n).clone())
        .unwrap_or_else(|| "no token defined".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens() -> Tokens {
        flatten(&serde_json::json!({
            "meta": { "schema": "devforgeai/tokens/1" },
            "color": {
                "primary": { "light": "#3355ff", "dark": "#7788ff" },
                "surface": { "light": "#ffffff", "dark": "#101010" },
                "broken": { "light": "#000000" }
            },
            "type": { "body": "1rem", "heading": "2rem" },
            "spacing": { "sm": "4px" }
        }))
    }

    #[test]
    fn flattening_names_every_group_the_same_way() {
        let t = tokens();
        assert!(t.has("TOKEN-color-primary"));
        assert!(t.has("TOKEN-type-body"));
        assert!(t.has("TOKEN-spacing-sm"));
        assert!(
            !t.has("TOKEN-color-broken"),
            "a colour leaf without both themes is not a token"
        );
    }

    #[test]
    fn a_colour_carries_both_theme_values() {
        let t = tokens();
        assert_eq!(
            t.values["TOKEN-color-primary"],
            vec!["#3355ff".to_string(), "#7788ff".to_string()]
        );
    }

    #[test]
    fn a_custom_property_resolves_to_its_token() {
        let t = tokens();
        assert_eq!(
            t.by_property("color-primary").map(String::as_str),
            Some("TOKEN-color-primary")
        );
        assert_eq!(t.by_property("color-nowhere"), None);
    }

    #[test]
    fn hex_parses_in_three_four_six_and_eight_digit_forms() {
        assert_eq!(to_rgb("#fff"), Some((255.0, 255.0, 255.0)));
        assert_eq!(to_rgb("#ffffff"), Some((255.0, 255.0, 255.0)));
        assert_eq!(to_rgb("#ffffff80"), Some((255.0, 255.0, 255.0)));
        assert_eq!(to_rgb("#000"), Some((0.0, 0.0, 0.0)));
    }

    #[test]
    fn rgb_and_a_named_colour_parse() {
        assert_eq!(to_rgb("rgb(1, 2, 3)"), Some((1.0, 2.0, 3.0)));
        assert_eq!(to_rgb("rgba(1,2,3,0.5)"), Some((1.0, 2.0, 3.0)));
        assert_eq!(to_rgb("red"), Some((255.0, 0.0, 0.0)));
    }

    #[test]
    fn the_nearest_colour_is_the_smallest_srgb_distance() {
        let t = tokens();
        assert_eq!(nearest_colour(&t, "#3356ff"), "TOKEN-color-primary");
        assert_eq!(nearest_colour(&t, "#fefefe"), "TOKEN-color-surface");
    }

    #[test]
    fn a_file_with_no_colour_token_says_so() {
        let t = flatten(&serde_json::json!({ "type": { "body": "1rem" } }));
        assert_eq!(nearest_colour(&t, "#000000"), "no token defined");
    }

    #[test]
    fn lengths_convert_at_sixteen_pixels_per_rem() {
        assert_eq!(to_px("1rem"), Some(16.0));
        assert_eq!(to_px("2em"), Some(32.0));
        assert_eq!(to_px("24px"), Some(24.0));
        assert_eq!(to_px("12pt"), Some(16.0));
        assert_eq!(to_px("wide"), None);
    }

    #[test]
    fn the_nearest_type_is_the_smallest_pixel_distance() {
        let t = tokens();
        assert_eq!(nearest_type(&t, "17px"), "TOKEN-type-body");
        assert_eq!(nearest_type(&t, "30px"), "TOKEN-type-heading");
    }

    #[test]
    fn the_named_colour_list_holds_the_hundred_and_forty_eight() {
        assert_eq!(NAMED_COLOURS.len(), 148);
        assert!(NAMED_COLOURS.contains(&"rebeccapurple"));
        assert!(
            !NAMED_COLOURS.contains(&"transparent"),
            "transparent is a keyword, not a named colour"
        );
    }
}
