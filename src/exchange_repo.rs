use hyper::body::Buf;
use hyper::{Client, StatusCode, Uri};
use hyper_tls::HttpsConnector;
use tracing::{info, error, Level, debug};

use crate::{AppError, ExchangeResult};

pub async fn convert_amount(from: String, to: String, amount: &f32) -> Result<f32, AppError> {
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