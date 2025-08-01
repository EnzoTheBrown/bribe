use crate::api::token::generate_token;
use crate::api::utils::verify_password;
use crate::model::User;
use crate::DbPool;
use actix_web::{post, web, HttpResponse};
use diesel::prelude::*;
use diesel::result::Error as DieselError;

#[derive(serde::Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
}

pub async fn get_user_by_email(pool: &DbPool, email_address: &str) -> Result<User, DieselError> {
    let user = User::get_by_field(pool, "email", email_address).await?;

    Ok(user)
}

#[post("/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    payload: web::Json<LoginPayload>,
) -> actix_web::Result<HttpResponse> {
    let password = payload.password.clone();
    let user = get_user_by_email(&pool, &payload.email)
        .await
        .map_err(|e| {
            log::error!("Database error: {e}");
            actix_web::error::ErrorInternalServerError("db failure")
        })?;
    let user_id = match user.id {
        Some(id) => id,
        None => return Ok(HttpResponse::Unauthorized().finish()),
    };
    let pw_ok = verify_password(&password, &user.hashed_password).map_err(|e| {
        log::error!("Password verification error: {e}");
        actix_web::error::ErrorInternalServerError("hash failure")
    })?;

    if !pw_ok {
        return Ok(HttpResponse::Unauthorized().finish());
    }
    let token = generate_token(user_id, &user.email).map_err(|e| {
        log::error!("Token generation error: {e}");
        actix_web::error::ErrorInternalServerError("token failure")
    })?;

    let resp = LoginResponse {
        access_token: token,
        token_type: "Bearer".into(),
    };
    Ok(HttpResponse::Ok().json(resp))
}
