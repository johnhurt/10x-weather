use std::{collections::HashMap, sync::OnceLock};

use chrono::NaiveDate;
use log::info;

use crate::weather_data::{weather_entries, WeatherEntry, WeatherKind};

/// This will store weather entries keyed by date
static WEATHER_BY_DATE: OnceLock<HashMap<NaiveDate, &WeatherEntry>> =
    OnceLock::new();

/// This will store weather entries grouped by weather kind
static WEATHER_BY_KIND: OnceLock<HashMap<WeatherKind, Vec<&WeatherEntry>>> =
    OnceLock::new();

/// This struct represents a validated version of `WeatherQueryRequest` in the
/// main.rs file. Once the query request is validated, it can be converted into
/// an instance of this type and used for querying weather data from the
/// supported indexes
#[derive(Debug)]
pub struct WeatherQuery {
    /// Optional limit on the number of items to return
    pub limit: Option<usize>,

    /// Optional date for which we want to return the weather. There is
    /// guaranteed to be at most one entry for any date
    pub date: Option<NaiveDate>,

    /// Optional mechanism for querying for days with a specific type of weather
    pub weather: Option<WeatherKind>,
}

/// This function is called once to populate all the data in the search indexes.
/// These indexes allow the weather data to be search quickly without scanning
pub fn populate_indexes() {
    info!("Populating search indexes");

    // Store a map of each weather entry by date
    WEATHER_BY_DATE
        .set(
            weather_entries()
                .iter()
                .map(|e| (e.date, e))
                .collect::<HashMap<_, _>>(),
        )
        .expect("populate_indexes should only be called once");

    info!("Populated weather-by-day index");

    let mut weather_by_kind: HashMap<WeatherKind, Vec<&'static WeatherEntry>> =
        HashMap::new();

    weather_entries().iter().for_each(|entry| {
        weather_by_kind
            .entry(entry.weather)
            .and_modify(|entries| entries.push(entry))
            .or_insert_with(|| vec![entry]);
    });

    WEATHER_BY_KIND
        .set(weather_by_kind)
        .expect("populate_indexes should only be called once");

    info!("Populated weather-by-kind index");
}

/// This function performs queries using the date-based index for queries that
/// are guaranteed to have:
///   1. non-zero limit
///   2. a specific date
/// If a weather type is specified, the result will only be returned if the
/// weather kind at the given date matches the weather kind in the query
fn handle_date_query(
    date: &NaiveDate,
    query: &WeatherQuery,
) -> Vec<&'static WeatherEntry> {
    WEATHER_BY_DATE
        .get()
        .expect("Populate should be called before querying indexes")
        .get(date)
        .copied()
        .filter(|single_result| {
            // Filter out the weather entry if the weather kind doesn't match
            // what was queried
            query
                .weather
                .map(|weather| weather == single_result.weather)
                .unwrap_or(true)
        })
        .into_iter()
        .collect::<Vec<_>>()
}

/// This function performs queries using the kind-based index for queries that
/// are guaranteed to have:
///   1. non-zero limit
///   2. no specific date
///   3. a valid weather kind
/// If a limit is provided, this function will ensure that the resulting vector
/// has at most the given limit
fn handle_kind_query(
    kind: WeatherKind,
    query: &WeatherQuery,
) -> Vec<&'static WeatherEntry> {
    let result_iter = WEATHER_BY_KIND
        .get()
        .expect("Populate should be called before querying indexes")
        .get(&kind)
        .into_iter()
        .flat_map(|v| v.iter())
        .copied();

    // Apply the query's limit if there is one
    if let Some(limit) = query.limit {
        result_iter.take(limit).collect::<Vec<_>>()
    } else {
        result_iter.collect::<Vec<_>>()
    }
}

// If there are no date or kind restrictions on the query, we need to do a full
// scan of the weather data and apply the limit if one is present in the query
fn full_scan(query: &WeatherQuery) -> Vec<&'static WeatherEntry> {
    if let Some(limit) = query.limit {
        weather_entries().iter().take(limit).collect()
    } else {
        weather_entries().iter().collect()
    }
}

/// This is the main querying function. It will return all weather entries that
/// are valid based on the query parameters
pub fn handle_query(query: &WeatherQuery) -> Vec<&'static WeatherEntry> {
    // This is the query-planning step. We check for which query path to follow
    // from most to least strict
    match query {
        // If the limit is zero, there is nothing else to check
        WeatherQuery { limit: Some(0), .. } => vec![],

        // If a date is specified, then we will return at most one entry
        WeatherQuery {
            date: Some(date_v), ..
        } => handle_date_query(date_v, query),

        // If a weather kind is specified, then we query based on the weather
        // kind index
        WeatherQuery {
            weather: Some(weather_kind),
            ..
        } => handle_kind_query(*weather_kind, query),

        // Otherwise we do a full scan (with limits applied)
        _ => full_scan(query),
    }
}
