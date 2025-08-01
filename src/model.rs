use crate::schema::{EVENTS, MESSAGES, USERS};
use actix_web::{error::BlockingError, get, post, web, HttpResponse, Responder};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Serialize, Deserialize, Selectable)]
#[diesel(table_name = USERS)]
#[diesel(check_for_backend(Sqlite))]
pub struct User {
    pub id: Option<i32>,
    pub full_name: String,
    pub birth_date: chrono::NaiveDate,
    pub email: String,
    pub hashed_password: String,
    pub lang: String,
}

impl User {
    pub async fn get_by_field(
        pool: &crate::DbPool,
        field: &str,
        value: &str,
    ) -> Result<Self, diesel::result::Error> {
        let pool = pool.clone();
        let field = field.to_owned();
        let value = value.to_owned();
        let user = web::block(move || {
            let mut conn = pool
                .get()
                .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
            match field.as_str() {
                "email" => USERS::table
                    .filter(USERS::email.eq(&value))
                    .first::<User>(&mut conn),
                "id" => USERS::table
                    .filter(USERS::id.eq(value.parse::<i32>().ok()))
                    .first::<User>(&mut conn),
                _ => Err(diesel::result::Error::NotFound),
            }
        })
        .await
        .map_err(|e| {
            log::error!("Blocking thread error: {e}");
            DieselError::NotFound
        })??;
        Ok(user)
    }
    pub async fn get(pool: &crate::DbPool, id: i32) -> Result<Self, diesel::result::Error> {
        Self::get_by_field(pool, "id", &id.to_string()).await
    }
}

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = USERS)]
pub struct NewUser {
    pub full_name: String,
    pub birth_date: chrono::NaiveDate,
    pub email: String,
    pub hashed_password: String,
    pub lang: String,
}

#[derive(Debug, Queryable, Serialize, Deserialize, Selectable)]
#[diesel(table_name = EVENTS)]
#[diesel(check_for_backend(Sqlite))]
pub struct Event {
    pub id: Option<i32>,
    pub title: Option<String>,
    pub category: Option<String>,
    pub date: chrono::NaiveDate,
    pub description: String,
    pub person_id: i32,
}

impl Event {
    pub async fn get(pool: &crate::DbPool, id: i32) -> Result<Self, diesel::result::Error> {
        let pool = pool.clone();
        let id = id.to_owned();
        let event = web::block(move || {
            let mut conn = pool
                .get()
                .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
            EVENTS::table
                .filter(EVENTS::id.eq(id))
                .select(Event::as_select())
                .first::<Event>(&mut conn)
        })
        .await
        .map_err(|e| {
            log::error!("Blocking thread error: {e}");
            DieselError::NotFound
        })??;
        Ok(event)
    }
}

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = EVENTS)]
pub struct NewEvent {
    pub date: chrono::NaiveDate,
    pub description: String,
    pub person_id: i32,
    pub title: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Queryable, Serialize, Deserialize, Selectable)]
#[diesel(table_name = MESSAGES)]
#[diesel(check_for_backend(Sqlite))]
pub struct Message {
    pub id: Option<i32>,
    pub event_id: i32,
    pub source: String,
    pub content: String,
}

impl Message {
    pub async fn get_all(
        pool: &crate::DbPool,
        event_id: i32,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        let pool = pool.clone();
        let event_id = event_id.to_owned();
        let messages = web::block(move || {
            let mut conn = pool
                .get()
                .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
            MESSAGES::table
                .filter(MESSAGES::event_id.eq(event_id))
                .load::<Message>(&mut conn)
        })
        .await
        .map_err(|e| {
            log::error!("Blocking thread error: {e}");
            DieselError::NotFound
        })??;
        Ok(messages)
    }
}

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = MESSAGES)]
pub struct NewMessage {
    pub event_id: i32,
    pub source: String,
    pub content: String,
}
