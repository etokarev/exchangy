use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct ConvertCurrency {
    pub from: String,
    pub to: String,
    pub amount: f32
}

// the output to our `create_user` handler
#[derive(Serialize)]
pub struct ConvertResult {
    pub from: String,
    pub to: String,
    pub amount: f32
}

#[derive(Deserialize, Debug)]
pub struct Country {
    pub name: Name,
    pub currencies: HashMap<String, Currency>
}

#[derive(Deserialize, Debug)]
pub struct Name {
    pub common: String,
}

#[derive(Deserialize, Debug)]
pub struct Currency {
    pub name: String,
    pub symbol: String
}


#[derive(Deserialize, Debug)]
pub struct ExchangeResult {
    pub base_code: String,
    pub target_code: String,
    pub conversion_rate: f32,
    pub conversion_result: f32
}
