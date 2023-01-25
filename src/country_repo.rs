use std::sync::Arc;
use dashmap::DashMap;
use hyper::body::Buf;
use hyper::{Client, StatusCode, Uri};
use hyper_tls::HttpsConnector;
use tracing::{info, debug};

use crate::AppError;
use crate::models::Country;

pub async fn get_by_name(name: &String, country_map: Arc<DashMap<String, String>>) -> Result<String, AppError> {
    let name = name.trim();
    let url: Uri = format!("https://restcountries.com/v3.1/name/{name}?fields=name,currencies").parse().
        map_err(|_| AppError::new("URL parsing error"))?;

    if let Some(v) = country_map.get(name) {
        debug!("cache hit for {name}, value: {}", v.to_string());
        return Ok(v.to_string());
    }

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    // Fetch the url...
    let res = client.get(url).
        await.map_err(|_| AppError::new("http response error"))?;

    if res.status() != StatusCode::OK {
        return Err(AppError::new(&format!("bad country name: {name}").to_string()))
    }

    // asynchronously aggregate the chunks of the body
    let body = hyper::body::aggregate(res).
        await.map_err(|_| AppError::new("reading buffer error"))?;

    // try to parse as json with serde_json
    let counties: Vec<Country> = serde_json::from_reader(body.reader()).
        map_err(|_| AppError::new("json deserialization error"))?;
    debug!("fetched result with {} items", counties.len());

    match counties.len() {
        1 => {
            match counties[0].currencies.keys().next() {
                Some(key) => {
                    debug!("cache miss for {name}, value: {}", key.to_string());
                    country_map.insert(name.to_string(), key.to_string());
                    return Ok(key.to_string())
                },
                None => Err(AppError::new(&format!("no currency for country {name}").to_string()))
            }
        }
        0 => Err(AppError::new(&format!("could not fetch country {name}").to_string())),
        _ => Err(AppError::new("unexpected result when fetching country {name}"))
    }
}