use std::convert::TryFrom;
use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

const TOKEN_ISSUER_ID: &str = "http://yu.xaxxi.net";
const TOKEN_AUDIENCE: &str = "http://yu.xaxxi.net";
const TOKEN_SECRET_KEY: &[u8] = b"TODO";
const TOKEN_AVAILABLE_SECONDS: u64 = 3600 * 24 * 31;

#[derive(Clone, PartialEq, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Authentication {
    Guest { guestid: String },
    Signin { userid: String }
}

impl Authentication {
    pub fn guest_new() -> Authentication {
        Authentication::Guest { guestid: guestid_unique() }
    }
}

type TokenText = String;

impl From<Authentication> for TokenText {
    fn from(item: Authentication) -> Self {
        let enckey = jsonwebtoken::EncodingKey::from_secret(TOKEN_SECRET_KEY);
        let header = jsonwebtoken::Header::default();
        let data = TokenData::new(match item {
            Authentication::Guest { ref guestid } => Some(guestid),
            Authentication::Signin { ref userid } => Some(userid)
        });
        jsonwebtoken::encode(&header, &data, &enckey).unwrap()
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum AuthType {
    Guest,
    Signin
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenData {
    // Standard claims
    iss: String, // Issuer
    sub: String, // Subject
    aud: String, // Audience
    iat: u64,    // Issued At
    nbf: u64,    // Not Before
    exp: u64,    // Expiration Time
    jti: String, // JWT ID

    // Application claims
    auth_type: AuthType
}

impl TokenData {
    fn new(userid: Option<&str>) -> Self {
        let epoch = now();
        TokenData {
            iss: TOKEN_ISSUER_ID.to_string(),
            sub: userid.unwrap_or(&guestid_unique()).to_string(),
            aud: TOKEN_AUDIENCE.to_string(),
            iat: epoch,
            nbf: epoch,
            exp: epoch + TOKEN_AVAILABLE_SECONDS,
            jti: jwtid_unique(),
            auth_type: AuthType::Guest
        }
    }

    pub fn get_id(self: &Self) -> &str {
        &self.sub
    }

    fn is_available(self: &Self) -> bool {
        let epoch = now();
        // TODO Check blacklist
        self.nbf < epoch && epoch <= self.exp
    }

    pub fn is_guest(self: &Self) -> bool {
        self.auth_type == AuthType::Guest
    }
}

// TODO Use AsRef<str>
impl TryFrom<&str> for TokenData
{
    type Error = String;

    fn try_from(item: &str) -> Result<Self, Self::Error> {
        let deckey = jsonwebtoken::DecodingKey::from_secret(TOKEN_SECRET_KEY);
        let alg = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);

        let token =
            jsonwebtoken::decode::<TokenData>(&item, &deckey, &alg)
            .map_err(|e| format!("Token invalid format, e: {}", &e)
                     .to_string())?;
        let token = token.claims;
        if !token.is_available() {
            Err("Token unavailable".to_string())?;
        }
        Ok(token)
    }
}

impl From<TokenData> for Authentication {
    fn from(item: TokenData) -> Self {
        if item.is_guest() {
            Authentication::Guest { guestid: item.get_id().to_string() }
        } else {
            Authentication::Signin { userid: item.get_id().to_string() }
        }
    }
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_else(|e| {
            warn!("Failed to calulate epoch time!? {}", &e); 0
        })
}

fn guestid_unique() -> String {
    format!("g:{}", Uuid::new_v4().to_simple().to_string())
}

fn jwtid_unique() -> String {
    Uuid::new_v4().to_hyphenated().to_string()
}

pub fn make_auth(token: Option<&str>) -> Result<Authentication, String> {
    let deckey = jsonwebtoken::DecodingKey::from_secret(TOKEN_SECRET_KEY);
    let alg = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    match token {
        Some(jwt) => {
            let token =
                jsonwebtoken::decode::<TokenData>(&jwt, &deckey, &alg)
                .map_err(|e| format!("Token invalid format, e: {}", &e)
                         .to_string())?;
            let token = token.claims;
            if !token.is_available() {
                Err("Token unavailable".to_string())?;
            }
            if token.is_guest() {
                Ok(Authentication::Guest {
                    guestid: token.get_id().to_string() })
            } else {
                Ok(Authentication::Signin {
                    userid: token.get_id().to_string() })
            }
        },
        None => Ok(Authentication::guest_new())
    }
}

