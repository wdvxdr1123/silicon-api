use std::collections::HashMap;

use actix_web::{web, HttpResponse};
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
pub struct NeneParams {
    words: String,
}

#[derive(Serialize)]
pub struct NeneResponse {
    count: u32,
    error: Option<String>,
    replies: Option<Vec<String>>,
}

impl NeneResponse {
    pub fn ok(relies: Vec<String>) -> Self {
        NeneResponse {
            count: relies.len() as u32,
            error: None,
            replies: Some(relies.into()),
        }
    }
}

impl Into<HttpResponse> for NeneResponse {
    fn into(self) -> HttpResponse {
        match self.error {
            Some(_) => panic!("this is a error!"),
            None => HttpResponse::Ok().json(&self),
        }
    }
}

pub async fn nene(params: web::Query<NeneParams>) -> HttpResponse {
    NeneResponse::ok(
        AC.find_iter(params.words.as_str())
            .filter_map(|matched| {
                DICT.get(KEYS[matched.pattern()].as_str()).and_then(|x| {
                    x.clone()
                        .iter()
                        .choose(&mut rand::thread_rng())
                        .map(|x| x.to_string())
                })
            })
            .collect::<Vec<_>>(),
    )
    .into()
}
