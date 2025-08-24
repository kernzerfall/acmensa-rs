use std::{
    fmt::{self, Display},
    path::PathBuf,
};

use clap::*;
use libacmensa::meal::MealType;

/// Fetch, parse, and display/export menu data for Mensen of the Studierendenwerk Aachen.
#[derive(Parser, Debug, Default)]
#[clap(author, version)]
pub struct Args {
    /// Target mensa.
    #[arg(short, long, default_value = "ahornstrasse")]
    pub mensa: Mensa,

    /// Switch to English. Default is German.
    #[arg(short, long)]
    pub english: bool,

    #[command(subcommand)]
    pub verb: Verb,
}

#[derive(Subcommand, Clone, Debug, PartialEq, Eq)]
pub enum Verb {
    /// Fetches and displays the daily menu for a given mensa.
    Menu(MenuOpts),
    /// Fetches and exports all available days for a given mensa in JSON.
    #[cfg(feature = "json")]
    Export(ExportOpts),
    /// Dumps the JSON schema for the exports.
    #[cfg(feature = "json-schema")]
    Schema,
    /// Fetches and displays the opening times for a given mensa. (Not implemented yet)
    Times,
}

impl Default for Verb {
    fn default() -> Self {
        Self::Menu(MenuOpts::default())
    }
}

#[derive(Parser, Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuOpts {
    /// Print JSON of day plan.
    #[cfg(feature = "json")]
    #[arg(short, long)]
    pub json: bool,

    /// ISO Date (YYYY-MM-DD). Takes precedence over --day/-d.
    #[arg(long)]
    pub date: Option<chrono::NaiveDate>,

    /// Day description.
    #[arg(short, long, default_value = "today")]
    pub day: MenuDate,

    /// Only print meals of the given category.
    #[arg(short, long)]
    pub only: Option<MealType>,

    /// Switch to English. Default is German.
    #[arg(short, long)]
    pub english: bool,

    /// Only print headline for main meals. Takes precedence over other specific options.
    #[arg(short, long)]
    pub short: bool,

    /// Print meal prices.
    #[arg(short, long)]
    pub prices: bool,

    /// Do not print sides.
    #[arg(short = 'm', long)]
    pub skip_sides: bool,

    /// Do not print vegan meals.
    #[arg(short = 'n', long)]
    pub skip_vegan: bool,

    /// Print allergens. (No guarantee that they are parsed correctly!)
    #[arg(short, long)]
    pub allergens: bool,
}

#[derive(Parser, Clone, Debug, Default, PartialEq, Eq)]
pub struct ExportOpts {
    /// Put files in this directory.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq)]
pub enum Mensa {
    Academica,
    #[default]
    Ahornstrasse,
    BistroTemplergraben,
    Bayernallee,
    EupenerStrasse,
    KMAC,
    Suedpark,
    Vita,
    Juelich,
}

impl Display for Mensa {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Mensa::Academica => "academica",
                Mensa::Ahornstrasse => "ahornstrasse",
                Mensa::BistroTemplergraben => "bistro_templergraben",
                Mensa::Bayernallee => "bayernallee",
                Mensa::EupenerStrasse => "eupener_strasse",
                Mensa::KMAC => "kmac",
                Mensa::Suedpark => "suedpark",
                Mensa::Vita => "vita",
                Mensa::Juelich => "juelich",
            }
        )
    }
}

impl Mensa {
    pub fn url_name(&self) -> String {
        self.to_string()
    }
}

#[derive(ValueEnum, Default, Clone, Debug, PartialEq, Eq)]
pub enum MenuDate {
    /// The day of today
    #[default]
    Today,

    /// Next day (tomorrow, or Monday if it's a weekend)
    Next,
}
