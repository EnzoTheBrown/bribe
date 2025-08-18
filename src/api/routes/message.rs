use crate::model::{Event, Message, NewMessage, User};
use crate::DbPool;
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use argon2::password_hash::Error as ArgonError;
use chrono::ParseError as ChronoParseError;
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
    pub event_id: i32,
    pub source: String,
    pub content: String,
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
pub async fn get_messages(
    pool: web::Data<DbPool>,
    user: web::ReqData<User>,
    event_id: web::Path<i32>,
) -> impl Responder {
    let event_id = event_id.into_inner();
    let user = user.into_inner();
    match Event::get(&pool, user.id.expect("User ID must be set"), event_id).await {
        Ok(_) => match Message::get_all(&pool, event_id).await {
            Ok(msgs) => HttpResponse::Ok().json(msgs),
            Err(e) => {
                eprintln!("Database error: {e:?}");
                HttpResponse::NotFound().body("Could not retrieve messages")
            }
        },
        Err(e) => {
            eprintln!("DB error loading event {}: {e:?}", event_id);
            return HttpResponse::NotFound().body("Event not found");
        }
    }
}

#[post("/events/{event_id}/messages")]
pub async fn create_message(
    pool: web::Data<DbPool>,
    payload: web::Json<InputMessagePayload>,
    user: web::ReqData<User>,
    event_id: web::Path<i32>,
) -> impl Responder {
    let event_id = event_id.into_inner();
    let user = user.into_inner();
    let payload = payload.into_inner();
    let payload = CreateMessagePayload {
        event_id,
        source: payload.source,
        content: payload.content,
    };
    match Event::get(&pool, user.id.expect("User ID must be set"), event_id).await {
        Ok(_) => {
            let new_message = match payload.try_into() {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("Error creating message: {e:?}");
                    return HttpResponse::BadRequest().body("Invalid message data");
                }
            };
            match Message::create(&pool, new_message).await {
                Ok(msg) => HttpResponse::Created().json(msg),
                Err(e) => {
                    eprintln!("Database error: {e:?}");
                    HttpResponse::NotFound().body("Could not create message")
                }
            }
        }
        Err(e) => {
            eprintln!("DB error loading event {}: {e:?}", event_id);
            HttpResponse::NotFound().body("Event not found")
        }
    }
}

#[patch("/events/{event_id}/messages/{message_id}")]
pub async fn update_message(
    pool: web::Data<DbPool>,
    payload: web::Json<InputMessagePayload>,
    user: web::ReqData<User>,
    param: web::Path<(i32, i32)>,
) -> impl Responder {
    let (event_id, message_id) = param.into_inner();
    let user = user.into_inner();
    let payload = payload.into_inner();
    match Event::get(&pool, user.id.expect("User ID must be set"), event_id).await {
        Ok(_) => {
            let updated_message = NewMessage {
                event_id,
                source: payload.source,
                content: payload.content,
            };
            match Message::update(&pool, message_id, updated_message).await {
                Ok(msg) => HttpResponse::Ok().json(msg),
                Err(e) => {
                    eprintln!("Database error: {e:?}");
                    HttpResponse::InternalServerError().body("Could not update message")
                }
            }
        }
        Err(e) => {
            eprintln!("DB error loading event {}: {e:?}", event_id);
            HttpResponse::NotFound().body("Event not found")
        }
    }
}

#[delete("/events/{event_id}/messages/{message_id}")]
pub async fn delete_message(
    pool: web::Data<DbPool>,
    user: web::ReqData<User>,
    param: web::Path<(i32, i32)>,
) -> impl Responder {
    let (event_id, message_id) = param.into_inner();
    let user = user.into_inner();
    match Event::get(&pool, user.id.expect("User ID must be set"), event_id).await {
        Ok(_) => match Message::delete(&pool, message_id).await {
            Ok(_) => HttpResponse::NoContent().finish(),
            Err(e) => {
                eprintln!("Database error: {e:?}");
                HttpResponse::InternalServerError().body("Could not delete message")
            }
        },
        Err(e) => {
            eprintln!("DB error loading event {}: {e:?}", event_id);
            HttpResponse::NotFound().body("Event not found")
        }
    }
}
