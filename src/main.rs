mod country_repo;
mod exchange_repo;
mod models;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{info, error, Level, debug};
use tracing_subscriber::FmtSubscriber;
use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use crate::models::{ConvertCurrency, ConvertResult};

#[tokio::main]
async fn main() {
    // initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // build our application with a route
    let app = Router::new()
        // `POST /users` goes to `create_user`
        .route("/currency", post(currency_handler));

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
    Json(payload): Json<ConvertCurrency>,
 ) -> Result<Json<ConvertResult>, AppError> {
    let from = payload.from;
    let to = payload.to;

    let from_currency = country_repo::get_by_name(&from).await?;
    let to_currency = country_repo::get_by_name(&to).await?;
    info!("from_currency={from_currency}, to_currency={to_currency}");

    let converted_amount = exchange_repo::convert_amount(from_currency, to_currency, &payload.amount).await?;

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
