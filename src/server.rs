use crate::sensor::get_conditions_json;
use actix_web::{App, HttpServer, web, Responder, HttpResponse, Result, error};
use actix_web::http::header;
use std::time::{SystemTime, UNIX_EPOCH};
use actix_files::Files;

async fn conditions() -> Result<impl Responder> {
    let (json, expiration) = get_conditions_json().await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok()
        .body(json)
        .with_header(header::EXPIRES, expiration.unwrap_or_else(now)))
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub async fn start_server() -> std::io::Result<()> {
        HttpServer::new(|| {
            App::new()
                .route("/conditions", web::get().to(conditions))
                .service(Files::new("/", "./public").prefer_utf8(true).index_file("index.html"))
                .service(Files::new("/js/", "./public/js").prefer_utf8(true))
                .service(Files::new("/stylesheets/", "./public/stylesheets").prefer_utf8(true))
        })
        .bind("0.0.0.0:80")?
        .run()
        .await
}
