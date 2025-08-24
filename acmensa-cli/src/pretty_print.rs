use std::collections::HashMap;

use lazy_static::lazy_static;
use libacmensa::{
    meal::{MealInfo, MealType, SideInfo},
    scrape::DayView,
};
use serde::{Deserialize, Serialize};

use crate::args::MenuOpts;

static CONFIG_TOML: &str = include_str!("../../res/pretty-print.toml");

const RST: &str = "\x1b[0m";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormatSet {
    pub meal_map: Vec<FormatMealMap>,
    pub meal_def: FormatMeal,
    pub meal_subtext_colour: String,
    pub side_style: StyleSide,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormatMealMap {
    #[serde(rename = "type")]
    pub typ: MealType,
    pub format: FormatMeal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormatMeal {
    pub main: StyleMeal,
    pub alt_veg: Option<StyleMeal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StyleMeal {
    pub emoji: String,
    pub colour: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StyleSide {
    pub head_colour: String,
    pub subtext_colour: String,
}

lazy_static! {
    pub static ref CONFIG: FormatSet = toml::from_str(CONFIG_TOML).unwrap();
    pub static ref FORMAT_MEAL_DEFAULT: &'static FormatMeal = &CONFIG.meal_def;
    pub static ref FORMAT_SIDE: &'static StyleSide = &CONFIG.side_style;
    pub static ref SUBTEXT_COLOUR: &'static str = &CONFIG.meal_subtext_colour;
    pub static ref FORMAT: HashMap<MealType, FormatMeal> = {
        CONFIG
            .meal_map
            .iter()
            .map(|FormatMealMap { typ, format }| (typ.clone(), format.clone()))
            .collect::<HashMap<MealType, FormatMeal>>()
    };
}

/// Prints a single main meal info
fn print_main(main: &MealInfo, opts: &MenuOpts) {
    let fmt = FORMAT.get(&main.typ).unwrap_or(&FORMAT_MEAL_DEFAULT);
    let fmt_style = if main.vegan && fmt.alt_veg.is_some() {
        unsafe { &fmt.alt_veg.clone().unwrap_unchecked() }
    } else {
        &fmt.main
    };
    let (emoji, colour) = (&fmt_style.emoji, &fmt_style.colour);

    // Print headline
    println!(
        "\x1b[38;5;{colour}m {emoji} {}{}{RST}",
        main.text,
        if main.vegan { " 🌱" } else { "" }
    );

    // Short output -> skip subtext etc.
    if opts.short {
        return;
    }

    // Print subtext
    if !main.subtext.is_empty() {
        println!("\t\x1b[3;38;5;{}m{}{RST}", *SUBTEXT_COLOUR, main.subtext);
    }

    if opts.allergens && !main.allergens.is_empty() {
        println!(
            "\t\x1b[3;38;5;{}m{}: {}{RST}",
            *SUBTEXT_COLOUR,
            if opts.english {
                "Allergens"
            } else {
                "Allergene"
            },
            main.allergens
        );
    }

    // If needed, print price as well
    if opts.prices {
        println!("\t\x1b[3;38;5;{}m{}{RST}", *SUBTEXT_COLOUR, main.price);
    }
}

/// Prints single side meal info
fn print_side(side: &SideInfo, opts: &MenuOpts) {
    let StyleSide {
        head_colour: colour,
        subtext_colour: colour_subtext,
    } = &*FORMAT_SIDE;

    // Category name
    println!("\x1b[38;5;{colour}m {}{RST}", side.typ.name(opts.english));

    // Print all alternatives in a list
    for alternative in &side.alternatives {
        println!("\x1b[38;5;{colour_subtext}m\t– {}{RST}", alternative.text);
        if opts.allergens && !alternative.allergens.is_empty() {
            println!(
                "\t\x1b[3;38;5;{}m  {}: {}{RST}",
                *SUBTEXT_COLOUR,
                if opts.english {
                    "Allergens"
                } else {
                    "Allergene"
                },
                alternative.allergens
            );
        }
    }
}

/// Prints all main/side meals in a day
pub async fn pretty_print_all(day: DayView<'_>, opts: &MenuOpts) {
    for main in day.main_dishes {
        // Skip vegal meals if requested
        if opts.skip_vegan && main.vegan {
            continue;
        }

        // If a category filter is available, apply it
        if let Some(ref only) = opts.only
            && &main.typ != only
        {
            continue;
        }

        print_main(main, opts);
    }

    // Category filter OR skip_sides => skip sides
    if opts.only.is_some() || opts.skip_sides {
        return;
    }

    println!();
    for side in day.side_dishes {
        print_side(side, opts);
    }
}
