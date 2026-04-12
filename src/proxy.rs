use anyhow::{Context, Result, anyhow};
use axum::body::{Body, Bytes};
use axum::extract::{ConnectInfo, State};
use axum::http::header::{CONNECTION, HOST};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use axum::response::Response;
use axum::routing::any;
use axum::{Router, serve};
use futures_util::stream::StreamExt;
use http_body_util::BodyDataStream;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio::task::JoinSet;
use tracing::{error, info, warn};
use url::Url;

use crate::config::{ResolvedConfig, ResolvedListener};

#[derive(Clone)]
struct ProxyState {
    listener_name: String,
    upstream_base: Url,
    connect_addr: SocketAddr,
    client: reqwest::Client,
    host_header: HeaderValue,
}

pub async fn run(config: ResolvedConfig) -> Result<()> {
    let mut join_set: JoinSet<Result<()>> = JoinSet::new();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    for listener in config.listeners {
        let state = build_state(&listener)?;
        let app = Router::new()
            .fallback(any(proxy_handler))
            .with_state(Arc::new(state));

        let bind_addr = listener.listen_addr;
        let listener_name = listener.name.clone();
        let tcp = TcpListener::bind(bind_addr)
            .await
            .with_context(|| format!("failed to bind {bind_addr} for {}", listener.name))?;

        let mut rx = shutdown_rx.clone();
        join_set.spawn(async move {
            info!(
                listener = %listener_name,
                bind = %bind_addr,
                "listener started"
            );

            serve(tcp, app.into_make_service_with_connect_info::<SocketAddr>())
                .with_graceful_shutdown(async move {
                    let _ = rx.changed().await;
                })
                .await
                .with_context(|| format!("listener {} terminated with error", listener_name))
        });
    }

    tokio::select! {
        _ = shutdown_signal() => {
            info!("shutdown signal received");
            let _ = shutdown_tx.send(true);
            drain_join_set(&mut join_set).await
        }
        first = join_set.join_next() => {
            let _ = shutdown_tx.send(true);
            let first_err = match first {
                Some(Ok(Ok(()))) => anyhow!("a listener exited unexpectedly"),
                Some(Ok(Err(err))) => err,
                Some(Err(err)) => anyhow!("listener task join error: {err}"),
                None => anyhow!("no listener tasks running"),
            };

            if let Err(err) = drain_join_set(&mut join_set).await {
                warn!(error = %err, "encountered extra errors while shutting down listeners");
            }

            Err(first_err)
        }
    }
}

async fn drain_join_set(join_set: &mut JoinSet<Result<()>>) -> Result<()> {
    let mut first_error: Option<anyhow::Error> = None;

    while let Some(join_result) = join_set.join_next().await {
        match join_result {
            Ok(Ok(())) => {}
            Ok(Err(err)) => {
                error!(error = %err, "listener exited with error");
                if first_error.is_none() {
                    first_error = Some(err);
                }
            }
            Err(err) => {
                let wrapped = anyhow!("listener task join error: {err}");
                error!(error = %wrapped, "listener task failed");
                if first_error.is_none() {
                    first_error = Some(wrapped);
                }
            }
        }
    }

    if let Some(err) = first_error {
        return Err(err);
    }

    Ok(())
}

fn build_state(listener: &ResolvedListener) -> Result<ProxyState> {
    let client = reqwest::Client::builder()
        .resolve(&listener.server_name, listener.connect_addr)
        .connect_timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .build()
        .with_context(|| format!("failed to create HTTP client for {}", listener.name))?;

    let host_header = HeaderValue::from_str(&listener.host_header)
        .with_context(|| format!("invalid host_header for {}", listener.name))?;

    Ok(ProxyState {
        listener_name: listener.name.clone(),
        upstream_base: listener.upstream_base.clone(),
        connect_addr: listener.connect_addr,
        client,
        host_header,
    })
}

