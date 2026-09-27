use axum::{
    Router,
    http::{self, header},
    response::Html,
    routing::{get, post},
};
use axum_client_ip::ClientIpSource;
use log::LevelFilter;
use std::{env, error::Error, net::SocketAddr};
use tokio_rusqlite::Connection;
use tower_http::cors::Any;

use crate::handlers::{AppState, decrement_count, get_count, increment_count, initialize};

mod handlers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    let example_html = include_str!("../example/index.html");
    let script = include_str!("./shc.js");
    let db = Connection::open("./shc.db").await?;

    initialize(&db).await?;

    let app = Router::new()
        .route("/health", get(http::StatusCode::OK))
        .route("/", get(Html(example_html)))
        .route(
            "/shc.js",
            get(([(header::CONTENT_TYPE, "text/javascript")], script)),
        )
        .route("/count", post(get_count))
        .route("/count/increment", post(increment_count))
        .route("/count/decrement", post(decrement_count))
        .layer(ClientIpSource::ConnectInfo.into_extension())
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(AppState { db });

    let host = &env::var("HOST").unwrap_or("localhost".into());
    let port = &env::var("PORT").unwrap_or("3001".into());
    let addr = format!("{}:{}", host, port); // TODO: SocketAddr
    log::info!("starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?)
}
