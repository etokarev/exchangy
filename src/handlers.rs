use axum::Json;
use tracing::info;
use crate::{AppError, country_repo, exchange_repo};
use crate::models::{ConvertCurrency, ConvertResult};

pub async fn currency_handler(
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