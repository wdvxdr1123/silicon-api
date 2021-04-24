use std::collections::HashMap;

use actix_web::{web, HttpResponse, Result};
use aho_corasick::AhoCorasick;
use lazy_static::lazy_static;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};

lazy_static! {
    static ref DICT: HashMap<String, Vec<String>> = {
        reqwest::blocking::get("https://cdn.jsdelivr.net/gh/Kyomotoi/AnimeThesaurus@main/data.json")
            .unwrap()
            .json::<HashMap<String, Vec<String>>>()
            .unwrap()
    };
    static ref KEYS: Vec<String> = DICT.keys().map(|x| x.clone()).collect::<Vec<_>>();
    static ref AC: AhoCorasick = AhoCorasick::new(KEYS.iter());
}

#[derive(Deserialize)]
pub struct ECYParams {
    words: String,
}

#[derive(Serialize)]
pub struct ECYResponse {
    count: u32,
    replies: Vec<String>,
}

pub async fn nene(params: web::Query<ECYParams>) -> Result<HttpResponse> {
    let mut rng = rand::thread_rng();
    let mut resp = ECYResponse {
        count: 0,
        replies: vec![],
    };

    for matched in AC.find_iter(params.words.as_str()) {
        let reply = DICT
            .get(KEYS[matched.pattern()].as_str())
            .ok_or_else(|| HttpResponse::BadRequest().finish())?
            .iter()
            .choose(&mut rng)
            .ok_or_else(|| HttpResponse::BadRequest().body("get rand item failed"))?;

        resp.replies.push(reply.to_string());
    }

    resp.count = resp.replies.len() as u32;
    Ok(HttpResponse::Ok().json(&resp))
}
