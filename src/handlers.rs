use axum::{Json, extract::State, http::HeaderMap};
use axum_client_ip::ClientIp;
use serde::Serialize;
use tokio_rusqlite::{Connection, params};

#[derive(Clone)]
pub struct AppState {
    pub db: Connection,
    pub salt: String,
}

pub async fn initialize(db: &Connection) -> Result<String, tokio_rusqlite::Error> {
    db.call(|conn| {
        conn.execute_batch(
            "
create table if not exists counts (
    user text not null, -- from headers: ip + user-agent
    url text not null, -- from window.location in shc.js
    updated_at datetime not null default current_timestamp
); -- each row is a 'count' for that url

create unique index if not exists idx_user_url on counts (user, url);

create table if not exists meta (key text primary key, value text);
insert into meta (key, value) values ('salt', hex(randomblob(32))) on conflict do nothing;",
        )
    })
    .await?;

    let salt = db
        .call(|conn| {
            conn.query_row("select value from meta where key = 'salt'", [], |r| {
                r.get::<_, String>(0)
            })
        })
        .await?;

    Ok(salt)
}

#[derive(Serialize, Default)]
pub struct Resp {
    count: i32,
    clicked: bool,
    error: Option<String>,
}

pub async fn count(
    headers: HeaderMap,
    ClientIp(ip): ClientIp,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Json<Resp> {
    log::info!("count {:?} {:?}", body["url"], body["delta"]);

    let Some(agent) = headers.get("user-agent").and_then(|s| s.to_str().ok()) else {
        log::error!("invalid user-agent");
        return Json(Resp {
            error: Some("could not find count".into()),
            ..Resp::default()
        });
    };
    let user = blake3::hash(format!("{}\0{}\0{}", state.salt, ip, agent).as_bytes()).to_string();
    let url = body["url"].to_string(); // TODO: as_str
    if url.is_empty() {
        log::error!("no url found");
        return Json(Resp {
            error: Some("could not find url".into()),
            ..Resp::default()
        });
    }
    let delta = body["delta"].as_i64();

    match state
        .db
        .call(move |conn| {
            match delta {
                Some(1) => {
                    // Increment
                    conn.execute(
                        "insert into counts (user, url) values (?1, ?2) on conflict do nothing;",
                        params![&user, &url],
                    )?;
                }
                Some(-1) => {
                    // Decrement
                    conn.execute(
                        "delete from counts where user = (?) and url = (?);",
                        params![&user, &url],
                    )?;
                }
                _ => {}
            };

            conn.query_one(
                "select count(*) as count,
(select count(*) from counts where user = (?) and url = (?)) as clicked
from counts where url = (?);",
                params![user, url, url],
                |row| {
                    let count = row.get(0).unwrap_or(0);
                    let clicked = row.get(1).unwrap_or(0);
                    Ok((count, clicked > 0))
                },
            )
        })
        .await
    {
        Ok((count, clicked)) => Json(Resp {
            clicked,
            count,
            error: None,
        }),

        Err(e) => {
            log::error!("could not return count due to {}", e);
            Json(Resp {
                error: Some("count not return count".into()),
                ..Resp::default()
            })
        }
    }
}
