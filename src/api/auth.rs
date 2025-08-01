use crate::models::User;
use crate::api::token::verify_token;

pub async fn get_user(pool: web::Data<DbPool>, req: HttpRequest) -> Result<User, Box<dyn Error>> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");
    let user_id = verify_token(token)
    Ok(user)
}
