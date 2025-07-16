use base64::{Engine, engine::general_purpose};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use log::*;
use serde::{Deserialize, Serialize};

// Header of Reqs
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ControlBlock {
    pub jwt: String,
    pub exp: i64,
}

impl ControlBlock {

    pub fn validate_jwt(&self) -> Result<bool, jsonwebtoken::errors::Error> {
        let claims = validate_jwt(&self.jwt)?;
        if claims.exp < Utc::now().timestamp() as usize {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn refresh_jwt(&mut self) -> Result<(), jsonwebtoken::errors::Error> {
        (self.jwt, self.exp) = refresh_jwt(&self.jwt)?;
        Ok(())
    }

    pub fn from_user_name(user_name: &str) -> Self {
        let (jwt, exp) = issue_jwt(user_name).unwrap();
        Self {
            jwt,
            exp
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_name: String,
    pub exp: usize, // 过期时间戳
}

fn get_secret_key() -> &'static [u8] {
    b"secret key demo"
}

pub fn issue_jwt(user_name: &str) -> Result<(String, i64), jsonwebtoken::errors::Error> {
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

    Ok((token, expiration.timestamp()))
}

pub fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(get_secret_key()),
        &validation,
    )?;

    Ok(token_data.claims)
}

pub fn refresh_jwt(token: &str) -> Result<(String, i64), jsonwebtoken::errors::Error> {
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

    Ok((new_token, new_expiration.timestamp()))
}

pub fn parse_input<T>(payload: &str) -> Result<(ControlBlock, T), String>
where
    T: serde::de::DeserializeOwned,
{
    let mut parts: Vec<&str> = payload.split(' ').collect();

    // 无payload
    if parts.len() == 2 {
        parts.push("0");
    }

    if parts.len() != 3 {
        return Err("invalid params".to_string());
    }

    let control_block = if parts[1] != "." {
        match general_purpose::STANDARD.decode(&parts[1]) {
            Ok(block) => match String::from_utf8(block) {
                Ok(block) => block,
                Err(e) => {
                    warn!("b64 decode err {}", e);
                    return Err(format!("b64 decode err {}", e));
                }
            },
            Err(e) => {
                warn!("b64 decode err {}", e);
                return Err(format!("b64 decode err {}", e));
            }
        }
    } else {
        ".".to_string()
    };
    let payload = match general_purpose::STANDARD.decode(&parts[2]) {
        Ok(payload) => match String::from_utf8(payload) {
            Ok(payload) => payload,
            Err(e) => {
                warn!("b64 decode err {}", e);
                return Err(format!("b64 decode err {}", e));
            }
        },
        Err(e) => {
            warn!("b64 decode err {}", e);
            return Err(format!("b64 decode err {}", e));
        }
    };

    let control_block: ControlBlock = if control_block != "." {
        match serde_json::from_str(&control_block) {
            Ok(block) => block,
            Err(e) => {
                warn!("deserialize err: {}, use empty control_block", e);
                ControlBlock::default()
            }
        }
    } else {
        ControlBlock::default()
    };
    let content: T = match serde_json::from_str(&payload) {
        Ok(content) => content,
        Err(e) => {
            warn!("deserialize content err: {}", e);
            return Err(format!("deserialize content err: {}", e));
        }
    };

    Ok((control_block, content))
}
