use crate::schema::{event, message, user};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Serialize, Deserialize, Selectable)]
#[diesel(table_name = user)]
pub struct User {
    pub id: Option<i32>,
    pub full_name: String,
    pub birth_date: chrono::NaiveDate,
    pub email: String,
    pub hashed_password: String,
    pub lang: String,
}

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = user)]
pub struct NewUser {
    pub full_name: String,
    pub birth_date: chrono::NaiveDate,
    pub email: String,
    pub hashed_password: String,
    pub lang: String,
}

#[derive(Debug, Queryable, Serialize, Deserialize, Selectable)]
#[diesel(table_name = event)]
pub struct Event {
    pub id: i32,
    pub date: chrono::NaiveDate,
    pub description: String,
    pub person_id: i32,
}

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = event)]
pub struct NewEvent {
    pub date: chrono::NaiveDate,
    pub description: String,
    pub person_id: i32,
}

#[derive(Debug, Queryable, Serialize, Deserialize, Selectable)]
#[diesel(table_name = message)]
pub struct Message {
    pub id: i32,
    pub event_id: i32,
    pub source: String,
    pub content: String,
}

#[derive(Debug, Queryable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = message)]
pub struct NewMessage {
    pub event_id: i32,
    pub source: String,
    pub content: String,
}
