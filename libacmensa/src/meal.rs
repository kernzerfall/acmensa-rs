use crate::DeEnStr;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt::Display, str::FromStr};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

#[derive(Hash, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum MealType {
    /// Classic dish (usually includes meat).
    Klassiker,
    /// Stew (incl. vegan options).
    Tellergericht,
    /// Suggestion of the day (incl. vegan options).
    Empfehlung,
    /// Wok (incl. vegan options).
    Wok,
    /// Classic burgers (Cheeseburger/Veggieburger/Chicken burger).
    BurgerClassics,
    /// Burger of the week (special offer).
    BurgerWoche,
    /// Pizza of the day (special offer).
    PizzaTag,
    /// Standard vegetarian meal.
    Vegetarisch,
    /// Catch-all for categories that could not be parsed.
    Unbekannt,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum SideType {
    /// Main side (Sättigungsbeilage).
    Main,
    /// Secondary side (Gemüsebeilage).
    Secondary,
    /// Catch-all for sides that could not be parsed.
    Unknown,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MealInfo {
    /// Type of meal (e.g. Klassiker).
    #[serde(rename = "type")]
    pub typ: MealType,

    /// Main meal description.
    pub text: String,

    /// Secondary meal description (e.g. sauces, sides etc.).
    pub subtext: String,

    /// Meal price (unparsed directly from site). Usually has the form "{:.02f} €".
    pub price: String,

    /// Sorted, deduplicated list of allergens.
    pub allergens: AllergenList,

    /// Vegan indication. Especially needed since sometimes "normal" meals are
    /// hijacked and replaced with vegan ones. Secondary heuristics are defined
    /// in `libacmensa::scrape::vegan_detektiv`.
    pub vegan: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SideAlternative {
    /// Main alternative description.
    pub text: String,

    /// Sorted, deduplicated list of allergens.
    pub allergens: AllergenList,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SideInfo {
    /// Type of side (Sättigungs-/Gemüsebeilage)
    #[serde(rename = "type")]
    pub typ: SideType,

    /// Alternative options for side.
    pub alternatives: Vec<SideAlternative>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct AllergenList(BTreeSet<String>);

lazy_static! {
    /// Type<->Name matcher definitions
    static ref NAMES_MAIN: Vec<(MealType, DeEnStr<&'static str>)> = vec![
        (
            // This needs to be above Klassiker to avoid "Classics" matching
            // it in English.
            MealType::BurgerClassics,
            DeEnStr {
                de: "Burger Classics",
                en: "Burger Classics"
            }
        ),
        (
            MealType::BurgerWoche,
            DeEnStr {
                de: "Burger der Woche",
                en: "Burger of the week"
            }
        ),
        (
            MealType::Tellergericht,
            DeEnStr {
                de: "Tellergericht",
                en: "Stew"
            }
        ),
        (
            MealType::PizzaTag,
            DeEnStr {
                de: "Pizza des Tages",
                en: "Pizza of the Day"
            }
        ),
        (
            MealType::Vegetarisch,
            DeEnStr {
                de: "Vegetarisch",
                en: "Vegetarian"
            }
        ),
        (
            MealType::Empfehlung,
            DeEnStr {
                de: "Empfehlung des Tages",
                en: "Suggestion of the day"
            }
        ),
        (
            MealType::Klassiker,
            DeEnStr {
                de: "Klassiker",
                en: "Classics"
            }
        ),
        (
            MealType::Wok,
            DeEnStr {
                de: "Wok",
                en: "Wok"
            }
        )
    ];

    /// Name <-> SideType matcher definitions
    static ref NAMES_SIDE: Vec<(SideType, DeEnStr<&'static str>)> = vec![
        (
            SideType::Main,
            DeEnStr {
                de: "Sättigungsbeilage",
                en: "Main side-dish"
            }
        ),
        (
            SideType::Main,
            DeEnStr {
                de: "Hauptbeilage",
                en: "Main side-dish"
            }
        ),
        (
            SideType::Secondary,
            DeEnStr {
                de: "Gemüsebeilage",
                en: "Secondary"
            }
        ),
        (
            SideType::Secondary,
            DeEnStr {
                de: "Nebenbeilage",
                en: "Secondary"
            }
        )
    ];
}

impl MealType {
    pub fn name(&self, english: bool) -> &str {
        let name = NAMES_MAIN
            .iter()
            .find(|(n, _)| n == self)
            .map(|(_, s)| s)
            .unwrap_or(&DeEnStr {
                de: "Unbekannt",
                en: "Unknown",
            });
        if english { name.en } else { name.de }
    }
}

impl SideType {
    pub fn name(&self, english: bool) -> &str {
        let name = NAMES_SIDE
            .iter()
            .find(|(n, _)| n == self)
            .map(|(_, s)| s)
            .unwrap_or(&DeEnStr {
                de: "Unbekannt",
                en: "Unknown",
            });
        if english { name.en } else { name.de }
    }
}

impl FromStr for MealType {
    type Err = anyhow::Error;

    /// Get MealType from Str. CANNOT FAIL. Safe to unwrap.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        for it in NAMES_MAIN.iter() {
            if s.contains(&it.1.en.to_lowercase()) || s.contains(&it.1.de.to_lowercase()) {
                return Ok(it.0.clone());
            }
        }

        Ok(MealType::Unbekannt)
    }
}

impl MealType {
    pub fn infer(s: &str) -> Self {
        //  SAFETY: See implementation of `FromStr` for `Self`
        unsafe { Self::from_str(s).unwrap_unchecked() }
    }
}

impl FromStr for SideType {
    type Err = anyhow::Error;

    /// Get SideType from Str. CANNOT FAIL. Safe to unwrap.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        for it in NAMES_SIDE.iter() {
            if s.contains(&it.1.en.to_lowercase()) || s.contains(&it.1.de.to_lowercase()) {
                return Ok(it.0.clone());
            }
        }

        Ok(SideType::Unknown)
    }
}

impl SideType {
    pub fn infer(s: &str) -> Self {
        //  SAFETY: See implementation of `FromStr` for `Self`
        unsafe { Self::from_str(s).unwrap_unchecked() }
    }
}

impl Display for AllergenList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.iter().fold(String::new(), |mut res, item| {
                if !res.is_empty() {
                    res.push_str(", ");
                }

                res.push_str(item);
                res
            }),
        )
    }
}

#[cfg(feature = "scrape")]
impl From<(&regex::Regex, &str)> for AllergenList {
    fn from((regex, s): (&regex::Regex, &str)) -> Self {
        Self(
            regex
                .captures_iter(s)
                .filter_map(|caps| caps.get(1).map(|m| m.as_str()))
                .flat_map(|s| s.split(","))
                .map(str::to_string)
                .collect(),
        )
    }
}

impl AllergenList {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
