use chrono::{DateTime, Datelike, Duration, FixedOffset, Weekday};
use clap::Parser;

#[cfg(feature = "json")]
use std::path::PathBuf;

#[cfg(feature = "json-schema")]
use libacmensa::scrape::WeekData;

use crate::args::{MenuDate, Verb};

mod pretty_print;
#[cfg(debug_assertions)]
const DEF_LOG_LEVEL: &str = "info";

#[cfg(not(debug_assertions))]
const DEF_LOG_LEVEL: &str = "warn";

const ENV_LOG_LEVEL: &str = "RUST_LOG";

const STDOUT_DATE_FMT: &str = "%d.%m.%Y";

#[cfg(feature = "json")]
const JSON_NAME_DATE_FMT: &str = "%Y%m%d.json";

use pretty_print::*;

pub(crate) mod args;

#[derive(Debug, Clone)]
struct DateCtx {
    pub utc2: FixedOffset,
    pub now: DateTime<FixedOffset>,
    pub day_n: u32,

    pub first_avail_date: DateTime<FixedOffset>,
    pub last_avail_date: DateTime<FixedOffset>,
}

async fn handle_menu(
    args: &args::Args,
    opts: &args::MenuOpts,
    datectx: &DateCtx,
) -> anyhow::Result<()> {
    let DateCtx {
        utc2: utc1,
        now,
        day_n,
        first_avail_date,
        last_avail_date,
    } = *datectx;

    let (next_week, idx) = if let Some(date) = opts.date {
        if [Weekday::Sat, Weekday::Sun].contains(&date.weekday()) {
            log::error!("requested date falls in a weekend");
            panic!();
        }

        let date = date
            .and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .and_local_timezone(utc1)
            .unwrap();

        if date > last_avail_date || date < first_avail_date {
            log::error!("requested date falls outside available data range");
            panic!();
        }

        let diff = (date - first_avail_date).num_days();
        let next_week = diff > 6;
        let idx = diff % 7;

        (next_week, idx)
    } else if opts.day == MenuDate::Today {
        if [Weekday::Sat, Weekday::Sun].contains(&now.weekday()) {
            log::error!("requested date falls in a weekend");
            panic!();
        }

        let diff = (now - first_avail_date).num_days();
        let next_week = diff > 6;
        let idx = diff % 7;

        (next_week, idx)
    } else {
        let next_week = day_n > 5;
        let idx = if next_week { 0 } else { day_n };

        let next_day = first_avail_date
            + if next_week {
                Duration::days(7)
            } else {
                Duration::days(day_n.into())
            };
        log::info!("Next day is {}", next_day.format(STDOUT_DATE_FMT));

        (next_week, idx as i64)
    };

    let get = libacmensa::scrape::get(&args.mensa.url_name(), next_week, opts.english).await?;
    let html = get.text().await?;
    let result = libacmensa::scrape::scrape_page(&html).await?.sorted();
    let result_day = result.get_day(idx as usize);

    #[cfg(feature = "json")]
    if opts.json {
        println!("{}", serde_json::to_string_pretty(&result_day)?);
    } else {
        pretty_print_all(result_day, opts).await;
    }

    #[cfg(not(feature = "json"))]
    pretty_print_all(result_day, opts).await;

    Ok(())
}

#[cfg(feature = "json")]
async fn handle_export(
    args: &args::Args,
    opts: &args::ExportOpts,
    datectx: &DateCtx,
) -> anyhow::Result<()> {
    let first_avail_date = datectx.first_avail_date;
    let outdir = opts.output.clone().unwrap_or(PathBuf::from("./"));

    let get_this = libacmensa::scrape::get(&args.mensa.url_name(), false, args.english).await?;
    let html_this = get_this.text().await?;
    let result_this = libacmensa::scrape::scrape_page(&html_this).await?.sorted();

    let get_next = libacmensa::scrape::get(&args.mensa.url_name(), true, args.english).await?;
    let html_next = get_next.text().await?;
    let result_next = libacmensa::scrape::scrape_page(&html_next).await?.sorted();

    for i in 0..5 {
        let date1 = first_avail_date + Duration::days(i);
        let date2 = first_avail_date + Duration::days(i + 7);
        std::fs::write(
            outdir.join(format!("{}", date1.format(JSON_NAME_DATE_FMT))),
            serde_json::to_string_pretty(&result_this.get_day(i as usize))
                .unwrap_or("{}".to_string()),
        )?;

        std::fs::write(
            outdir.join(format!("{}", date2.format(JSON_NAME_DATE_FMT))),
            serde_json::to_string_pretty(&result_next.get_day(i as usize))
                .unwrap_or("{}".to_string()),
        )?;
    }

    Ok(())
}

#[cfg(feature = "json-schema")]
async fn handle_schema() -> anyhow::Result<()> {
    println!("{}", WeekData::day_schema()?);
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::var(ENV_LOG_LEVEL).is_err() {
        unsafe { std::env::set_var(ENV_LOG_LEVEL, DEF_LOG_LEVEL) };
    }
    pretty_env_logger::init();

    let args = args::Args::parse();

    let utc2 = chrono::FixedOffset::east_opt(7200).unwrap();
    let now = chrono::Utc::now()
        .with_timezone(&utc2)
        .with_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap();

    let day_n = now.weekday().number_from_monday();

    let first_avail_date = now - chrono::Duration::days((day_n - 1) as i64);
    let last_avail_date =
        now + chrono::Duration::days((13 - day_n) as i64) - chrono::Duration::milliseconds(1);

    log::info!("Today is: {}", now.format(STDOUT_DATE_FMT));
    log::info!(
        "Available date range: {} -- {}",
        first_avail_date.format(STDOUT_DATE_FMT),
        last_avail_date.format(STDOUT_DATE_FMT)
    );

    let datectx = &DateCtx {
        utc2,
        now,
        day_n,
        first_avail_date,
        last_avail_date,
    };

    match args.verb {
        Verb::Menu(ref menu_opts) => handle_menu(&args, menu_opts, datectx).await?,
        #[cfg(feature = "json")]
        Verb::Export(ref menu_opts) => handle_export(&args, menu_opts, datectx).await?,
        Verb::Times => todo!(),
        #[cfg(feature = "json-schema")]
        Verb::Schema => handle_schema().await?,
    };

    return Ok(());
}
