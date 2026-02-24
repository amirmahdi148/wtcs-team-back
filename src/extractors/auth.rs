use actix_web::dev::Payload;
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, FromRequest, HttpRequest};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::future::{Ready, ready};

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
