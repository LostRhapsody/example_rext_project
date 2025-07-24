use axum::{
  extract::Request,
  http::{StatusCode, header},
  middleware::Next,
  response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{bridge::types::auth::AuthUser, infrastructure::{app_error::AppError, jwt_claims::Claims}};

// JWT middleware
pub async fn auth_middleware(mut request: Request, next: Next) -> Result<Response, AppError> {
  let auth_header = request
      .headers()
      .get(header::AUTHORIZATION)
      .and_then(|header| header.to_str().ok());

  let auth_header = match auth_header {
      Some(header) => header,
      None => {
          return Err(AppError {
              message: "Missing Authorization header".to_string(),
              status_code: StatusCode::UNAUTHORIZED,
          });
      }
  };

  let token = match auth_header.strip_prefix("Bearer ") {
      Some(token) => token,
      None => {
          return Err(AppError {
              message: "Invalid Authorization header format".to_string(),
              status_code: StatusCode::UNAUTHORIZED,
          });
      }
  };

  let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret".to_string());
  let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());

  let token_data = match decode::<Claims>(token, &decoding_key, &Validation::default()) {
      Ok(data) => data,
      Err(_) => {
          return Err(AppError {
              message: "Invalid token".to_string(),
              status_code: StatusCode::UNAUTHORIZED,
          });
      }
  };

  // Check if token is expired
  let current_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs() as usize;

  if token_data.claims.exp < current_time {
      return Err(AppError {
          message: "Token expired".to_string(),
          status_code: StatusCode::UNAUTHORIZED,
      });
  }

  // Add user to request extensions
  request.extensions_mut().insert(AuthUser {
      user_id: uuid::Uuid::parse_str(&token_data.claims.sub).unwrap(),
  });

  Ok(next.run(request).await)
}