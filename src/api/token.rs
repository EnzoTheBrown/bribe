use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i32,
    email: String,
    exp: usize,
}

pub fn generate_token(
    user_id: i32,
    email: &str,
    secret_key: &str,
) -> Result<String, Box<dyn Error>> {
    let expiration = (Utc::now() + Duration::days(7)).timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        email: email.to_owned(),
        exp: expiration,
    };

    let header = Header::new(Algorithm::HS256);

    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret_key.as_bytes()),
    )?;

    Ok(token)
}

pub fn verify_token(token: &str, secret_key: &str) -> Result<i32, Box<dyn Error>> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 60;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims.sub)
}