async fn proxy_handler(
    State(state): State<Arc<ProxyState>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Response {
    let target_url = match build_target_url(&state.upstream_base, &uri) {
        Ok(url) => url,
        Err(err) => {
            error!(listener = %state.listener_name, error = %err, "failed to build target URL");
            return text_response(StatusCode::BAD_GATEWAY, "bad gateway: invalid upstream URL");
        }
    };

    let mut req_builder = state.client.request(method.clone(), target_url.clone());

    for (name, value) in &headers {
        if is_hop_by_hop_header(name) || *name == HOST {
            continue;
        }
        req_builder = req_builder.header(name, value);
    }

    req_builder = req_builder.header(HOST, state.host_header.clone());

    let xff = append_forwarded_for(headers.get("x-forwarded-for"), peer.ip().to_string());
    req_builder = req_builder.header("x-forwarded-for", xff);

    let req_body = reqwest::Body::wrap_stream(BodyDataStream::new(body));
    req_builder = req_builder.body(req_body);

    let upstream_response = match req_builder.send().await {
        Ok(resp) => resp,
        Err(err) => {
            error!(
                listener = %state.listener_name,
                method = %method,
                target = %target_url,
                connect = %state.connect_addr,
                error = %err,
                error_debug = ?err,
                "upstream request failed"
            );
            return text_response(
                StatusCode::BAD_GATEWAY,
                "bad gateway: upstream request failed",
            );
        }
    };

    let status = upstream_response.status();
    let response_headers = upstream_response.headers().clone();
    let stream = upstream_response
        .bytes_stream()
        .map(|chunk| -> Result<Bytes, reqwest::Error> { chunk });

    let mut response_builder = Response::builder().status(status);
    for (name, value) in &response_headers {
        if is_hop_by_hop_header(name) || *name == CONNECTION {
            continue;
        }
        response_builder = response_builder.header(name, value);
    }

    match response_builder.body(Body::from_stream(stream)) {
        Ok(response) => response,
        Err(err) => {
            error!(listener = %state.listener_name, error = %err, "failed to build response");
            text_response(StatusCode::INTERNAL_SERVER_ERROR, "internal proxy error")
        }
    }
}

fn build_target_url(base: &Url, incoming_uri: &Uri) -> Result<Url> {
    let mut target = base.clone();

    let base_path = base.path().trim_end_matches('/');
    let request_path = incoming_uri.path().trim_start_matches('/');

    let merged_path = match (base_path.is_empty(), request_path.is_empty()) {
        (true, true) => "/".to_string(),
        (true, false) => format!("/{request_path}"),
        (false, true) => format!("{base_path}/"),
        (false, false) => format!("{base_path}/{request_path}"),
    };

    target.set_path(&merged_path);
    target.set_query(incoming_uri.query());

    Ok(target)
}

fn append_forwarded_for(existing: Option<&HeaderValue>, peer_ip: String) -> String {
    match existing.and_then(|h| h.to_str().ok()) {
        Some(value) if !value.trim().is_empty() => format!("{value}, {peer_ip}"),
        _ => peer_ip,
    }
}

fn is_hop_by_hop_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str().to_ascii_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}

fn text_response(status: StatusCode, message: &'static str) -> Response {
    let mut response = Response::new(Body::from(message));
    *response.status_mut() = status;
    response.headers_mut().insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    response
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(sig) => sig,
            Err(err) => {
                warn!(error = %err, "failed to register SIGTERM handler; falling back to ctrl-c only");
                let _ = tokio::signal::ctrl_c().await;
                return;
            }
        };

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = sigterm.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_path_and_query() {
        let base = Url::parse("https://example.com:8443/api").expect("base URL");
        let uri: Uri = "/v1/chat/completions?stream=true".parse().expect("uri");

        let out = build_target_url(&base, &uri).expect("target URL");
        assert_eq!(
            out.as_str(),
            "https://example.com:8443/api/v1/chat/completions?stream=true"
        );
    }

    #[test]
    fn keeps_root_when_no_path() {
        let base = Url::parse("https://example.com:8443").expect("base URL");
        let uri: Uri = "/".parse().expect("uri");

        let out = build_target_url(&base, &uri).expect("target URL");
        assert_eq!(out.as_str(), "https://example.com:8443/");
    }

    #[test]
    fn appends_xff_chain() {
        let existing = HeaderValue::from_static("1.1.1.1");
        let out = append_forwarded_for(Some(&existing), "2.2.2.2".to_string());
        assert_eq!(out, "1.1.1.1, 2.2.2.2");
    }
}
