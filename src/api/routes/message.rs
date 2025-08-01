use crate::api::utils::hash_password;
use crate::model::{Message, NewMessage};
use crate::schema::MESSAGES::dsl::MESSAGES;
use crate::DbPool;
use actix_web::{get, post, web, HttpResponse, Responder};
use argon2::password_hash::Error as ArgonError;
use chrono::ParseError as ChronoParseError;
use diesel::insert_into;
use diesel::prelude::*;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum PwError {
    HashError(ArgonError),
    ParseError(ChronoParseError),
}

impl From<ChronoParseError> for PwError {
    fn from(e: ChronoParseError) -> Self {
        PwError::ParseError(e)
    }
}

impl From<ArgonError> for PwError {
    fn from(e: ArgonError) -> Self {
        PwError::HashError(e)
    }
}
impl fmt::Display for PwError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PwError::HashError(e) => write!(f, "password hashing failed: {}", e),
            PwError::ParseError(e) => write!(f, "invalid birth date: {}", e),
        }
    }
}

impl Error for PwError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PwError::HashError(e) => Some(e),
            PwError::ParseError(e) => Some(e),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct InputMessagePayload {
    pub source: String,
    pub content: String,
}

#[derive(serde::Deserialize)]
pub struct CreateMessagePayload {
    pub source: String,
    pub content: String,
    pub event_id: i32,
}

impl TryFrom<CreateMessagePayload> for NewMessage {
    type Error = PwError;
    fn try_from(p: CreateMessagePayload) -> Result<Self, Self::Error> {
        Ok(NewMessage {
            event_id: p.event_id,
            source: p.source,
            content: p.content,
        })
    }
}

#[get("/events/{event_id}/messages")]
pub async fn get_messages(pool: web::Data<DbPool>, event_id: web::Path<i32>) -> impl Responder {
    match Message::get_all(&pool, event_id.into_inner()).await {
        Ok(msgs) => HttpResponse::Ok().json(msgs),
        Err(e) => {
            eprintln!("Database error: {e:?}");
            HttpResponse::InternalServerError().body("Could not retrieve messages")
        }
    }
}

#[post("/events/{event_id}/messages")]
pub async fn create_message(
    pool: web::Data<DbPool>,
    payload: web::Json<InputMessagePayload>,
    event_id: web::Path<i32>,
) -> impl Responder {
    let pool = pool.clone();
    let payload = payload.into_inner();
    let event_id = event_id.into_inner();
    let payload = CreateMessagePayload {
        source: payload.source,
        content: payload.content,
        event_id,
    };
    let result = web::block(move || -> Result<Message, diesel::result::Error> {
        let mut conn = pool
            .get()
            .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
        let new_message: NewMessage = payload
            .try_into()
            .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
        insert_into(MESSAGES)
            .values(&new_message)
            .returning(Message::as_returning())
            .get_result::<Message>(&mut conn)
    })
    .await;
    match result {
        Ok(Ok(u)) => HttpResponse::Created().json(u),
        Ok(Err(e)) => {
            eprintln!("DB error: {e:?}");
            HttpResponse::InternalServerError().body("Could not create user")
        }
        Err(e) => {
            eprintln!("Blocking thread error: {e:?}");
            HttpResponse::InternalServerError().body("Could not create user")
        }
    }
}
