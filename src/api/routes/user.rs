use crate::api::auth::get_user;
use crate::api::utils::hash_password;
use crate::model::{NewUser, User};
use crate::schema::USERS::dsl::USERS;
use crate::DbPool;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
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
pub struct CreateUserPayload {
    pub full_name: String,
    pub birth_date: String,
    pub email: String,
    pub password: String,
    pub lang: String,
}

impl TryFrom<CreateUserPayload> for NewUser {
    type Error = PwError;
    fn try_from(p: CreateUserPayload) -> Result<Self, Self::Error> {
        Ok(NewUser {
            full_name: p.full_name,
            birth_date: chrono::NaiveDate::parse_from_str(&p.birth_date, "%Y-%m-%d")?,
            email: p.email,
            hashed_password: hash_password(&p.password)?,
            lang: p.lang,
        })
    }
}

#[get("/users")]
pub async fn get_users(pool: web::Data<DbPool>, req: HttpRequest) -> impl Responder {
    get_user(pool, req).await.unwrap_or_else(|_| {
        eprintln!("Failed to retrieve user from request");
        HttpResponse::InternalServerError().body("Could not retrieve user")
    });
    let users = USERS
        .load::<User>(&mut pool.get().expect("Failed to get DB connection"))
        .expect("Error loading users");
    HttpResponse::Ok().json(users)
}

#[get("/users/{user_id}")]
pub async fn get_user_by_id(pool: web::Data<DbPool>, path: web::Path<i32>) -> impl Responder {
    let user = User::get(&pool, path.to_owned()).await.map_err(|e| {
        eprintln!("Database error: {e:?}");
        HttpResponse::InternalServerError().body("Could not retrieve user")
    });
    match user {
        Ok(u) => HttpResponse::Ok().json(u),
        Err(e) => {
            eprintln!("Blocking thread error: {e:?}");
            HttpResponse::InternalServerError().body("Could not retrieve user")
        }
    }
}

#[post("/users")]
pub async fn create_user(
    pool: web::Data<DbPool>,
    payload: web::Json<CreateUserPayload>,
) -> impl Responder {
    let pool = pool.clone();
    let payload = payload.into_inner();

    let result = web::block(move || -> Result<User, diesel::result::Error> {
        let mut conn = pool
            .get()
            .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;

        let new_user: NewUser = payload
            .try_into()
            .map_err(|e| diesel::result::Error::SerializationError(Box::new(e)))?;
        insert_into(USERS)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result::<User>(&mut conn)
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
