use actix_web::dev::Payload;
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, FromRequest, HttpRequest};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::future::{Ready, ready};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct Auth {
    pub claims: Claims,
}

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-only-secret-change-me".to_string())
}

pub fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )?;

    Ok(data.claims)
}

impl FromRequest for Auth {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = match req.headers().get("Authorization") {
            Some(h) => h,
            None => return ready(Err(ErrorUnauthorized("missing Authorization header"))),
        };

        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => return ready(Err(ErrorUnauthorized("invalid Authorization header"))),
        };

        let token = match auth_str.strip_prefix("Bearer ") {
            Some(t) if !t.is_empty() => t,
            _ => return ready(Err(ErrorUnauthorized("expected Bearer token"))),
        };

        match verify_jwt(token) {
            Ok(claims) => ready(Ok(Auth { claims })),
            Err(_) => ready(Err(ErrorUnauthorized("invalid or expired token"))),
        }
    }
}

pub fn create_jwt(
    user_id: i32,
    expiry_seconds: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    // Get the current time
    let current_time = SystemTime::now();
    // Calculate the expiration time
    let expiry_time = current_time
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        + std::time::Duration::from_secs(expiry_seconds);
    let expiry_timestamp = expiry_time.as_secs() as usize;

    // Create the claims for the token
    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiry_timestamp,
        // Add any other claims here
        // roles: vec!["user".to_string()],
    };

    // Create a new JWT header
    let header = Header::new(jsonwebtoken::Algorithm::HS256); // Using HS256 algorithm

    // Encode the claims into a JWT token
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )?;

    Ok(token)
}
