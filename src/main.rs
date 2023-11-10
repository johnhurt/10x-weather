use std::str::FromStr;

use crate::indexes::{handle_query, populate_indexes, WeatherQuery};
use crate::weather_data::{parse_date, WeatherEntry, WeatherKind};
use log::info;
use poem::{
    error::ResponseError,
    get, handler,
    http::StatusCode,
    listener::TcpListener,
    web::{Html, Json, Query},
    Result, Route, Server,
};
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use strum::IntoEnumIterator;

// This includes the exposed types from the `weather_data.rs` and `indexes.rs`
mod indexes;
mod weather_data;

/// This is some static content to render if someone navigates to the root path
/// of the web server
const WELCOME_CONTENT: &str = include_str!("resources/welcome.html");

/// This struct represents the supported query parameters in the weather query
/// endpoint
#[derive(Debug, Deserialize, Serialize)]
struct WeatherQueryRequest {
    limit: Option<String>,
    weather: Option<String>,
    date: Option<String>,
}

/// This is a custom error type that we can return from the query handler to
/// indicate a 400 error and given some helpful feedback to the user
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
struct QueryError {
    message: String,
}

// This impl block just tells poem how to interpret our error type
impl ResponseError for QueryError {
    fn status(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }
}

// This impl block implements the parsing of the stringy query parameters into
// useable values for querying weather data. It attempts to convert the raw
// weather query to the typed query, and returns a helpful error message if
// there are any invalid values
impl TryFrom<WeatherQueryRequest> for WeatherQuery {
    type Error = QueryError;

    fn try_from(
        raw_query: WeatherQueryRequest,
    ) -> std::result::Result<Self, Self::Error> {
        // Try to parse the limit as an unsigned integer
        let limit = if let Some(limit_str) = raw_query.limit {
            Some(limit_str.parse::<usize>().map_err(|_| QueryError {
                message: format!(
                    "Invalid limit value in query parameters: {limit_str} \
                    - Expected a non-negative integer"
                ),
            })?)
        } else {
            None
        };

        let date = if let Some(date_str) = raw_query.date {
            Some(
                parse_date(&date_str)
                    .map_err(|_| QueryError {
                        message: format!(
                    "Invalid date value in query parameters: {date_str} \
                    - Expected a date of the form YYYY-MM-DD"
                ),
                    })?
                    .1,
            )
        } else {
            None
        };

        let weather = if let Some(weather_str) = raw_query.weather {
            Some(WeatherKind::from_str(&weather_str).map_err(|_| {
                QueryError {
                    message: format!(
                        "Invalid weather kind: {weather_str} \
                        - Expected one of the following: {}",
                        WeatherKind::iter()
                            .map(|c| c.to_string().to_lowercase())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                }
            })?)
        } else {
            None
        };

        Ok(WeatherQuery {
            limit,
            date,
            weather,
        })
    }
}

/// This function handles the calls to the weather query service. Query
/// parameters are synthesized into the input for the function and the resulting
/// list of weather entries is serialized to json for the output
#[handler]
fn weather_query(
    query: Query<WeatherQueryRequest>,
) -> Result<Json<Vec<&'static WeatherEntry>>> {
    info!("Handling raw weather query:\n{:#?}", query.0);

    let parsed_query = query.0.try_into().map_err(|e| {
        info!("Query parameters are invalid. Returning 400 error {e}");
        e
    })?;

    info!("Successfully parsed weather query as:\n{:#?}", parsed_query);

    let result = handle_query(&parsed_query);

    info!("Query returned {} weather entries", result.len());

    Ok(Json(result))
}

/// Handler for the root path that returns some static html about this project
#[handler]
fn welcome() -> Html<String> {
    Html(WELCOME_CONTENT.into())
}

/// This is the entry point for the application. This is where we
/// 1. Initialize logging
/// 2. Preload the weather data and populate indexes
/// 3. Configure the web server
/// 4. Launch the web server and wait for `SIG_INT` (ctrl-c)
///
/// All other events are handling asynchronously
#[tokio::main]
async fn main() {
    SimpleLogger::new()
        .init()
        .expect("The logger should always initialize successfully");

    populate_indexes();

    info!("Starting weather-query server on port 3000");

    let app = Route::new()
        .at("/", get(welcome))
        .at("/query", get(weather_query));

    let _ = Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run_with_graceful_shutdown(
            app,
            async move {
                let _ = tokio::signal::ctrl_c().await;
                info!("Received shutdown signal. Stopping server");
            },
            None,
        )
        .await;
}
