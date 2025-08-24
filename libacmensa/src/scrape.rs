use std::str::FromStr;

use crate::{
    config::{self, *},
    meal::{self, MealInfo, MealType, SideAlternative, SideInfo, SideType},
};
use lazy_static::lazy_static;
use regex::Regex;
use scraper::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json-schema")]
use schemars::{JsonSchema, schema_for};

/// Encapsulates DayData for five days (Mon..Fri)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeekData {
    pub main_dishes: [Vec<MealInfo>; config::OPEN_DAYS],
    pub side_dishes: [Vec<SideInfo>; config::OPEN_DAYS],
}

/// DayView is a view into a day of WeekData.
#[derive(Clone, Debug, Serialize)]
pub struct DayView<'a> {
    pub main_dishes: &'a Vec<MealInfo>,
    pub side_dishes: &'a Vec<SideInfo>,
}

/// DayData is like DayView but owns its data. It holds
/// data for dishes for a single day.
#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct DayData {
    /// Main dishes array (Klassiker, Vegetarisch, Wok, ...)
    pub main_dishes: Vec<MealInfo>,
    /// Side dishes array (Sättigungsbeilagen/Gemüsebeilagen)
    pub side_dishes: Vec<SideInfo>,
}

lazy_static! {
    static ref ALLERGEN_REGEX: Regex = Regex::from_str(r"\(([A-Z0-9,]*)\)").unwrap();
    static ref SPACE_REGEX: Regex = Regex::from_str(r"\s\s+").unwrap();
    static ref SEL_MAIN: Selector = Selector::parse("tr.main-dish").unwrap();
    static ref SEL_SIDE: Selector = Selector::parse("tr.side-dish").unwrap();
    static ref SEL_X_CELL: Selector = Selector::parse("td").unwrap();
    static ref SEL_X_DTEXT: Selector = Selector::parse(".dish-text").unwrap();
}

pub async fn get(mensa: &str, next_week: bool, english: bool) -> anyhow::Result<reqwest::Response> {
    let url = CONFIG.endpoint.host.clone()
        + "/"
        + &CONFIG.endpoint.menu.build_path(mensa, next_week, english);

    Ok(reqwest::get(url).await?)
}

/// Heuristic to catch as many veg* meals as possible.
pub async fn vegan_detektiv(typ: &MealType, html: &str) -> bool {
    if typ == &MealType::Vegetarisch {
        return true;
    }

    let t = html.to_lowercase();

    t.contains("vegan") || t.contains("vegetarian") || t.contains("vegetarisch")
}

/// Remove all allergen groups and trim extra spaces.
pub fn remove_allergens(text: &str) -> String {
    SPACE_REGEX
        .replace_all(&ALLERGEN_REGEX.replace_all(text, ""), " ")
        .trim()
        .to_string()
}

/// Collect all allergens and retun them in a sorted vector.
/// This is NOT a HashSet since we want it to be sorted and `Ord`.
pub fn collect_allergens(full_text: &str) -> meal::AllergenList {
    (&*ALLERGEN_REGEX, full_text).into()
}

