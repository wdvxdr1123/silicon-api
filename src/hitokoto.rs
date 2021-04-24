use actix_web::{web, HttpResponse, Result};
use lazy_static::lazy_static;
use rand::seq::IteratorRandom;
use redis::{cmd, Client};
use serde::Deserialize;

lazy_static! {
    static ref CATEGORY: Vec<&'static str> = vec![
        "anime",
        "comic",
        "game",
        "literature",
        "original",
        "internet",
        "other",
        "video",
        "poem",
        "ncm",
        "philosophy",
        "funny",
    ];
    static ref CATEGORY_RANDOM: Vec<&'static str> = vec![
        "anime",
        "comic",
        "game",
        "literature",
        "original",
        "internet",
        "video",
    ];
    static ref REDIS_CLIENT: Client = Client::open("redis://127.0.0.1/").unwrap();
}

#[derive(Deserialize)]
pub struct HitokotoParam {
    category: Option<String>,
}

pub async fn hitokoto(params: web::Query<HitokotoParam>) -> Result<HttpResponse> {
    let category = params.category.clone().unwrap_or_else(|| {
        CATEGORY_RANDOM
            .iter()
            .choose(&mut rand::thread_rng())
            .unwrap_or(&"anime")
            .to_string()
    });

    let category = CATEGORY
        .iter()
        .find(|x| x.to_string() == category)
        .ok_or_else(|| HttpResponse::BadRequest().body(""))?
        .to_string();

    let mut con = REDIS_CLIENT.get_tokio_connection_tokio().await.unwrap();

    let r = cmd("RPOPLPUSH")
        .arg(category.clone())
        .arg(category.clone())
        .query_async::<_, String>(&mut con)
        .await
        .map_err(|x| HttpResponse::BadRequest().body(x.to_string()))?;

    let sentence = serde_json::from_str::<serde_json::Value>(r.as_str())
        .map_err(|_| HttpResponse::InternalServerError().body("get sentence failed"))?;

    Ok(HttpResponse::Ok().json(sentence))
}
