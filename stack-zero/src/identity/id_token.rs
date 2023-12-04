use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use jsonwebtoken::{self as jwt, jwk::JwkSet};
use jwt::{DecodingKey, Validation};
use serde::Deserialize;
use url::Url;

/// A validated Id Token.
#[derive(Debug)]
pub struct IdToken {
    pub claims: Claims,
}

/// The id token claims expected.
///
/// Depending on the scopes, the following claims can be expected:
/// <https://auth0.com/docs/get-started/apis/scopes/openid-connect-scopes#standard-claims>
///
/// For a complete list of standard claims:
/// <https://openid.net/specs/openid-connect-core-1_0.html#StandardClaims>
#[derive(Debug, Deserialize)]
pub struct Claims {
    #[serde(flatten)]
    pub profile: Profile,
    #[serde(flatten)]
    pub email: Email,
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    /// This might be an email.
    pub name: String,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
    pub nickname: Option<String>,
    // This URL MUST refer to an image file (for example, a PNG, JPEG, or GIF image file)
    pub picture: Option<Url>,
    // TODO: Why is this an actual Date string?
    // From: Standard Claims:
    // > Its value is a JSON number representing the number of seconds from 1970-01-01T0:0:0Z as measured in UTC until the date/time.
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct Email {
    // TODO: verify: RFC 5322 [RFC5322] addr-spec syntax
    pub email: String,
    pub email_verified: bool,
}

impl IdToken {
    pub fn validate(
        issuer: &str,
        audience: &str,
        jwk_set: &JwkSet,
        token: &str,
    ) -> Result<IdToken> {
        let header = jwt::decode_header(token)?;
        let Some(kid) = header.kid else {
            bail!("Expected kid");
        };
        let Some(jwk) = jwk_set.find(&kid) else {
            bail!("kid not found in Jwk Set");
        };
        let key = DecodingKey::from_jwk(jwk)?;

        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[issuer]);
        validation.set_audience(&[audience]);

        let token_data = jwt::decode::<Claims>(token, &key, &validation)?;
        Ok(IdToken {
            claims: token_data.claims,
        })
    }
}

// Example Header and IdToken TODO: Create a test case.

//     {
//     "alg": "RS256",
//     "typ": "JWT",
//     "kid": "im_P3_afNRBtnZtDmcrqA"
//   }

// {
//     "nickname": "armin",
//     "name": "armin@replicator.org",
//     "picture": "https://s.gravatar.com/avatar/ab20e5126cf31ef02c32e250d1037663?s=480&r=pg&d=https%3A%2F%2Fcdn.auth0.com%2Favatars%2Far.png",
//     "updated_at": "2023-12-01T09:59:57.720Z",
//     "email": "armin@replicator.org",
//     "email_verified": false,
//     "iss": "https://stack-zero.eu.auth0.com/",
//     "aud": "WvPW3q4XLzxWyzLkRDLZn6mnF3ucbuMv",
//     "iat": 1701424798,
//     "exp": 1701460798,
//     "sub": "auth0|6567904611cb1aa37c0a2ba2",
//     "sid": "nhlK01VHDV2-vYIO75vCS31ltmZz_FDc"
//   }
