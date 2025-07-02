use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

// Header of Reqs
#[derive(Serialize, Deserialize, Debug)]
pub struct ControlBlock {
    pub jwt: String,
}

impl ControlBlock {
    /// validate if true then refresh
    pub fn validate_jwt(&mut self) -> Result<bool, jsonwebtoken::errors::Error> {
        let claims = validate_jwt(&self.jwt)?;
        if claims.exp < Utc::now().timestamp() as usize {
            Ok(false)
        } else {
            self.refresh_jwt();
            Ok(true)
        }
    }

    pub fn refresh_jwt(&mut self) -> Result<(), jsonwebtoken::errors::Error> {
        self.jwt = refresh_jwt(&self.jwt)?;
        Ok(())
    }

    pub fn from_user_name(user_name: &str) -> Self {
        Self {
            jwt: issue_jwt(user_name).unwrap(),
        }
    }
}

// JWT 声明结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_name: String,
    pub exp: usize, // 过期时间戳
}

fn get_secret_key() -> &'static [u8] {
    b"secret key demo"
}

/// 颁发 JWT
pub fn issue_jwt(user_name: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now() + Duration::hours(24);
    let claims = Claims {
        user_name: user_name.to_string(),
        exp: expiration.timestamp() as usize,
    };

    let header = Header::new(Algorithm::HS256);
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(get_secret_key()),
    )?;

    Ok(token)
}

/// 鉴权 JWT
pub fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_secret_key()),
        &validation,
    )?;

    Ok(token_data.claims)
}

/// 刷新 JWT
pub fn refresh_jwt(token: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = validate_jwt(token)?;
    let new_expiration = Utc::now() + Duration::hours(24);
    let new_claims = Claims {
        user_name: claims.user_name,
        exp: new_expiration.timestamp() as usize,
    };

    let header = Header::new(Algorithm::HS256);
    let new_token = encode(
        &header,
        &new_claims,
        &EncodingKey::from_secret(get_secret_key()),
    )?;

    Ok(new_token)
}
