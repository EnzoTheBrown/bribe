use actix_web::{post, web, Error, HttpResponse};
use diesel::prelude::*;
use diesel::result::Error as DieselError;

use crate::api::utils::verify_password;
use crate::model::User;
use crate::schema::user::dsl::{email as email_col, user};
use crate::DbPool;

#[derive(serde::Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

async fn get_user_by_email(pool: &DbPool, email: &str) -> Result<Option<User>, DieselError> {
    use crate::schema::user::dsl::*;

    let conn = pool.get()?;
    let result = user
        .filter(email_col.eq(email))
        .first::<User>(&conn)
        .optional()?;
    Ok(result)
}

#[post("/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    payload: web::Json<LoginPayload>,
) -> Result<HttpResponse, Error> {
    let password = payload.password.clone();
    let user = get_user_by_email(&pool, &payload.email)
        .await
        .map_err(|e| {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().finish()
        })?;
}
