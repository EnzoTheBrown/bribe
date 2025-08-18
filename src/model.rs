use crate::schema::{EVENTS, MESSAGES, USERS};
use actix_web::web;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sqlite::Sqlite;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Serialize, Deserialize, Selectable, Clone)]
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
    pub date: chrono::NaiveDate,
    pub title: Option<String>,
    pub category: Option<String>,
    pub description: String,
    pub user_id: i32,
}

impl Event {
    pub async fn get(
        pool: &crate::DbPool,
        event_user_id: i32,
        event_id: i32,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::EVENTS::dsl::*;
        let conn = &mut pool.get().unwrap();
        let result = EVENTS
            .filter(id.eq(event_id).and(user_id.eq(event_user_id)))
            .first::<Self>(conn)?;
        Ok(result)
    }
    pub async fn get_by_user(
        pool: &crate::DbPool,
        event_user_id: i32,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::schema::EVENTS::dsl::*;
        let conn = &mut pool.get().unwrap();
        let result = EVENTS
            .filter(user_id.eq(event_user_id))
            .load::<Self>(conn)?;
        Ok(result)
    }
    pub async fn create(
        pool: &crate::DbPool,
        new_event: NewEvent,
        event_user_id: i32,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::EVENTS::dsl::*;
        let conn = &mut pool.get().unwrap();
        let new_event = NewEvent {
            user_id: event_user_id,
            ..new_event
        };
        let result = diesel::insert_into(EVENTS)
            .values(&new_event)
            .returning(Event::as_returning())
            .get_result(conn)?;
        Ok(result)
    }
    pub async fn delete(
        pool: &crate::DbPool,
        event_user_id: i32,
        event_id: i32,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::EVENTS::dsl::*;
        let conn = &mut pool.get().unwrap();
        let result = diesel::delete(EVENTS.filter(id.eq(event_id).and(user_id.eq(event_user_id))))
            .execute(conn)?;
        Ok(result)
    }
    pub async fn update(
        pool: &crate::DbPool,
        event_user_id: i32,
        event_id: i32,
        updated_event: NewEvent,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::EVENTS::dsl::*;
        let conn = &mut pool.get().unwrap();
        let result = diesel::update(EVENTS.filter(id.eq(event_id).and(user_id.eq(event_user_id))))
            .set((
                title.eq(updated_event.title),
                category.eq(updated_event.category),
                description.eq(updated_event.description),
                date.eq(updated_event.date),
            ))
            .returning(Event::as_returning())
            .get_result(conn)?;
        Ok(result)
    }
}

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = EVENTS)]
pub struct NewEvent {
    pub date: chrono::NaiveDate,
    pub description: String,
    pub user_id: i32,
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
                .order(MESSAGES::id.asc())
                .load::<Message>(&mut conn)
        })
        .await
        .map_err(|e| {
            log::error!("Blocking thread error: {e}");
            DieselError::NotFound
        })??;
        Ok(messages)
    }
    pub async fn create(
        pool: &crate::DbPool,
        new_message: NewMessage,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::MESSAGES::dsl::*;
        let conn = &mut pool.get().unwrap();
        let result = diesel::insert_into(MESSAGES)
            .values(&new_message)
            .returning(Message::as_returning())
            .get_result(conn)?;
        Ok(result)
    }
    pub async fn delete(
        pool: &crate::DbPool,
        message_id: i32,
    ) -> Result<usize, diesel::result::Error> {
        use crate::schema::MESSAGES::dsl::*;
        let conn = &mut pool.get().unwrap();
        let result = diesel::delete(MESSAGES.filter(id.eq(message_id))).execute(conn)?;
        Ok(result)
    }
    pub async fn update(
        pool: &crate::DbPool,
        message_id: i32,
        updated_message: NewMessage,
    ) -> Result<Self, diesel::result::Error> {
        use crate::schema::MESSAGES::dsl::*;
        let conn = &mut pool.get().unwrap();
        let result = diesel::update(MESSAGES.filter(id.eq(message_id)))
            .set((
                event_id.eq(updated_message.event_id),
                source.eq(updated_message.source),
                content.eq(updated_message.content),
            ))
            .returning(Message::as_returning())
            .get_result(conn)?;
        Ok(result)
    }
}

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = MESSAGES)]
pub struct NewMessage {
    pub event_id: i32,
    pub source: String,
    pub content: String,
}
