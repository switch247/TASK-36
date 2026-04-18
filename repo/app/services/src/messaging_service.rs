use anyhow::Result;
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(Debug, Clone, Serialize)]
pub struct MessageDraft {
    pub id: String,
    pub channel: String,
    pub recipient: String,
    pub subject: Option<String>,
    pub body: String,
}

#[derive(Clone)]
pub struct MessagingService {
    pool: MySqlPool,
}

impl MessagingService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create_message_draft(
        &self,
        channel: &str,
        recipient: &str,
        subject: Option<&str>,
        body: &str,
        created_by: &str,
    ) -> Result<MessageDraft> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO message_drafts (id, channel, recipient, subject, body, created_by) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(channel)
        .bind(recipient)
        .bind(subject)
        .bind(body)
        .bind(created_by)
        .execute(&self.pool)
        .await?;

        Ok(MessageDraft {
            id,
            channel: channel.to_string(),
            recipient: recipient.to_string(),
            subject: subject.map(|s| s.to_string()),
            body: body.to_string(),
        })
    }
}
