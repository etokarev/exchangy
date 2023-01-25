use std::time::Duration;
use axum::Json;
use tokio::time::sleep;
use tracing::{debug};
use crate::{AppError, country_repo, exchange_repo};
use crate::models::{ConvertCurrency, ConvertResult};

pub async fn currency_handler(
    Json(payload): Json<ConvertCurrency>,
) -> Result<Json<ConvertResult>, AppError> {
    let from_currency = country_repo::get_by_name(&payload.from).await?;
    let to_currency = country_repo::get_by_name(&payload.to).await?;
    debug!("from_currency={from_currency}, to_currency={to_currency}");

    if to_currency == "RUB" {
        debug!("Russia is under sanctions, you wait 30 secs...");
        sleep(Duration::from_secs(30)).await;
        debug!("wait time is over!");
    }

    let converted_amount = exchange_repo::convert_amount(&from_currency, &to_currency, &payload.amount).await?;

    let response = ConvertResult {
        from: payload.from,
        to: payload.to,
        amount: converted_amount,
    };

    Ok(response.into())
}