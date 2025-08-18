use crate::ai::agent::{
    ask_question_agent, categorize_with_agent, init_categorize_event_agent, init_conversation_agent,
};
use crate::event::EventCategory;
use crate::model::{Event, Message, NewEvent, NewMessage, User};
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
pub struct PatchEventPayload {
    pub date: String,
    pub title: Option<String>,
    pub description: String,
    pub category: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct InputEventPayload {
    pub date: String,
    pub title: Option<String>,
    pub description: String,
}

#[derive(serde::Deserialize)]
pub struct CreateEventPayload {
    pub date: String,
    pub title: Option<String>,
    pub category: Option<String>,
    pub description: String,
    pub user_id: i32,
}

impl TryFrom<CreateEventPayload> for NewEvent {
    type Error = PwError;
    fn try_from(p: CreateEventPayload) -> Result<Self, Self::Error> {
        Ok(NewEvent {
            date: p.date.parse().map_err(PwError::ParseError)?,
            description: p.description,
            user_id: p.user_id,
            title: p.title,
            category: p.category,
        })
    }
}

#[get("/events")]
pub async fn get_events(pool: web::Data<DbPool>, user: web::ReqData<User>) -> impl Responder {
    let user = user.into_inner();
    let events = Event::get_by_user(&pool, user.id.expect("User ID not found"))
        .await
        .expect("Filed to retrieve events");
    HttpResponse::Ok().json(events)
}

#[get("/events/{event_id}")]
pub async fn get_event_by_id(
    pool: web::Data<DbPool>,
    event_id: web::Path<i32>,
    user: web::ReqData<User>,
) -> impl Responder {
    let event_id = event_id.into_inner();
    let user = user.into_inner();
    let result = Event::get(&pool, user.id.expect("User ID not found"), event_id)
        .await
        .map_err(|e| {
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

async fn ask_question(
    pool: &web::Data<DbPool>,
    event_id: i32,
    event_name: &str,
    description: &str,
    date: &str,
) {
    let mut agent = init_conversation_agent();
    let question = ask_question_agent(&mut agent, event_name, description, date, Vec::new())
        .await
        .expect("Failed to ask question");
    println!("Question: {}", question);
    Message::create(
        &pool,
        NewMessage {
            event_id: event_id,
            source: "bot".to_string(),
            content: question,
        },
    )
    .await
    .expect("Failed to create message");
}

#[post("/events")]
pub async fn create_event(
    pool: web::Data<DbPool>,
    payload: web::Json<InputEventPayload>,
    user: web::ReqData<User>,
) -> impl Responder {
    let user = user.into_inner();
    let user_id = user.id.expect("User ID not found");
    let input = payload.into_inner();
    let title: String = input.title.unwrap_or_default();
    let description: String = input.description.clone();
    let date: String = input.date.clone();
    let mut agent = init_categorize_event_agent();
    let category = match categorize_with_agent(&mut agent, &title, &description, &date).await {
        Ok(cat) => cat,
        Err(e) => {
            eprintln!("Error categorizing event: {e}");
            EventCategory::Unknown
        }
    };
    let create_payload = CreateEventPayload {
        date: date.clone(),
        title: Some(title.clone()),
        category: Some(category.to_string()),
        description: description.clone(),
        user_id,
    };
    let new_event = match create_payload.try_into() {
        Ok(event) => event,
        Err(e) => {
            eprintln!("Error creating event: {e}");
            return HttpResponse::BadRequest().body("Invalid event data");
        }
    };
    let event = Event::create(&pool, new_event, user_id).await.map_err(|e| {
        eprintln!("Database error: {e}");
        HttpResponse::InternalServerError().body("Could not create event")
    });

    match event {
        Ok(ev) => {
            ask_question(
                &pool,
                ev.id.expect("Expect event ID"),
                &title,
                &description,
                &date,
            )
            .await;
            HttpResponse::Created().json(ev)
        }
        Err(e) => {
            eprintln!("Blocking thread error: {e:?}");
            HttpResponse::InternalServerError().body("Could not create event")
        }
    }
}

#[patch("/events/{event_id}")]
pub async fn update_event(
    pool: web::Data<DbPool>,
    event_id: web::Path<i32>,
    payload: web::Json<PatchEventPayload>,
    user: web::ReqData<User>,
) -> impl Responder {
    let event_id = event_id.into_inner();
    let user = user.into_inner();
    let payload = payload.into_inner();
    let payload = CreateEventPayload {
        date: payload.date,
        title: payload.title,
        category: payload.category,
        description: payload.description,
        user_id: user.id.expect("User ID not found"),
    };
    let updated_event = match payload.try_into() {
        Ok(event) => event,
        Err(e) => {
            eprintln!("Error updating event: {e}");
            return HttpResponse::BadRequest().body("Invalid event data");
        }
    };
    let result = Event::update(
        &pool,
        user.id.expect("User ID not found"),
        event_id,
        updated_event,
    )
    .await
    .map_err(|e| {
        eprintln!("Database error: {e}");
        HttpResponse::InternalServerError().body("Could not update event")
    });
    match result {
        Ok(ev) => HttpResponse::Ok().json(ev),
        Err(e) => {
            eprintln!("Blocking thread error: {e:?}");
            HttpResponse::InternalServerError().body("Could not update event")
        }
    }
}

#[delete("/events/{event_id}")]
pub async fn delete_event(
    pool: web::Data<DbPool>,
    event_id: web::Path<i32>,
    user: web::ReqData<User>,
) -> impl Responder {
    let event_id = event_id.into_inner();
    let user = user.into_inner();
    let result = Event::delete(&pool, user.id.expect("User ID not found"), event_id)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e}");
            HttpResponse::InternalServerError().body("Could not delete event")
        });
    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            eprintln!("Blocking thread error: {e:?}");
            HttpResponse::InternalServerError().body("Could not delete event")
        }
    }
}
