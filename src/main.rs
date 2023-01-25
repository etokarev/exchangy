mod country_repo;

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
// A simple type alias so as to DRY.
// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

use country_repo::{DynCountryRepo, ExampleCountryRepo};

#[tokio::main]
async fn main() {
    // initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let country_repo = Arc::new(ExampleCountryRepo) as DynCountryRepo;

    // build our application with a route
    let app = Router::new()
        // `POST /users` goes to `create_user`
        .route("/currency", post(currency_handler))
        .with_state(country_repo);

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
    Json(payload): Json<ConvertCurrency>,
 ) -> Result<Json<ConvertResult>, AppError> {
    let from = payload.from;
    let to = payload.to;

    let from_currency = country_repo.get_by_name(&from).await?;
    let to_currency = country_repo.get_by_name(&to).await?;
    info!("from_currency={from_currency}, to_currency={to_currency}");

    let converted_amount = convert(from_currency, to_currency, &payload.amount).await?;

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

async fn convert(from: String, to: String, amount: &f32) -> Result<f32, AppError> {
    let url: Uri = format!("https://v6.exchangerate-api.com/v6/615900b74ad7d5dec68b5f0f/pair/{from}/{to}/{amount}").parse().unwrap();
    debug!("convert url={}", url.to_string());


    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    // Fetch the url...
    let res = client.get(url).
        await.map_err(|_| AppError::new("http response error"))?;

    if res.status() != StatusCode::OK {
        return Err(AppError::new(&format!("Exchangerate-api returned {} http code", res.status())))
    }

    let body = hyper::body::aggregate(res).
        await.map_err(|_| AppError::new("reading buffer error"))?;

    // try to parse as json with serde_json
    let result: ExchangeResult = serde_json::from_reader(body.reader()).
        map_err(|_| AppError::new("json deserialization error"))?;
    debug!("fetched result {}", result.conversion_result);


    Ok(result.conversion_result)
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
