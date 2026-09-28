use anyhow::Result;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use rust_embed::RustEmbed;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(RustEmbed)]
#[folder = "web/"]
struct WebAssets;

/// UI encryption key, set at startup based on license.
/// If None — crypto binding disabled (Community fallback).
pub type UiKey = Arc<tokio::sync::RwLock<Option<[u8; 32]>>>;

pub async fn run_server(token: String, ui_key: UiKey) -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("HTTP: http://{}", addr);
    let listener = TcpListener::bind(addr).await?;
    let token = Arc::new(token);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let token = token.clone();
        let ui_key = ui_key.clone();

        tokio::task::spawn(async move {
            let svc = service_fn(move |req| {
                let token = token.clone();
                let ui_key = ui_key.clone();
                async move { handle_request(req, token, ui_key).await }
            });
            if let Err(e) = http1::Builder::new().serve_connection(io, svc).await {
                tracing::debug!("connection error: {}", e);
            }
        });
    }
}

async fn handle_request(
    req: Request<hyper::body::Incoming>,
    token: Arc<String>,
    ui_key: UiKey,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path();
    let path = if path == "/" {
        "index.html"
    } else {
        &path[1..]
    };

    // Session token for WebSocket auth
    if path == "api/token" {
        return Ok(Response::builder()
            .header("Content-Type", "application/json")
            .header("X-Content-Type-Options", "nosniff")
            .body(Full::new(Bytes::from(format!(
                "{{\"token\":\"{}\"}}",
                token
            ))))
            .unwrap());
    }

    // UI encryption key (for crypto binding)
    if path == "api/ui-key" {
        let key = ui_key.read().await;
        return match key.as_ref() {
            Some(k) => {
                let hex_key: String = k.iter().map(|b| format!("{:02x}", b)).collect();
                Ok(Response::builder()
                    .header("Content-Type", "application/json")
                    .header("X-Content-Type-Options", "nosniff")
                    .body(Full::new(Bytes::from(format!(
                        "{{\"key\":\"{}\"}}",
                        hex_key
                    ))))
                    .unwrap())
            }
            None => Ok(Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from("{\"error\":\"no-license\"}")))
                .unwrap()),
        };
    }

    match WebAssets::get(path) {
        Some(c) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Ok(Response::builder()
                .header("Content-Type", mime.as_ref())
                .header("X-Content-Type-Options", "nosniff")
                .header("Referrer-Policy", "no-referrer")
                .body(Full::new(Bytes::from(c.data.to_vec())))
                .unwrap())
        }
        None => match WebAssets::get("index.html") {
            Some(c) => Ok(Response::builder()
                .header("Content-Type", "text/html")
                .body(Full::new(Bytes::from(c.data.to_vec())))
                .unwrap()),
            None => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("404")))
                .unwrap()),
        },
    }
}