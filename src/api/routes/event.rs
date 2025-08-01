use crate::model::{Event, NewEvent};
use crate::schema::EVENTS::dsl::EVENTS;
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
pub struct CreateEventPayload {
    pub date: String,
    pub title: Option<String>,
    pub category: Option<String>,
    pub description: String,
    pub person_id: i32,
}

impl TryFrom<CreateEventPayload> for NewEvent {
    type Error = PwError;
    fn try_from(p: CreateEventPayload) -> Result<Self, Self::Error> {
        Ok(NewEvent {
            date: p.date.parse().map_err(PwError::ParseError)?,
            description: p.description,
            person_id: p.person_id,
            title: p.title,
            category: p.category,
        })
    }
}

#[get("/events")]
pub async fn get_events(pool: web::Data<DbPool>) -> impl Responder {
    let events = EVENTS
        .select(Event::as_select())
        .load::<Event>(&mut pool.get().expect("Failed to get DB connection"))
        .expect("Error loading users");
    HttpResponse::Ok().json(events)
}

#[get("/events/{event_id}")]
pub async fn get_event_by_id(pool: web::Data<DbPool>, event_id: web::Path<i32>) -> impl Responder {
    let event_id = event_id.into_inner();
    let result = Event::get(&pool, event_id).await.map_err(|e| {
        eprintln!("Database error: {e}");
        HttpResponse::InternalServerError().body("Could not retrieve event")
    });
    match result {
        Ok(ev) => HttpResponse::Ok().json(ev),
        Err(e) => {
            eprintln!("Blocking thread error: {e:?}");
            HttpResponse::InternalServerError().body("Could not retrieve user")
        }
    }
}

#[post("/events")]
pub async fn create_event(
    pool: web::Data<DbPool>,
    payload: web::Json<CreateEventPayload>,
) -> impl Responder {
    let pool = pool.clone();
    let payload = payload.into_inner();

    let result = web::block(move || -> Result<Event, diesel::result::Error> {
        let mut conn = pool
            .get()
            .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;

        let new_event: NewEvent = payload
            .try_into()
            .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;

        insert_into(EVENTS)
            .values(&new_event)
            .returning(Event::as_returning())
            .get_result::<Event>(&mut conn)
    })
    .await;
    match result {
        Ok(Ok(u)) => HttpResponse::Created().json(u),
        Ok(Err(e)) => {
            eprintln!("DB error: {e:?}");
            HttpResponse::InternalServerError().body("Could not create event")
        }
        Err(e) => {
            eprintln!("Blocking thread error: {e:?}");
            HttpResponse::InternalServerError().body("Could not create event")
        }
    }
}
