use envie::Envie;
use gurtlib::{GurtStatusCode, prelude::*};
use std::path::Path;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let env: Envie = Envie::load().expect("Failed to load environment variables file");

    let port: i32 = (env.get("PORT").unwrap_or_else(|| "4878".to_string()))
        .parse::<i32>()
        .expect("Failed to parse port");

    let cert_path: String = env
        .get("CERT_PATH")
        .unwrap_or_else(|| "tls/localhost+2.pem".to_string());

    let key_path: String = env
        .get("KEY_PATH")
        .unwrap_or_else(|| "tls/localhost+2-key.pem".to_string());

    let server: GurtServer = GurtServer::with_tls_certificates(&cert_path, &key_path)?
        .get("/", |_ctx| async move {
            match fs::read_to_string("./www/index.html").await {
                Ok(mut data) => {
                    let pub_url: String = Envie::load()
                        .expect("Failed to load environment variables file")
                        .get("PUB_URL")
                        .unwrap_or_else(|| "gurt://127.0.0.1:4878".to_string());

                    data = data.replace("{pub}", &pub_url);
                    Ok(GurtResponse::ok()
                        .with_header("content-type", "text/html")
                        .with_body(data))
                }
                Err(_) => Ok(GurtResponse::not_found().with_string_body("File not found")),
            }
        })
        .get("/*", |ctx: &ServerContext| {
            let ctx: ServerContext = ctx.clone();
            async move {
                let path: &str = ctx.path();
                let file_path: &str = path.strip_prefix("/").unwrap_or("");
                if file_path.contains("..") {
                    return Ok(GurtResponse::new(GurtStatusCode::Forbidden)
                        .with_string_body("Access denied"));
                }

                let full_path: String = format!("./www/{}", file_path);
                match fs::read(&full_path).await {
                    Ok(data) => {
                        let ext: Option<&str> =
                            Path::new(&full_path).extension().and_then(|e| e.to_str());

                        let content_type: &'static str = match ext {
                            Some("html") => "text/html",
                            Some("css") => "text/css",
                            Some("js") => "application/javascript",
                            Some("lua") => "application/lua",
                            Some("json") => "application/json",
                            Some("png") => "image/png",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("gif") => "image/gif",
                            Some("svg") => "image/svg+xml",
                            _ => "application/octet-stream",
                        };

                        if matches!(
                            content_type,
                            "text/html"
                                | "text/css"
                                | "application/javascript"
                                | "application/json"
                        ) {
                            if let Ok(mut text) = String::from_utf8(data.clone()) {
                                let pub_url: String = Envie::load()
                                    .expect("Failed to load environment variables file")
                                    .get("PUB_URL")
                                    .unwrap_or_else(|| "gurt://127.0.0.1:4878".to_string());

                                text = text.replace("{pub}", &pub_url);
                                return Ok(GurtResponse::ok()
                                    .with_header("content-type", content_type)
                                    .with_body(text));
                            }
                        }

                        Ok(GurtResponse::ok()
                            .with_header("content-type", content_type)
                            .with_body(data))
                    }
                    Err(_) => Ok(GurtResponse::not_found().with_string_body("File not found")),
                }
            }
        });

    server
        .listen(&format!("0.0.0.0:{}", port.to_string()))
        .await
}
