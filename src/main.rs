use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::{self, VarError};
use std::fmt::Debug;
use std::str::FromStr;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

// // Use Jemalloc only for musl-64 bits platforms
// #[cfg(all(target_env = "musl", target_pointer_width = "64"))]
// #[global_allocator]
// static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

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
    base: String,
}

impl Default for CurrencyService {
    fn default() -> CurrencyService {
        CurrencyService {
            url: "".to_owned(),
            base: "USD".to_owned(),
        }
    }
}

#[tonic::async_trait]
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
        let r = get_current_data(&self.url, &b).await;

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

async fn get_current_data(url: &str, base: &str) -> Result<Rate, Box<dyn std::error::Error>> {
    let url = format!("{}/{}", url, base);
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
    // Load environment variables from .env file.
    // Fails if .env file not found, not readable or invalid.
    dotenvy::from_path("./config/.env")?;

    let mut port = get_env_var::<u32>("PORT");

    if port == 0 {
        port = 8001
    }

    let addr = format!("0.0.0.0:{:?}", port).parse().unwrap();

    println!("Currency server listening on: {}", addr);

    let service: CurrencyService = CurrencyService {
        url: get_env_var::<String>("URL"),
        ..Default::default()
    };

    let svc: CurrencyServer<CurrencyService> = CurrencyServer::new(service);

    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
