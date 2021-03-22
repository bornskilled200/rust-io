use crate::sensor::get_conditions_json;
use actix_web::{App, HttpServer, web, Responder, HttpResponse, Result, error};
use actix_web::http::header;
use std::time::{SystemTime, UNIX_EPOCH};

async fn conditions() -> Result<impl Responder> {
    let (json, expiration) = get_conditions_json().await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok()
        .body(json)
        .with_header((header::EXPIRES, expiration.unwrap_or_else(now))))
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub async fn start_server() -> std::io::Result<()> {
        HttpServer::new(|| {
            App::new()
                .route("/conditions", web::get().to(conditions))
        })
        .bind("0.0.0.0:80")?
        .run()
        .await
}
