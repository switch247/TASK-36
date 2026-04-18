use anyhow::{anyhow, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use sqlx::MySqlPool;
use tracing::info;
use uuid::Uuid;

use app_core::auth::validate_password_policy;
use app_core::errors::CoreError;
use app_core::session::{calculate_session_expiry, lockout_until, LOCKOUT_ATTEMPTS};
use app_core::types::{ApiActor, Claims};
use app_models::entities::User;

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
}

#[derive(Clone)]
pub struct AuthService {
    pub pool: MySqlPool,
    pub config: AuthConfig,
}

pub struct AuthTokens {
    pub session_id: Uuid,
    pub session_expires_at: chrono::DateTime<Utc>,
    pub jwt: String,
    pub jwt_expires_at: chrono::DateTime<Utc>,
}

impl AuthService {
    pub fn new(pool: MySqlPool, jwt_secret: String) -> Self {
        Self {
            pool,
            config: AuthConfig { jwt_secret },
        }
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        validate_password_policy(password)?;
        hash(password, DEFAULT_COST).map_err(|e| anyhow!(e))
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Result<AuthTokens> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, username, password_hash, role, failed_login_attempts, lockout_until, created_at
               FROM users WHERE username = ?"#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!(CoreError::AuthenticationFailed))?;

        if let Some(lock_time) = user.lockout_until {
            if lock_time > Utc::now() {
                info!("authentication denied due to active lockout");
                return Err(anyhow!(CoreError::AccountLocked(lock_time.to_rfc3339())));
            }
        }

        let is_valid = verify(password, &user.password_hash).map_err(|e| anyhow!(e))?;
        if !is_valid {
            let attempts = user.failed_login_attempts + 1;
            if attempts >= LOCKOUT_ATTEMPTS {
                let until = lockout_until(Utc::now());
                sqlx::query(
                    "UPDATE users SET failed_login_attempts = ?, lockout_until = ? WHERE id = ?",
                )
                .bind(attempts)
                .bind(until.naive_utc())
                .bind(user.id.to_string())
                .execute(&self.pool)
                .await?;
            } else {
                sqlx::query("UPDATE users SET failed_login_attempts = ? WHERE id = ?")
                    .bind(attempts)
                    .bind(user.id.to_string())
                    .execute(&self.pool)
                    .await?;
            }
            info!("authentication failed");
            return Err(anyhow!(CoreError::AuthenticationFailed));
        }

        sqlx::query(
            "UPDATE users SET failed_login_attempts = 0, lockout_until = NULL WHERE id = ?",
        )
        .bind(user.id.to_string())
        .execute(&self.pool)
        .await?;

        info!("authentication succeeded");
        self.issue_tokens(&user).await
    }

    pub async fn validate_actor(&self, bearer_token: &str, session_id: &str) -> Result<ApiActor> {
        let claims = self.validate_jwt(bearer_token)?;

        let user_id = claims.sub.clone();
        let role = claims.role;

        let session = sqlx::query_as::<_, (String, String, chrono::NaiveDateTime, chrono::NaiveDateTime)>(
            "SELECT id, user_id, last_activity, expires_at FROM user_sessions WHERE id = ? AND user_id = ?",
        )
        .bind(session_id)
        .bind(&user_id)
        .fetch_optional(&self.pool)
        .await?;

        let (_, _, _, expires_at) = session.ok_or_else(|| anyhow!(CoreError::SessionInvalid))?;
        if expires_at <= Utc::now().naive_utc() {
            return Err(anyhow!(CoreError::SessionInvalid));
        }

        let now = Utc::now();
        let new_expiry = calculate_session_expiry(now);
        sqlx::query("UPDATE user_sessions SET last_activity = ?, expires_at = ? WHERE id = ?")
            .bind(now.naive_utc())
            .bind(new_expiry.naive_utc())
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        Ok(ApiActor {
            user_id: Some(user_id),
            role,
            username: None,
        })
    }

    pub fn validate_actor_jwt_only(&self, bearer_token: &str) -> Result<ApiActor> {
        let claims = self.validate_jwt(bearer_token)?;
        Ok(ApiActor {
            user_id: Some(claims.sub),
            role: claims.role,
            username: None,
        })
    }

    pub async fn validate_actor_session_only(&self, session_id: &str) -> Result<ApiActor> {
        let session = sqlx::query_as::<
            _,
            (
                String,
                String,
                chrono::NaiveDateTime,
                chrono::NaiveDateTime,
                String,
            ),
        >(
            r#"SELECT s.id, s.user_id, s.last_activity, s.expires_at, u.role
               FROM user_sessions s
               JOIN users u ON u.id = s.user_id
               WHERE s.id = ?"#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some((_, user_id, _, expires_at, role_raw)) = session else {
            return Err(anyhow!(CoreError::SessionInvalid));
        };
        if expires_at <= Utc::now().naive_utc() {
            return Err(anyhow!(CoreError::SessionInvalid));
        }

        let now = Utc::now();
        let new_expiry = calculate_session_expiry(now);
        sqlx::query("UPDATE user_sessions SET last_activity = ?, expires_at = ? WHERE id = ?")
            .bind(now.naive_utc())
            .bind(new_expiry.naive_utc())
            .bind(session_id)
            .execute(&self.pool)
            .await?;

        let role = match role_raw.as_str() {
            "Admin" => app_core::types::UserRole::Admin,
            "Coordinator" => app_core::types::UserRole::Coordinator,
            "Proctor" => app_core::types::UserRole::Proctor,
            _ => app_core::types::UserRole::Auditor,
        };
        Ok(ApiActor {
            user_id: Some(user_id),
            role,
            username: None,
        })
    }

    fn validate_jwt(&self, bearer_token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            bearer_token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| anyhow!(CoreError::InvalidToken))?;

        Ok(token_data.claims)
    }

    async fn issue_tokens(&self, user: &User) -> Result<AuthTokens> {
        let now = Utc::now();
        let session_expires_at = calculate_session_expiry(now);
        let jwt_expires_at = now + Duration::hours(8);

        let claims = Claims {
            sub: user.id.to_string(),
            role: user.parsed_role(),
            iat: now.timestamp() as usize,
            exp: jwt_expires_at.timestamp() as usize,
        };

        let jwt = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|_| anyhow!(CoreError::TokenGenerationFailed))?;

        let session_id = Uuid::new_v4();
        sqlx::query("INSERT INTO user_sessions (id, user_id, last_activity, expires_at) VALUES (?, ?, ?, ?)")
            .bind(session_id.to_string())
            .bind(user.id.to_string())
            .bind(now.naive_utc())
            .bind(session_expires_at.naive_utc())
            .execute(&self.pool)
            .await?;

        Ok(AuthTokens {
            session_id,
            session_expires_at,
            jwt,
            jwt_expires_at,
        })
    }

    pub fn generate_data_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }
}
