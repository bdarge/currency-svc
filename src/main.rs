use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::{self, VarError};
use std::fmt::Debug;
use std::str::FromStr;
use tonic::transport::Server;
use tonic::{Request, Response, Status};
use tonic::async_trait;

use currency::currency_server::{Currency, CurrencyServer};
use currency::{CurrencyRequest, CurrencyResponse};

pub mod currency {
    tonic::include_proto!("currency");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rate {
    pub conversion_rates: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct CurrencyService {
    url: String,
    token: String,
    base: String,
}

impl Default for CurrencyService {
    fn default() -> CurrencyService {
        CurrencyService {
            url: "".to_owned(),
            token: "".to_owned(),
            base: "".to_owned(),
        }
    }
}


#[async_trait]
impl Currency for CurrencyService {
    async fn convert(
        &self,
        request: Request<CurrencyRequest>,
    ) -> Result<Response<CurrencyResponse>, Status> {
        println!("Convert = {:?}", request);
        let s = request.into_inner().to_owned();
        let mut b = s.base;

        if b == "" {
            b = self.base.clone();
        }

        let r = get_current_data(&self.url, &self.token, &b).await;

        match r {
            Ok(rate) => {
                let reply = currency::CurrencyResponse {
                    to: s.symbol.clone(),
                    base: b,
                    value: rate.conversion_rates[&s.symbol],
                };
                return Ok(Response::new(reply));
            }
            Err(e) => {
                println!("{:}", e);
                return Err(Status::internal("Internal error"));
            }
        }
    }
}

async fn get_current_data(url: &str, token: &str, base: &str) -> Result<Rate, Box<dyn std::error::Error>> {
    println!("Get today's exchange rate for {}", base);
    let url = format!("{}/{}/latest/{}", url, token, base);
    let text_response = reqwest::get(url).await?.text().await?;
    let rate: Rate = serde_json::from_str(&text_response)?;
    Ok(rate)
}

fn get_env_var<T>(env_var_name: &str) -> T
where
    T: FromStr,
    T::Err: Debug,
    T: Default,
{
    let var = match env::var(env_var_name) {
        Err(VarError::NotPresent) => return T::default(),
        res => res.unwrap(),
    };

    var.parse().unwrap()
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut environment = get_env_var::<String>("ENV");

    if environment == "" {
        environment = String::from("dev");
    }

    // Load environment variables from .env file.
    // Fails if .env file not found, not readable or invalid.
    dotenvy::from_path(format!("./config/{environment}.env"))?;

    let mut port = get_env_var::<u32>("PORT");

    if port == 0 {
        port = 8001
    }

    let addr = format!("0.0.0.0:{:?}", port).parse().unwrap();

    let name = get_env_var::<String>("TOKEN_NAME");
    let token = env::var(name).unwrap();
    let service: CurrencyService = CurrencyService {
        url: get_env_var::<String>("RATE_URL"),
        token: token,
        ..Default::default()
    };

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<CurrencyServer<CurrencyService>>()
        .await;

    println!("Currency, and health server listening on: {}", addr);

    Server::builder()
        .add_service(health_service)
        .add_service(CurrencyServer::new(service)).serve(addr).await?;

    Ok(())
}
