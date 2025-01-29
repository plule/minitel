//! Example minitel web application using axum.
//!
//! It serves a websocket on /ws

use axum::{
    extract::ws::WebSocketUpgrade,
    http::StatusCode,
    response::IntoResponse,
    routing::{any, post},
    Json, Router,
};
use base64::Engine;
use clap::Parser;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use std::{collections::HashMap, net::SocketAddr, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;

use crate::app::App;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Address to bind to
    #[clap(short, long, default_value = "127.0.0.1:3615")]
    bind: String,
    /// Public host for redirections. Must include the port, no http://
    #[clap(short, long, default_value = None)]
    minipavi_host: Option<String>,
    /// Minipavi protocol, either http or https
    #[clap(long, default_value = "http")]
    minipavi_proto: String,
    /// Open the browser on startup
    #[clap(long)]
    open: bool,
}

#[tokio::main]
pub async fn main() {
    let args = Args::parse();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=info,tower_http=info", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");

    let mut app = Router::new()
        // web ui with the emulator
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        // websocket route
        .route("/ws", any(ws_handler));
    if let Some(_) = args.minipavi_host.as_ref() {
        // minipavi api route
        app = app.route("/minipavi", post(minipavi));
    }

    // build our application with some routes
    app = app.layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );

    if args.open {
        // Open the browser on startup
        if let Err(e) = open::that(format!("http://{}", args.bind)) {
            warn!("Failed to open the browser: {}", e);
        }
    }

    info!("Listening on {}", args.bind);
    let listener = tokio::net::TcpListener::bind(args.bind).await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

/// The main entrypoint of the application: handle a websocket connection by running the ratatui app
async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("Client at {addr} connected.");
    ws.on_upgrade(move |socket| async move {
        let mut port = minitel::axum::Port::new(socket);
        match App::default().run(&mut port).await {
            Ok(()) => info!("Client {addr} terminated normally"),
            Err(e) => warn!("Client {addr} terminated with error: {e}"),
        }
    })
}

/// Minipavi entrypoint: Redirect to the websocket then exit
async fn minipavi(Json(payload): Json<PasserelleMessage>) -> (StatusCode, Json<ServiceMessage>) {
    let args = Args::parse();

    let rep;
    match payload.pavi.fctn.as_str() {
        "DIRECTCNX" => {
            // Initial connection, redirect to the websocket
            rep = ServiceMessage {
                version: "1".to_string(),
                content: base64::prelude::BASE64_STANDARD.encode(""),
                context: "context".to_string(),
                echo: "on".to_string(),
                next: format!(
                    "{}://{}/minipavi",
                    args.minipavi_proto,
                    args.minipavi_host.as_ref().unwrap()
                ),
                directcall: "no".to_string(),
                command: Command {
                    name: "connectToWs".to_string(),
                    param: [
                        ("host", args.minipavi_host.unwrap().as_str()),
                        ("key", ""),
                        ("path", "/ws"),
                        ("echo", "on"),
                        ("case", "upper"),
                        ("proto", ""),
                    ]
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .into_iter()
                    .collect(),
                },
            };
        }
        "DIRECTCALLENDED" | "FIN" => {
            // Call with the websocket ended, send the exit command
            rep = ServiceMessage {
                version: "1".to_string(),
                content: base64::prelude::BASE64_STANDARD.encode(""),
                context: "context".to_string(),
                echo: "off".to_string(),
                next: "".to_string(),
                directcall: "no".to_string(),
                command: Command {
                    name: "libCnx".to_string(),
                    param: HashMap::new(),
                },
            };
        }
        _ => {
            // Unknown function, send the exit command
            error!("Unknown function {}", payload.pavi.fctn);
            rep = ServiceMessage {
                version: "1".to_string(),
                content: base64::prelude::BASE64_STANDARD.encode(""),
                context: "context".to_string(),
                echo: "off".to_string(),
                next: "".to_string(),
                directcall: "no".to_string(),
                command: Command {
                    name: "libCnx".to_string(),
                    param: HashMap::new(),
                },
            };
        }
    }
    (StatusCode::OK, rep.into())
}

/// A message from the minipavi server to this service
#[derive(Debug, Serialize, Deserialize)]
struct PasserelleMessage {
    #[serde(rename = "PAVI")]
    pavi: PaviMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct PaviMessage {
    content: Vec<String>,
    context: String,
    fctn: String,
    #[serde(rename = "remoteAddr")]
    remote_addr: String,
    typesocket: String,
    #[serde(rename = "uniqueId")]
    unique_id: String,
    version: String,
    versionminitel: String,
}

/// A message from this service to the minipavi server
#[derive(Debug, Serialize, Deserialize)]
struct ServiceMessage {
    version: String,
    content: String,
    context: String,
    echo: String,
    next: String,
    directcall: String,
    #[serde(rename = "COMMAND")]
    command: Command,
}

#[derive(Debug, Serialize, Deserialize)]
struct Command {
    name: String,
    param: HashMap<String, String>,
}
