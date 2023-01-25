mod country_repo;
mod exchange_repo;
mod models;
mod handlers;

use axum::{http::StatusCode, response::{IntoResponse, Response}, Router, routing::post};

use std::net::SocketAddr;
use tracing::{info};
use tracing_subscriber::FmtSubscriber;
use std::fmt;
use std::error::Error;
use std::sync::Arc;
use dashmap::DashMap;
use crate::handlers::currency_handler;

#[tokio::main]
async fn main() {
    // initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_env_filter("exchangy=debug")
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let map: Arc<DashMap<String, String>> = Arc::new(DashMap::new());
    let shared_state = Arc::new(AppState { country_map: map.clone() });
    // build our application with routes
    let app = Router::new()
        .route("/currency", post(currency_handler))
        .with_state(shared_state);

    // run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Debug)]
pub struct AppError{
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

#[derive(Clone)]
pub struct AppState {
    pub country_map: Arc<DashMap<String, String>>,
}