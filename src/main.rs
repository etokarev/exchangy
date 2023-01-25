mod country_repo;
mod exchange_repo;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use hyper::body::Buf;
use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{info, error, Level, debug};
use tracing_subscriber::FmtSubscriber;
use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use std::sync::Arc;

use country_repo::{DynCountryRepo, ExampleCountryRepo};
use crate::exchange_repo::{DynExchangeRepo, ExampleExchangeRepo};

#[tokio::main]
async fn main() {
    // initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let country_repo = Arc::new(ExampleCountryRepo) as DynCountryRepo;
    let exchange_repo = Arc::new(ExampleExchangeRepo) as DynExchangeRepo;

    // build our application with a route
    let app = Router::new()
        // `POST /users` goes to `create_user`
        .route("/currency", post(currency_handler))
        .with_state(country_repo)
        .with_state(exchange_repo);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn currency_handler(
    State(country_repo): State<DynCountryRepo>,
    State(exchange_repo): State<DynExchangeRepo>,
    Json(payload): Json<ConvertCurrency>,
 ) -> Result<Json<ConvertResult>, AppError> {
    let from = payload.from;
    let to = payload.to;

    let from_currency = country_repo.get_by_name(&from).await?;
    let to_currency = country_repo.get_by_name(&to).await?;
    info!("from_currency={from_currency}, to_currency={to_currency}");

    let converted_amount = exchange_repo.convert(from_currency, to_currency, &payload.amount).await?;

    let response = ConvertResult {
        from: from,
        to: to,
        amount: converted_amount,
    };

    Ok(response.into())
}

#[derive(Debug)]
struct AppError{
    details: String
}

impl AppError {
    fn new(msg: &str) -> AppError {
        AppError{details: msg.to_string()}
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for AppError {
    fn description(&self) -> &str {
        &self.details
    }
}


// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.details),
        ).into_response()
    }
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct ConvertCurrency {
    to: String,
    from: String,
    amount: f32
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct ConvertResult {
    from: String,
    to: String,
    amount: f32
}

#[derive(Deserialize, Debug)]
struct Country {
    name: Name,
    currencies: HashMap<String, Currency>
}

#[derive(Deserialize, Debug)]
struct Name {
    common: String,
}

#[derive(Deserialize, Debug)]
struct Currency {
    name: String,
    symbol: String
}


#[derive(Deserialize, Debug)]
struct ExchangeResult {
    base_code: String,
    target_code: String,
    conversion_rate: f32,
    conversion_result: f32
}
