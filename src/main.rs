use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use axum_client_ip::{ClientIp, ClientIpSource};
use serde::Serialize;
use std::{env, error::Error, fs, net::SocketAddr};
use tokio_rusqlite::{Connection, params};
use tower_http::cors::Any;

#[derive(Clone)]
struct AppState {
    db: Connection,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let script = fs::read_to_string("shc.js").expect("read shc.js");
    let db = Connection::open("./shc.db").await?;

    initialize(&db).await?;

    let app = Router::new()
        .route("/shc.js", get(script))
        .route("/count/increment", post(increment_count))
        .route("/count", post(get_count))
        .layer(ClientIpSource::ConnectInfo.into_extension())
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(AppState { db });

    let port = &env::var("PORT").map_or("3000".into(), |f| f);
    let addr = format!("127.0.0.1:{}", port);
    log::info!("starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok(axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?)
}

async fn initialize(db: &Connection) -> Result<(), tokio_rusqlite::Error> {
    db.call(|conn| {
        conn.execute_batch(
            "create table if not exists counts (
    user text not null, -- from headers: ip + user-agent
    url text not null,
    count integer not null default 0,
    timestamp datetime not null default current_timestamp
);

create unique index if not exists idx_headers_url on counts (user, url);",
        )
    })
    .await
}

async fn increment_count(
    headers: HeaderMap,
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> StatusCode {
    log::info!("increment_count {:?}", headers.get("host"));

    let Some(agent) = headers.get("user-agent").and_then(|s| s.to_str().ok()) else {
        log::error!("invalid user-agent");
        return StatusCode::BAD_REQUEST;
    };
    let user = format!("{:?}::{}", ip, agent);

    let url = body["url"].to_string();
    if url.is_empty() {
        log::error!("no url found");
        return StatusCode::BAD_REQUEST;
    }

    match state
        .db
        .call(move |conn| {
            conn.execute(
                "insert into counts (user, url, count) values (?, ?, ?)
on conflict do nothing;",
                params![user, url, 1],
            )
        })
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            log::error!("could not increment due to {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    StatusCode::OK
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum Resp {
    Data { count: i32, clicked: bool },
    Error(String),
}

async fn get_count(
    headers: HeaderMap,
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Json<Resp> {
    log::info!("get_count {:?}", headers.get("host"));

    let Some(agent) = headers.get("user-agent").and_then(|s| s.to_str().ok()) else {
        log::error!("invalid user-agent");
        return Json(Resp::Error("could not find count".into()));
    };
    let user = format!("{:?}::{}", ip, agent);

    let url = body["url"].to_string();
    if url.is_empty() {
        log::error!("no url found");
        return Json(Resp::Error("could not find url".into()));
    }

    match state
        .db
        .call(move |conn| {
            conn.query_one(
                "select count(*) as count,
(select count(*) from counts where user = (?)) as clicked
from counts where url = (?);",
                params![user, url],
                |row| {
                    print!("{:?}", row);
                    let count = row.get(0).unwrap_or(0);
                    let clicked = row.get(1).unwrap_or(0);
                    Ok((count, clicked == 1))
                },
            )
        })
        .await
    {
        Ok((count, clicked)) => Json(Resp::Data { clicked, count }),
        Err(e) => {
            if e.to_string() == "Query returned no rows" {
                return Json(Resp::Data {
                    clicked: false,
                    count: 0,
                });
            } else {
                log::error!("could not return count due to {}", e);
                return Json(Resp::Error("could not find count".into()));
            }
        }
    }
}
