use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FillStyle {
    Hachure,
    #[serde(rename = "cross-hatch")]
    CrossHatch,
    Solid,
    Zigzag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StrokeStyle {
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StrokeRoundness {
    Round,
    Sharp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundnessType {
    Legacy,
    ProportionalRadius,
    AdaptiveRadius,
}

impl RoundnessType {
    /// The number upstream stores in `roundness.type`.
    ///
    /// `ROUNDNESS` in `packages/common/src/constants.ts`. Upstream writes
    /// `{ type: 3 }` for a plain rounded rectangle, so a name would not be
    /// understood by excalidraw.com.
    pub fn code(self) -> u32 {
        match self {
            Self::Legacy => 1,
            Self::ProportionalRadius => 2,
            Self::AdaptiveRadius => 3,
        }
    }

    /// Inverse of [`RoundnessType::code`].
    ///
    /// An unknown algorithm falls back to proportional radius: legacy rounding
    /// is defined to behave the same way, so it is the safe generic bucket.
    pub fn from_code(code: u32) -> Self {
        match code {
            1 => Self::Legacy,
            3 => Self::AdaptiveRadius,
            _ => Self::ProportionalRadius,
        }
    }
}

impl Serialize for RoundnessType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.code())
    }
}

impl<'de> Deserialize<'de> for RoundnessType {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RoundnessTypeVisitor;

        impl de::Visitor<'_> for RoundnessTypeVisitor {
            type Value = RoundnessType;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a roundness type code")
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<RoundnessType, E> {
                Ok(RoundnessType::from_code(value as u32))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<RoundnessType, E> {
                Ok(RoundnessType::from_code(value as u32))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<RoundnessType, E> {
                Ok(RoundnessType::from_code(value as u32))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<RoundnessType, E> {
                Ok(match value {
                    "legacy" => RoundnessType::Legacy,
                    "adaptiveRadius" => RoundnessType::AdaptiveRadius,
                    _ => RoundnessType::ProportionalRadius,
                })
            }
        }

        deserializer.deserialize_any(RoundnessTypeVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Roundness {
    #[serde(rename = "type")]
    pub kind: RoundnessType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontFamily {
    Virgil,
    Helvetica,
    Cascadia,
    Assistant,
    Excalifont,
    ComicShanns,
    Liberation,
    Nunito,
    Lilita,
    Xiaolai,
}

impl FontFamily {
    /// The number upstream writes into `fontFamily`.
    ///
    /// `FONT_FAMILY` in `packages/common/src/constants.ts`, plus `Xiaolai`,
    /// which upstream only knows as the CJK hand-drawn fallback
    /// (`FONT_FAMILY_FALLBACKS`). The code is what a `.excalidraw` file
    /// carries, so the enum has to round-trip through these numbers or the
    /// font is lost between the two applications.
    pub fn code(self) -> u32 {
        match self {
            Self::Virgil => 1,
            Self::Helvetica => 2,
            Self::Cascadia => 3,
            Self::Excalifont => 5,
            Self::Nunito => 6,
            Self::Lilita => 7,
            Self::ComicShanns => 8,
            Self::Liberation => 9,
            Self::Assistant => 10,
            Self::Xiaolai => 100,
        }
    }

    /// Inverse of [`FontFamily::code`].
    ///
    /// Anything unrecognised becomes `DEFAULT_FONT_FAMILY`, the same fallback
    /// upstream's `getFontFamilyByName` applies. `4` is deliberately absent:
    /// upstream reserves it for the retired Assistant/custom-font slot.
    pub fn from_code(code: u32) -> Self {
        match code {
            1 => Self::Virgil,
            2 => Self::Helvetica,
            3 => Self::Cascadia,
            5 => Self::Excalifont,
            6 => Self::Nunito,
            7 => Self::Lilita,
            8 => Self::ComicShanns,
            9 => Self::Liberation,
            10 => Self::Assistant,
            100 => Self::Xiaolai,
            _ => Self::Excalifont,
        }
    }

    /// Names upstream accepts for a family, used by `getFontFamilyByName` when
    /// an old file stored the name instead of the code.
    fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "virgil" => Some(Self::Virgil),
            "helvetica" => Some(Self::Helvetica),
            "cascadia" | "cascadia code" => Some(Self::Cascadia),
            "assistant" => Some(Self::Assistant),
            "excalifont" => Some(Self::Excalifont),
            "comic shanns" | "comicshanns" => Some(Self::ComicShanns),
            "liberation sans" | "liberation" => Some(Self::Liberation),
            "nunito" => Some(Self::Nunito),
            "lilita one" | "lilita" => Some(Self::Lilita),
            "xiaolai" | "xiaolai sc" => Some(Self::Xiaolai),
            _ => None,
        }
    }
}

impl Serialize for FontFamily {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.code())
    }
}

impl<'de> Deserialize<'de> for FontFamily {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FontFamilyVisitor;

        impl de::Visitor<'_> for FontFamilyVisitor {
            type Value = FontFamily;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a font family code or name")
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<FontFamily, E> {
                Ok(FontFamily::from_code(value as u32))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<FontFamily, E> {
                Ok(FontFamily::from_code(value as u32))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> Result<FontFamily, E> {
                Ok(FontFamily::from_code(value as u32))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<FontFamily, E> {
                Ok(value
                    .parse::<u32>()
                    .ok()
                    .map(FontFamily::from_code)
                    .or_else(|| FontFamily::from_name(value))
                    .unwrap_or(FontFamily::Excalifont))
            }

            fn visit_none<E: de::Error>(self) -> Result<FontFamily, E> {
                Ok(FontFamily::Excalifont)
            }

            fn visit_unit<E: de::Error>(self) -> Result<FontFamily, E> {
                Ok(FontFamily::Excalifont)
            }
        }

        deserializer.deserialize_any(FontFamilyVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PointerType {
    Mouse,
    Pen,
    Touch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StrokeVariability {
    Variable,
    Constant,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StrokeOptions {
    #[serde(default = "default_variability")]
    pub variability: StrokeVariability,
    #[serde(default = "default_streamline")]
    pub streamline: f64,
}

/// `restoreFreedrawStrokeOptions` in upstream's `restore.ts` defaults each key on
/// its own, because freedraw elements written before streamline was introduced
/// carry `variability` only.
fn default_variability() -> StrokeVariability {
    StrokeVariability::Variable
}

/// Upstream's `DEFAULT_STROKE_STREAMLINE`.
fn default_streamline() -> f64 {
    0.3
}

impl Default for StrokeOptions {
    fn default() -> Self {
        Self {
            variability: default_variability(),
            streamline: default_streamline(),
        }
    }
}