/// Scrape a single page HTML for `WeekData`.
/// This should be one of the menu endpoints defined in `mensen.toml`.
/// It is language-agnostic, only relying on the structure of the meal table.
pub async fn scrape_page(html: &str) -> anyhow::Result<WeekData> {
    let dom = scraper::Html::parse_document(html);

    let main_rows = dom.select(&SEL_MAIN);
    let side_rows = dom.select(&SEL_SIDE);

    let mut main_dishes: [Vec<MealInfo>; config::OPEN_DAYS] = Default::default();
    let mut side_dishes: [Vec<SideInfo>; config::OPEN_DAYS] = Default::default();

    for (row_num, row) in main_rows.into_iter().enumerate() {
        let mut cells = row.select(&SEL_X_CELL);

        // Get and parse MealType from first cell
        let type_text = cells
            .next()
            .unwrap_or_else(|| {
                log::error!("[parse/main] could not get (row,col)=({row_num}, 0)",);
                panic!()
            })
            .inner_html();
        let typ = MealType::infer(&type_text);

        // Price should be in the first cell, after a line break
        let price = type_text
            .split_once("<br>")
            .unwrap_or(("", ""))
            .1
            .to_string();

        // Handle rest cells
        for col_num in 0..config::OPEN_DAYS {
            let curr = cells.next().unwrap_or_else(|| {
                log::error!(
                    "[parse/main] could not get (row,col)=({row_num}, {})",
                    col_num + 1 // +1 -> we have parsed the 0'th cell separately
                );
                panic!()
            });

            // There should only be one of these
            if let Some(dishtext) = curr.select(&SEL_X_DTEXT).next() {
                let mut text_iter = dishtext.text();

                // First field should be meal name
                let text_v = text_iter
                    .next()
                    .unwrap_or_else(|| {
                        log::error!(
                            "[parse/main] could not get main text field at \
                                (row,col)=({row_num}, {})",
                            col_num + 1
                        );
                        panic!()
                    })
                    .trim();
                // Rest of field should be secondary info about the mean (e.g. sauces)
                let subtext_v = text_iter.collect::<String>();

                main_dishes
                    .get_mut(col_num)
                    .unwrap_or_else(|| {
                        log::error!("[parse/main/internal] failed to get_mut({col_num}) on vector");
                        panic!()
                    })
                    .push(MealInfo {
                        // Type inferred above
                        typ: typ.clone(),
                        // Cleaned up meal description fields
                        text: remove_allergens(text_v),
                        subtext: remove_allergens(&subtext_v),
                        // Price inferred above (first cell w/ `MealType`)
                        price: price.clone(),
                        // Allergens separately
                        allergens: collect_allergens(&(text_v.to_string() + &subtext_v)),
                        vegan: vegan_detektiv(&typ, &curr.inner_html()).await,
                    });
            }
        }
    }

    for (row_num, row) in side_rows.into_iter().enumerate() {
        let mut cells = row.select(&SEL_X_CELL);

        // Get and parse `SideType` from first cell
        let type_text = cells
            .next()
            .unwrap_or_else(|| {
                log::error!(
                    "[parse/secondary] could not get \
                    (row,col)=({row_num}, 0)",
                );
                panic!()
            })
            .inner_html();
        let typ = SideType::infer(&type_text);

        for col_num in 0..config::OPEN_DAYS {
            let curr = cells.next().unwrap_or_else(|| {
                log::error!(
                    "[parse/secondary] could not get (row,col)=({row_num}, {})",
                    col_num + 1 // +1 -> we have parsed the 0'th cell separately
                );
                panic!()
            });

            side_dishes
                .get_mut(col_num)
                .unwrap_or_else(|| {
                    log::error!(
                        "[parse/secondary/internal] get_mut({col_num}) failed \
                        on vector"
                    );
                    panic!()
                })
                .push(SideInfo {
                    typ: typ.clone(),
                    alternatives: curr
                        .text()
                        .filter(|s| s.ne(&"oder") && s.ne(&"or"))
                        .map(|s| s.to_string())
                        .map(|s| SideAlternative {
                            text: remove_allergens(&s),
                            allergens: collect_allergens(&s),
                        })
                        .collect::<Vec<_>>(),
                });
        }
    }

    Ok(WeekData {
        main_dishes,
        side_dishes,
    })
}

impl WeekData {
    /// Returns a `DayView` into a single day of `WeekData`
    pub fn get_day<'a>(&'a self, day: usize) -> DayView<'a> {
        if day >= config::OPEN_DAYS {
            log::error!("Requested {day} > {}", config::OPEN_DAYS - 1);
            panic!()
        }

        DayView {
            main_dishes: &self.main_dishes[day],
            side_dishes: &self.side_dishes[day],
        }
    }

    /// Print json schema for `DayData`
    #[cfg(feature = "json-schema")]
    pub fn day_schema() -> anyhow::Result<String> {
        let schema = schema_for!(DayData);
        Ok(serde_json::to_string_pretty(&schema)?)
    }

    /// Sorts the dishes inside each day (order inherited from `MainInfo`, `SideInfo`)
    pub fn sorted(&self) -> Self {
        let mut main_dishes = self.main_dishes.clone();
        let mut side_dishes = self.side_dishes.clone();

        main_dishes.iter_mut().for_each(|m| m.sort());
        side_dishes.iter_mut().for_each(|m| m.sort());

        Self {
            main_dishes,
            side_dishes,
        }
    }
}
