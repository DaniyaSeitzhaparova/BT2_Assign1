use actix_files::NamedFile;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use serde::{Serialize};
use std::env;
use reqwest::Client;

#[derive(Serialize)]
struct CoinData {
    name: String,
    symbol: String,
    price_usd: f64,
}

#[derive(Serialize)]
struct NewsArticle {
    title: String,
    source: String,
    date: String,
    summary: String,
    url: String,
}

#[derive(Serialize)]
struct ApiResponse {
    coin: Option<CoinData>,
    news: Vec<NewsArticle>,
}

#[get("/api/{crypto}")]
async fn fetch_data(crypto: web::Path<String>) -> impl Responder {
    let crypto = crypto.into_inner();
    let data = get_crypto_info(&crypto).await;
    match data {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(_) => HttpResponse::InternalServerError().body("Failed to fetch data"),
    }
}

#[get("/")]
async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    HttpServer::new(|| {
        App::new()
            .service(fetch_data)
            .service(index)
            .service(actix_files::Files::new("/", "./static").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn get_crypto_info(symbol: &str) -> Result<ApiResponse, reqwest::Error> {
    let coingecko_url = env::var("COINGECKO_API_URL").unwrap();
    let cryptonews_url = env::var("CRYPTONEWS_API_URL").unwrap();
    let news_api_key = env::var("CRYPTONEWS_API_KEY").unwrap();
    
    let client = Client::new();
    
    let coin_url = format!("{}/coins/{}", coingecko_url, symbol);
    let coin_resp: serde_json::Value = client
        .get(&coin_url)
        .send()
        .await?
        .json()
        .await?;

    let coin_data = CoinData {
        name: coin_resp["name"].as_str().unwrap_or_default().to_string(),
        symbol: coin_resp["symbol"].as_str().unwrap_or_default().to_string(),
        price_usd: coin_resp["market_data"]["current_price"]["usd"]
            .as_f64()
            .unwrap_or_default(),
    };

    let news_url = format!(
        "{}?q=cryptocurrency&apiKey={}&pageSize=3", 
        cryptonews_url, news_api_key
    );

    let news_resp: serde_json::Value = client
        .get(&news_url)
        .send()
        .await?
        .json()
        .await?;

    let news = news_resp["articles"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|article| NewsArticle {
            title: article["title"].as_str().unwrap_or_default().to_string(),
            source: article["source"]["name"].as_str().unwrap_or_default().to_string(),
            date: article["publishedAt"].as_str().unwrap_or_default().to_string(),
            summary: article["description"].as_str().unwrap_or_default().to_string(),
            url: article["url"].as_str().unwrap_or_default().to_string(),
        })
        .collect();

    Ok(ApiResponse {
        coin: Some(coin_data),
        news,
    })
}
