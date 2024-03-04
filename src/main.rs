use ::core::{future::Future, marker::Send, pin::Pin};
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::{Json, Router};
use axum::routing::{get, post};
use axum::http::{request::Parts, StatusCode};
use axum::extract::{FromRequestParts, Path, Query, State};
use serde::Deserialize;
use sqlite::SqliteDb;
use util::{Answer, Event, NewHttpEvent, Ordinal};
use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;

pub mod sqlite;
pub mod util;

struct AppState {
    db: SqliteDb
}

type Shared = Arc<Mutex<AppState>>;

#[tokio::main]
async fn main() {
    let db = SqliteDb::new("sqlite.db".to_string()).unwrap();

    let shared_state = Arc::new(Mutex::new( AppState { db }));

    let auth_router = Router::new()
        .route("/events", get(events))
        .route("/event", post(event))
        .route("/query", post(event))
        .route("/event/:ord/reply", post(reply));

    let app = Router::new()
        .route("/token", get(token))
        .route("/checkToken", get(verify_handler))
        .nest("/", auth_router)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub struct Auth(pub Claim);

impl<S> FromRequestParts<S> for Auth {
    type Rejection = StatusCode;

    #[must_use]
    #[allow(clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn from_request_parts<'parts,'s,'tr>(parts: &'parts mut Parts,_: &'s S) ->
     Pin<Box<dyn Future<Output = Result<Self,Self::Rejection> > + Send+'tr> >where 'parts:'tr,'s:'tr,Self:'tr {
        Box::pin(extract_auth(parts))
    }
}

async fn extract_auth(parts: &mut Parts) -> Result<Auth, StatusCode> {
    let auth = parts.headers.get("Authorization").ok_or(StatusCode::UNAUTHORIZED)?;
    let str = auth.to_str().unwrap();
    let claim = verify(str.to_owned()).ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Auth(claim))
}

async fn event(State(state): State<Shared>, Auth(auth): Auth, Json(event): Json<NewHttpEvent>) -> Json<Ordinal> {
    let lock = state.lock().unwrap();

    let ordinal = lock.db.insert(event.name(auth.name)).unwrap();
    
    Json(ordinal)
}

#[derive(Deserialize)]
struct EventsParams {
    extra: Option<bool>,
}

async fn events(State(state): State<Shared>, auth: Auth, Query(params): Query<EventsParams>) -> Json<Vec<Event>> {
    let lock = state.lock().unwrap();

    let values = lock.db.read_from(Ordinal(0), params.extra.unwrap_or(false)).unwrap();

    println!("{}", auth.0.name);
    
    Json(values)
}

async fn reply(State(state): State<Shared>, Auth(auth): Auth, Path(ord): Path<Ordinal>, body: Bytes) -> String {
    let lock = state.lock().unwrap();

    let answer = Answer {
        name: auth.name,
        data: body.into(),
    };

    let vec = serde_json::to_vec(&answer).unwrap();

    let success = lock.db.answer(ord, vec).is_ok();

    success.to_string()
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Claim {
    pub name: String
}

#[derive(serde::Deserialize)]
pub struct TokenParams {
    name: String
}

async fn token(Query(params): Query<TokenParams>) -> String {
    sign( Claim { name: params.name })
}

#[derive(serde::Deserialize)]
pub struct VerifyParams {
    token: String
}

async fn verify_handler(Query(params): Query<VerifyParams>) -> String {
    verify(params.token).unwrap().name
}

fn sign(claim: Claim) -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice(b"super secure key").unwrap();
    claim.sign_with_key(&key).unwrap()
}

fn verify(token: String) -> Option<Claim> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(b"super secure key").unwrap();
    token.verify_with_key(&key).ok()
}