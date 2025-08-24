use serde::Deserialize;

/// Meal data model. Can be used standalone to parse e.g. JSONs from
/// a caching server.
pub mod meal;

/// Configuration for scraping.
#[cfg(feature = "scrape")]
pub(crate) mod config;

/// Scraper module.
#[cfg(feature = "scrape")]
pub mod scrape;

/// Holds a German and English version of a string (e.g. for endpoint templates)
#[derive(Deserialize, Clone, Debug)]
pub struct DeEnStr<T> {
    pub de: T,
    pub en: T,
}
