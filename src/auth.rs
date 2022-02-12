use crate::error::APIError;
use chrono::Utc;
use jsonwebtoken::errors::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::request::{FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};

pub const COOKIE_SESSION_TOKEN: &str = "session_token";

pub struct AdminUser;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = APIError;
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let jwt = match req.cookies().get(COOKIE_SESSION_TOKEN) {
            Some(jwt) => jwt.value(),
            None => return Outcome::Forward(()),
        };

        match decode::<Claims>(
            jwt,
            &DecodingKey::from_secret(jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS512),
        ) {
            Ok(decoded_jwt) => {
                if decoded_jwt.claims.role == Role::Admin {
                    Outcome::Success(AdminUser)
                } else {
                    // Not enough permission
                    Outcome::Forward(())
                }
            }
            Err(_) => Outcome::Forward(()),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Claims {
    role: Role,
    exp: usize,
}

#[derive(Eq, PartialEq, Serialize, Deserialize)]
pub enum Role {
    Admin,
    Default,
}

#[derive(Deserialize)]
pub struct LoginInfo<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

pub fn create_jwt(role: Role) -> Result<String> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::minutes(90))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        role,
        exp: expiration as usize,
    };

    let header = Header::new(Algorithm::HS512);

    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
}
