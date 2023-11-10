use std::str::FromStr;
use std::sync::OnceLock;

use chrono::NaiveDate;
use log::{error, info, warn};
use nom::character::complete::{char, i32, u32};
use nom::combinator::{map, map_opt, map_res, rest};
use nom::number::complete::float;
use nom::sequence::delimited;
use nom::{
    sequence::{terminated, tuple},
    IResult,
};
use serde::Serialize;
use strum_macros::{Display, EnumIter, EnumString};

/// This is the weather data loaded into the binary at compile time
const RAW_WEATHER_DATA: &str = include_str!("resources/seattle-weather.csv");

/// This will be where the parsed weather data will be stored after lazy loading
static WEATHER_DATA: OnceLock<Vec<WeatherEntry>> = OnceLock::new();

/// Enumeration of the different types of weather (non exhaustive means we
/// aren't guarantying there won't be new types of weather in the future 😅)
#[non_exhaustive]
#[derive(
    Debug,
    Serialize,
    EnumString,
    EnumIter,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    Display,
)]
#[serde(rename_all = "lowercase")]
#[strum(ascii_case_insensitive)]
pub enum WeatherKind {
    Drizzle,
    Rain,
    Snow,
    Sun,
    Fog,
}

/// This struct represents one entry in the table of all weather data
#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub struct WeatherEntry {
    /// Date of the entry
    pub date: NaiveDate,

    /// Amount of precipitation in cm(?)
    pub precipitation: f32,

    /// Daily low temperature in ºC
    pub temp_min: f32,

    /// Daily high temperature in ºC
    pub temp_max: f32,

    /// Average speed over the day m/s(?)
    pub wind: f32,

    /// Human weather description
    pub weather: WeatherKind,
}

/// Information about an error that ocurred when parsing a weather entry
#[derive(Debug, PartialEq)]
struct WeatherEntryParseError {
    line_num: usize,
    line: String,
    parse_error: String,
}

/// Parse a simple year-month-day date
pub fn parse_date(date_str: &str) -> IResult<&str, NaiveDate> {
    map_opt(
        tuple((i32, delimited(char('-'), u32, char('-')), u32)),
        |(y, m, d)| NaiveDate::from_ymd_opt(y, m, d),
    )(date_str)
}

/// Attempt to parse the given string representing a line from the input weather
/// file into a `WeatherEntry`.
fn parse_weather_row(weather_entry_str: &str) -> IResult<&str, WeatherEntry> {
    map(
        tuple((
            terminated(parse_date, char(',')),
            terminated(float, char(',')),
            terminated(float, char(',')),
            terminated(float, char(',')),
            terminated(float, char(',')),
            map_res(rest, |wk: &str| WeatherKind::from_str(wk)),
        )),
        |(date, precipitation, temp_max, temp_min, wind, weather)| {
            WeatherEntry {
                date,
                precipitation,
                temp_max,
                temp_min,
                wind,
                weather,
            }
        },
    )(weather_entry_str)
}

/// Attempt to parse all the weather entries in the input data file. This
/// function expects there to be a header line at the beginning of the file
/// which will be skipped. Each line in the file will either return a
/// successfully-parsed weather entry or an error containing helpful info for
/// fixing the problem in the input file.
///
/// Note. This function returns an iterator, so no processing will happen until
///     the elements of the iterator are collected
fn parse_weather_file_contents<'a>(
    raw_weather_data: &'a str,
) -> impl Iterator<Item = Result<WeatherEntry, WeatherEntryParseError>> + 'a {
    raw_weather_data
        .lines()
        .skip(1)
        .filter(|line| !line.is_empty())
        .map(|line| (line, parse_weather_row(line)))
        .enumerate()
        .map(|(line_index, (line, parse_result))| match parse_result {
            Ok((_, entry)) => Ok(entry),
            Err(e) => Err(WeatherEntryParseError {
                line_num: line_index + 2,
                line: line.to_owned(),
                parse_error: format!("{e}"),
            }),
        })
}

