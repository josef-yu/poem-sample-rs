use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use poem::http::StatusCode;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct JwtData {
    pub username: String,
    pub permissions: Vec<String>,
    exp: i64
}

impl JwtData {
    pub fn new(username: String, permissions: Vec<String>, token_duration: Duration) -> Self {
        Self {
            username,
            permissions,
            exp: (Utc::now() + token_duration).timestamp()
        }
    }

    pub fn is_expired(&self) -> bool {
        self.exp <= Utc::now().timestamp()
    }
}

#[derive(Clone)]
pub struct Manager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    expiration: Duration
}

impl Manager {
    pub fn init(secret_key: String, expiration_hours: i64) -> Self {
        let expiration = Duration::try_hours(expiration_hours).expect("Parsing expiration hours");
        let encoding_key = EncodingKey::from_secret(secret_key.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret_key.as_bytes());

        Self {
            encoding_key,
            decoding_key,
            expiration
        }
    }

    pub fn create_token_data(&self, username: String, permissions: Vec<String>) -> JwtData {
        JwtData::new(username, permissions, self.expiration)
    }

    pub fn encode(&self, data: JwtData) -> poem::Result<String> {
        jsonwebtoken::encode(&Header::default(), &data, &self.encoding_key)
            .map_err(|_| 
                poem::Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
            )
    }

    pub fn decode(&self, token: &str) -> poem::Result<JwtData> {
        let result = jsonwebtoken::decode::<JwtData>(token, &self.decoding_key, &Validation::default())
            .map(|x| x.claims)
            .map_err(|_| poem::Error::from_status(StatusCode::UNAUTHORIZED));

        
        if let Ok(data) = result {
            if data.exp <= Utc::now().timestamp() {
                return Err(poem::Error::from_status(StatusCode::UNAUTHORIZED))
            }

            return Ok(data)
        } 

        return result
    }
}


