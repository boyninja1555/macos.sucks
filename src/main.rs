use gurtlib::{GurtStatusCode, prelude::*};
use std::path::Path;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let server: GurtServer =
        GurtServer::with_tls_certificates("tls/localhost+2.pem", "tls/localhost+2-key.pem")?
            .get("/", |_ctx| async move {
                match fs::read("./www/index.html").await {
                    Ok(data) => Ok(GurtResponse::ok()
                        .with_header("content-type", "text/html")
                        .with_body(data)),
                    Err(_) => {
                        Ok(GurtResponse::not_found().with_string_body("Index file not found"))
                    }
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
                            let content_type: &'static str = match Path::new(&full_path)
                                .extension()
                                .and_then(|ext| ext.to_str())
                            {
                                Some("html") => "text/html",
                                Some("css") => "text/css",
                                Some("js") => "application/javascript",
                                Some("json") => "application/json",
                                Some("png") => "image/png",
                                Some("jpg") | Some("jpeg") => "image/jpeg",
                                Some("gif") => "image/gif",
                                Some("svg") => "image/svg+xml",
                                _ => "application/octet-stream",
                            };

                            Ok(GurtResponse::ok()
                                .with_header("content-type", content_type)
                                .with_body(data))
                        }
                        Err(_) => Ok(GurtResponse::not_found().with_string_body("File not found")),
                    }
                }
            });

    server.listen("0.0.0.0:4878").await
}
