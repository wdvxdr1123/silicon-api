mod ecy;

use std::{borrow::Borrow, fs, io};

use actix_web::{post, web, App, Error, HttpResponse, HttpServer, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use silicon::formatter::{ImageFormatter, ImageFormatterBuilder};
use silicon::utils;
use uuid::Uuid;

use actix_web::web::{get, resource};
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    port: u32,
    host: String,
    base_url: String,
}

lazy_static! {
    static ref SYNTECT: (SyntaxSet, ThemeSet) = utils::init_syntect();
    static ref CONFIG: Config = {
        let config_file = fs::read("config.json")
            .map_err(|e| {
                fs::write(
                    "config.json",
                    serde_json::to_string(&Config {
                        port: 8080,
                        host: "127.0.0.1".to_string(),
                        base_url: "http://127.0.0.1".to_string(),
                    })
                    .unwrap(),
                )
                .unwrap();
                e
            })
            .unwrap();

        let config: Config = serde_json::from_slice(config_file.as_slice()).unwrap();
        config
    };
}

#[derive(Debug, Deserialize)]
pub struct Format {
    pub theme: String,
    pub language: String,
    pub line_pad: u32,
    pub line_offset: u32,
    pub tab_width: u8,
}

impl Default for Format {
    fn default() -> Self {
        Format {
            theme: "Dracula".to_string(),
            language: "rs".to_string(),
            line_pad: 2,
            line_offset: 1,
            tab_width: 4,
        }
    }
}

pub fn get_formatter(format: &Format) -> Result<ImageFormatter, Error> {
    let formatter = ImageFormatterBuilder::<&str>::new()
        .line_pad(format.line_pad)
        .tab_width(format.tab_width)
        .line_offset(format.line_offset);
    formatter.build().map_err(|_| {
        HttpResponse::BadRequest()
            .json(&SiliconResp {
                code: 100,
                err: Some("bad format!".to_string()),
                url: None,
            })
            .into()
    })
}

#[derive(Debug, Deserialize)]
pub struct SiliconReq {
    code: String,
    #[serde(default)]
    format: Format,
}

#[derive(Debug, Serialize)]
pub struct SiliconResp {
    code: u32,
    err: Option<String>,
    url: Option<String>,
}

#[post("/silicon")]
async fn code_to_image(input: web::Json<SiliconReq>) -> Result<HttpResponse, Error> {
    let ps = SYNTECT.0.borrow();
    let language = ps
        .find_syntax_by_token(input.format.language.as_str())
        .ok_or_else(|| {
            HttpResponse::BadRequest().json(&SiliconResp {
                code: 101,
                err: Some("unknown language".to_string()),
                url: None,
            })
        })?;

    let theme = SYNTECT
        .1
        .themes
        .get(input.format.theme.as_str())
        .ok_or_else(|| {
            HttpResponse::BadRequest().json(&SiliconResp {
                code: 102,
                err: Some("unknown theme".to_string()),
                url: None,
            })
        })?;

    let mut h = HighlightLines::new(language, theme);
    let highlight = LinesWithEndings::from(input.code.as_str())
        .map(|line| h.highlight(line, ps))
        .collect::<Vec<_>>();

    let mut formatter = get_formatter(&input.format)?;
    let image = formatter.format(highlight.as_ref(), theme);
    let file = format!("{}.png", Uuid::new_v4());
    image.save(format!("images/{}", file)).map_err(|_| {
        HttpResponse::BadRequest().json(&SiliconResp {
            code: 103,
            err: Some("save file error".to_string()),
            url: None,
        })
    })?;

    println!("generate image {}.png successfully!", file);
    Ok(HttpResponse::Ok().json(&SiliconResp {
        code: 200,
        err: None,
        url: Some(format!("{}/{}", CONFIG.base_url, file)),
    }))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    fs::create_dir("images").unwrap_or_default();

    let _ = HttpServer::new(|| {
        App::new()
            .service(code_to_image)
            .service(resource("/nene").route(get().to(ecy::nene)))
    })
    .bind(format!("{}:{}", CONFIG.host, CONFIG.port))?
    .run()
    .await;
    Ok(())
}
