use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use axum_client_ip::ClientIp;
use serde::Serialize;
use tokio_rusqlite::{Connection, params};

#[derive(Clone)]
pub struct AppState {
    pub db: Connection,
}

pub async fn initialize(db: &Connection) -> Result<(), tokio_rusqlite::Error> {
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

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Resp {
    Data { count: i32, clicked: bool },
    Error(String),
}

pub async fn get_count(
    headers: HeaderMap,
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Json<Resp> {
    log::info!("get_count {:?}", body["url"]);

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
                Json(Resp::Data {
                    clicked: false,
                    count: 0,
                })
            } else {
                log::error!("could not return count due to {}", e);
                Json(Resp::Error("could not find count".into()))
            }
        }
    }
}

pub async fn increment_count(
    headers: HeaderMap,
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> StatusCode {
    log::info!("increment_count {:?}", body["url"]);

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
                "insert into counts (user, url, count) values (?, ?, 1)
on conflict do nothing;",
                params![user, url],
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

pub async fn decrement_count(
    headers: HeaderMap,
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> StatusCode {
    log::info!("decrement_count {:?}", body["url"]);

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
                "update counts set count = count - 1 where user = (?) and url = (?);",
                params![user, url],
            )
        })
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            log::error!("could not decrement due to {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    StatusCode::OK
}