/// Lazily load, parse, and cache all the weather entries from the file. If
/// there is an error parsing the data, this function will panic and print out
/// a message indicating the lines of the input that could not be parsed
pub fn weather_entries() -> &'static [WeatherEntry] {
    WEATHER_DATA.get_or_init(|| {
        let mut error_count = 0;

        info!("Parsing weather data");

        let mut result = parse_weather_file_contents(RAW_WEATHER_DATA)
            .filter_map(|parse_result| match parse_result {
                Ok(entry) => Some(entry),
                Err(e) => {
                    error_count += 1;
                    error!(
                        "Failed to parse line {}:{}\nParse Error: {}",
                        e.line_num, e.line, e.parse_error
                    );
                    None
                }
            })
            .collect::<Vec<_>>();

        if error_count > 0 {
            panic!("Exiting because of {error_count} parse error(s)");
        }

        if !result.is_empty() {
            info!("Successfully parsed {} weather entries", result.len());

            result.sort_by_key(|e| e.date);

            info!(
                "Sorted weather entries by date. The range covered is {} to {}",
                result[0].date,
                result.last().expect("Known to not be empty").date
            );
        } else {
            warn!(
                "The weather data from the file contained no entries. This \
                isn't technically an error, but it will make the service very \
                boring"
            );
        }

        result
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_date_parse() {
        assert_eq!(
            NaiveDate::from_ymd_opt(2034, 12, 23).unwrap(),
            parse_date("2034-12-23").unwrap().1
        )
    }

    #[test]
    fn test_parse_entry() {
        assert_eq!(
            WeatherEntry {
                date: NaiveDate::from_ymd_opt(2012, 6, 3).unwrap(),
                precipitation: 0.,
                temp_min: 9.4,
                temp_max: 17.2,
                wind: 2.9,
                weather: WeatherKind::Sun
            },
            parse_weather_row("2012-06-03,0.0,17.2,9.4,2.9,sun")
                .unwrap()
                .1
        )
    }

    #[test]
    fn test_parse_all_entries_happy() {
        assert_eq!(
            parse_weather_file_contents(
                "date,precipitation,temp_max,temp_min,wind,weather\n\
                2012-06-03,0.0,17.2,9.4,2.9,sun\n\
                2012-06-04,1.3,12.8,8.9,3.1,rain"
            )
            .collect::<Vec<_>>(),
            vec![
                Ok(WeatherEntry {
                    date: NaiveDate::from_ymd_opt(2012, 6, 3).unwrap(),
                    precipitation: 0.,
                    temp_min: 9.4,
                    temp_max: 17.2,
                    wind: 2.9,
                    weather: WeatherKind::Sun
                }),
                Ok(WeatherEntry {
                    date: NaiveDate::from_ymd_opt(2012, 6, 4).unwrap(),
                    precipitation: 1.3,
                    temp_min: 8.9,
                    temp_max: 12.8,
                    wind: 3.1,
                    weather: WeatherKind::Rain
                }),
            ]
        )
    }

    #[test]
    fn test_parse_all_entries_with_error() {
        let results = parse_weather_file_contents(
            "date,precipitation,temp_max,temp_min,wind,weather\n\
            2012-06-03,Oops,17.2,9.4,2.9,sun\n\
            2012-06-04,1.3,12.8,8.9,3.1,rain",
        )
        .collect::<Vec<_>>();

        assert!(matches!(
            results[0],
            Err(WeatherEntryParseError { line_num: 2, .. })
        ));

        assert_eq!(
            results[1],
            Ok(WeatherEntry {
                date: NaiveDate::from_ymd_opt(2012, 6, 4).unwrap(),
                precipitation: 1.3,
                temp_min: 8.9,
                temp_max: 12.8,
                wind: 3.1,
                weather: WeatherKind::Rain
            })
        );
    }
}
