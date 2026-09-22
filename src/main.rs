mod models;
mod register;
mod tests;
mod util;

use crate::models::Config;
use crate::util::{ddnet, HealthStatus};
use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::header::{CACHE_CONTROL, CONTENT_TYPE};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use log::{debug, error, info, trace};
use prometheus::{Encoder, TextEncoder};
use std::net::SocketAddr;
use tokio::net::TcpListener;

type BoxBody = Full<Bytes>;

async fn serve_req(
    req: Request<Incoming>,
    health: HealthStatus,
    max_health_age: u64,
) -> Result<Response<BoxBody>, hyper::Error> {
    if req.method() != Method::GET && req.method() != Method::HEAD {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Full::new(Bytes::from("Method Not Allowed\n")))
            .unwrap());
    }

    let path = req.uri().path();

    match path {
        "/health" | "/healthz" | "/live" | "/ready" => {
            let (is_healthy, age) = health.check(max_health_age);
            if is_healthy {
                let body = if req.method() == Method::HEAD {
                    Bytes::new()
                } else {
                    Bytes::from(format!("OK (age={age}s)\n"))
                };
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .header(CACHE_CONTROL, "no-cache, no-store, must-revalidate")
                    .body(Full::new(body))
                    .unwrap())
            } else {
                let body = if req.method() == Method::HEAD {
                    Bytes::new()
                } else {
                    Bytes::from(format!(
                        "Service Unavailable (age={age}s, uninitialized or stale)\n"
                    ))
                };
                Ok(Response::builder()
                    .status(StatusCode::SERVICE_UNAVAILABLE)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .header(CACHE_CONTROL, "no-cache, no-store, must-revalidate")
                    .body(Full::new(body))
                    .unwrap())
            }
        }
        "/metrics" | "/" => {
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = vec![];
            if let Err(err) = encoder.encode(&metric_families, &mut buffer) {
                error!("Failed to encode metrics: {err}");
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                    .body(Full::new(Bytes::from("Failed to encode metrics\n")))
                    .unwrap());
            }

            let body = if req.method() == Method::HEAD {
                Bytes::new()
            } else {
                Bytes::from(buffer)
            };

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, encoder.format_type())
                .body(Full::new(body))
                .unwrap())
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Full::new(Bytes::from("Not Found\n")))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env().expect("Failed loading config from env");
    config.set_logging();

    let addr: SocketAddr = ([0, 0, 0, 0], config.web_port).into();
    let listener = TcpListener::bind(addr).await.expect("Failed to bind TCP listener");

    info!(
        "Listening on http://0.0.0.0:{0} (metrics: http://127.0.0.1:{0}/metrics, health: http://127.0.0.1:{0}/health)",
        config.web_port
    );

    let health = HealthStatus::new();
    let max_health_age = (config.delay * 3).max(60);

    tokio::spawn(ddnet(config, health.clone()));

    let (tx_shutdown, mut rx_shutdown) = tokio::sync::watch::channel(false);

    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        info!("Shutdown signal received, closing listener...");
        let _ = tx_shutdown.send(true);
    });

    loop {
        tokio::select! {
            accept_res = listener.accept() => {
                match accept_res {
                    Ok((stream, addr)) => {
                        trace!("Accepted connection from {addr}");

                        let io = TokioIo::new(stream);
                        let health_clone = health.clone();
                        let service = service_fn(move |req| {
                            let health_inner = health_clone.clone();
                            serve_req(req, health_inner, max_health_age)
                        });

                        tokio::spawn(async move {
                            if let Err(err) = http1::Builder::new()
                                .serve_connection(io, service)
                                .await
                            {
                                // Client disconnects during BodyWrite or Read are normal
                                debug!("Connection from {addr} closed: {err}");
                            }
                        });
                    }
                    Err(err) => {
                        error!("Accept connection error: {err:?}");
                    }
                }
            }
            _ = rx_shutdown.changed() => {
                info!("Server shutting down gracefully");
                break;
            }
        }
    }

    Ok(())
}
