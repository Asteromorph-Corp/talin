use std::error::Error;

use talin::tool;

pub struct ExchangeRateTool;

#[tool]
impl ExchangeRateTool {
    type Output = String;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        let rate = match input.currency.to_uppercase().as_str() {
            "USD" => 1.0,
            "EUR" => 1.08,
            "GBP" => 1.27,
            "JPY" => 0.0067,
            "KRW" => 0.00073,
            "CHF" => 1.12,
            other => return Err(format!("no rate for {other}").into()),
        };
        Ok(format!("1 {} = {rate} USD", input.currency.to_uppercase()))
    }

    fn from_config(_: _) -> Self {
        Self
    }
}
