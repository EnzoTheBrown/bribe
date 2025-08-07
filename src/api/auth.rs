use crate::api::token::verify_token;
use crate::model::User;
use crate::DbPool;
use actix_web::{
    dev::ServiceRequest,
    error::{ErrorInternalServerError, ErrorUnauthorized},
    web, Error, HttpMessage,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use std::env;

pub async fn bearer_validator(
    req: ServiceRequest,
    creds: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    println!("Validating bearer token: {}", creds.token());
    let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY should be set");
    let pool = match req.app_data::<web::Data<DbPool>>().cloned() {
        Some(p) => p,
        None => {
            println!("No database pool found in request");
            return Err((ErrorInternalServerError("Pool missing"), req));
        }
    };
    let user_id = match verify_token(creds.token(), secret_key.as_str()) {
        Ok(id) => id,
        Err(_) => {
            println!("Invalid or expired token");
            return Err((ErrorUnauthorized("Invalid or expired token"), req));
        }
    };
    let user = match User::get(&pool, user_id).await {
        Ok(u) => u,
        Err(e) => {
            println!("DB error loading user {user_id}: {e:?}");
            return Err((ErrorUnauthorized("User not found"), req));
        }
    };
    req.extensions_mut().insert::<User>(user);
    Ok(req)
}
