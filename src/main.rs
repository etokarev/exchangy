use axum::{
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


// A simple type alias so as to DRY.
// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

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
        .route("/currency", post(convert_currency));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn convert_currency(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<ConvertCurrency>,
 ) -> Result<Json<ConvertResult>, AppError> {
    let from = payload.from;
    let to = payload.to;

//   try_thing()?;
     let from_currency = fetch_country(&from).await?;
     info!(from_currency="{from_currency}");

    // insert your application logic here
    let response = ConvertResult {
        from: from,
        to: to,
        amount: payload.amount,
    };

    Ok(response.into())
}
//-> impl IntoResponse {
//
//     let from = payload.from;
//     let to = payload.to;
//
//     let from_currency = fetch_country(from).await?;
//     info!(from_currency="{from_currency}");
//
//     // insert your application logic here
//     let response = ConvertResult {
//         from: from,
//         to: to,
//         amount: payload.amount,
//     };
//
//     // this will be converted into a JSON response
//     // with a status code of `201 Created`
//     (StatusCode::OK, Json(response))
// }

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
    where
        E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
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

async fn fetch_country(name: &String) -> Result<String, anyhow::Error> {
    let url: Uri = format!("https://restcountries.com/v3.1/name/{name}?fields=name,currencies").parse().unwrap();
    info!("fetch_country called with name={name}");

    debug!("fetch_country url={}", url.to_string());
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    // Fetch the url...
    let res = client.get(url).await?;

    // asynchronously aggregate the chunks of the body
    let body = hyper::body::aggregate(res).await?;

    // try to parse as json with serde_json
    let counties: Vec<Country> = serde_json::from_reader(body.reader())?;
    info!("fetched result with {} items", counties.len());

    Ok("all good".to_string())
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