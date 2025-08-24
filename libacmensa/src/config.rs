use lazy_static::lazy_static;
use serde::Deserialize;

use crate::DeEnStr;

const CONFIG_TOML: &str = include_str!("../../res/mensen.toml");
pub const OPEN_DAYS: usize = 5;

/// Master config for library
#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    /// Holds endpoint configurations
    pub endpoint: Endpoint,
}

/// Endpoint configuration
#[derive(Deserialize, Clone, Debug)]
pub struct Endpoint {
    /// Base part of the URL (e.g. `https://example.com`)
    pub host: String,

    /// Endpoint to grab timeplan
    #[allow(dead_code)] // timeplan is to be implemented
    pub timeplan: String,

    /// Templates for endpoints to mensa-specific menus
    pub menu: PathTemplate,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PathTemplate {
    /// Common prefix
    pub prefix: String,
    /// Mensa-specific suffix template. Differenent suffix for each language.
    pub suffix_template: DeEnStr<String>,
}

lazy_static! {
    pub static ref CONFIG: Config = toml::from_str(CONFIG_TOML).unwrap();
    pub static ref THIS_WEEK: DeEnStr<String> = DeEnStr {
        de: "diese".into(),
        en: "this".into()
    };
    pub static ref NEXT_WEEK: DeEnStr<String> = DeEnStr {
        de: "naechste".into(),
        en: "next".into()
    };
}

impl PathTemplate {
    pub fn fill_suffix_v(template: &str, name: &str, week: &str) -> String {
        template.replace("{{name}}", name).replace("{{week}}", week)
    }

    pub fn fill_suffix(&self, mensa: &str, next_week: bool, english: bool) -> String {
        let week_t = if next_week { &*NEXT_WEEK } else { &*THIS_WEEK };
        let (template, week) = if english {
            (&self.suffix_template.en, &week_t.en)
        } else {
            (&self.suffix_template.de, &week_t.de)
        };
        Self::fill_suffix_v(template, mensa, week)
    }

    pub fn build_path(&self, mensa: &str, next_week: bool, english: bool) -> String {
        self.prefix.clone() + "/" + &self.fill_suffix(mensa, next_week, english)
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    pub fn parse_cfg() {
        println!("{:#?}", *CONFIG);
    }

    #[test]
    pub fn gen_mensa_url_path() {
        let mensa = "academica";

        // DE, this week
        assert_eq!(
            CONFIG.endpoint.menu.build_path(&mensa, false, false),
            String::from_str("files/content/Downloads/Gastronomie/Speiseplaene/speiseplan_mensa_")
                .unwrap()
                + mensa
                + "_diese_woche.html"
        );

        // DE, next week
        assert_eq!(
            CONFIG.endpoint.menu.build_path(&mensa, true, false),
            String::from_str("files/content/Downloads/Gastronomie/Speiseplaene/speiseplan_mensa_")
                .unwrap()
                + mensa
                + "_naechste_woche.html"
        );

        // EN, this week
        assert_eq!(
            CONFIG.endpoint.menu.build_path(&mensa, false, true),
            String::from_str("files/content/Downloads/Gastronomie/Speiseplaene/menu_mensa_")
                .unwrap()
                + mensa
                + "_this_week.html"
        );

        // EN, next week
        assert_eq!(
            CONFIG.endpoint.menu.build_path(&mensa, true, true),
            String::from_str("files/content/Downloads/Gastronomie/Speiseplaene/menu_mensa_")
                .unwrap()
                + mensa
                + "_next_week.html"
        );
    }
}
