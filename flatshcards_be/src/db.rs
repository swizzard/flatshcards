use chrono::{DateTime, Utc};
use sqlx::{
    FromRow, Row,
    postgres::{PgPool, Postgres},
};
// use rusqlite::types::Type;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub async fn create_tables_in_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS stack (
  uri TEXT PRIMARY KEY,
  author_did TEXT NOT NULL,
  back_lang VARCHAR(2),
  front_lang VARCHAR(2),
  label VARCHAR(100),
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  indexed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
)",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS stack_author_label ON stack (author_did, label)",
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS stack_author ON stack (author_did)")
        .execute(pool)
        .await?;

    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS card (
  uri TEXT PRIMARY KEY,
  author_did TEXT NOT NULL,
  back_lang VARCHAR (2) NOT NULL,
  back_text TEXT NOT NULL,
  front_lang VARCHAR (2) NOT NULL,
  front_text TEXT NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  indexed_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  stack_id TEXT REFERENCES stack(uri) ON DELETE CASCADE
)
",
    )
    .execute(pool)
    .await?;
    sqlx::query("CREATE UNIQUE INDEX IF NOT EXISTS card_langs ON card (front_lang, back_lang);")
        .execute(pool)
        .await?;
    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS auth_state (
  key TEXT PRIMARY KEY,
  state TEXT NOT NULL
)
",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS auth_session (
  key TEXT PRIMARY KEY,
  session TEXT NOT NULL
);
",
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbStack {
    pub uri: String,
    pub author_did: String,
    pub back_lang: Option<String>,
    pub front_lang: Option<String>,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub indexed_at: DateTime<Utc>,
}

impl DbStack {
    pub fn new(
        StackArgs {
            uri,
            author_did,
            back_lang,
            front_lang,
            label,
            indexed_at,
        }: StackArgs,
    ) -> Self {
        let ia = indexed_at.unwrap_or_else(Utc::now);
        Self {
            uri,
            author_did,
            back_lang,
            front_lang,
            label,
            created_at: ia,
            indexed_at: ia,
        }
    }
    pub async fn save(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        self.save_with_executor(pool).await
    }
    async fn save_with_executor<'a, Ex>(&self, executor: Ex) -> Result<(), sqlx::Error>
    where
        Ex: sqlx::Executor<'a, Database = Postgres>,
    {
        sqlx::query(
            "
      INSERT INTO stack (uri, author_did, back_lang, front_lang, label, created_at, indexed_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7);
    ",
        )
        .bind(&self.uri)
        .bind(&self.author_did)
        .bind(&self.back_lang)
        .bind(&self.front_lang)
        .bind(self.created_at)
        .bind(self.indexed_at)
        .execute(executor)
        .await?;
        Ok(())
    }
    pub async fn upsert(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        let existing = sqlx::query("SELECT uri FROM stack WHERE uri = $1")
            .bind(&self.uri)
            .fetch_optional(&mut *tx)
            .await?;
        if existing.is_some() {
            sqlx::query(
                "
    UPDATE stack SET author_did = $2, back_lang = $3, front_lang = $4, label = $5, indexed_at = $6
    WHERE uri = $1",
            )
            .bind(&self.uri)
            .bind(&self.author_did)
            .bind(&self.back_lang)
            .bind(&self.front_lang)
            .bind(self.indexed_at)
            .execute(&mut *tx)
            .await?;
        } else {
            self.save_with_executor(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn delete_by_uri(uri: &str, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM stack WHERE uri = $1")
            .bind(uri)
            .execute(pool)
            .await?;
        Ok(())
    }
    pub async fn is_owned_by(
        author_did: &str,
        stack_uri: &str,
        pool: &PgPool,
    ) -> Result<bool, sqlx::Error> {
        sqlx::query(
            r#"SELECT EXISTS(SELECT 1 FROM stack WHERE author_did = $1 AND uri = $2) AS "exists"#,
        )
        .bind(author_did)
        .bind(stack_uri)
        .fetch_one(pool)
        .await
        .map(|r| r.get("exists"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackArgs {
    pub uri: String,
    pub author_did: String,
    pub back_lang: Option<String>,
    pub front_lang: Option<String>,
    pub label: String,
    pub indexed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbCard {
    pub uri: String,
    pub author_did: String,
    pub back_lang: String,
    pub back_text: String,
    pub front_lang: String,
    pub front_text: String,
    pub created_at: DateTime<Utc>,
    pub indexed_at: DateTime<Utc>,
    pub stack_id: String,
}

impl DbCard {
    fn new(
        CardArgs {
            uri,
            author_did,
            back_lang,
            back_text,
            front_lang,
            front_text,
            indexed_at,
            stack_id,
        }: CardArgs,
    ) -> Self {
        let ia = indexed_at.unwrap_or_else(Utc::now);
        Self {
            uri,
            author_did,
            back_lang,
            back_text,
            front_lang,
            front_text,
            created_at: ia,
            indexed_at: ia,
            stack_id,
        }
    }
    async fn save_with_executor<'a, Ex>(&self, executor: Ex) -> Result<(), sqlx::Error>
    where
        Ex: sqlx::Executor<'a, Database = Postgres>,
    {
        sqlx::query(
            "
      INSERT INTO card (uri, author_did, back_lang, back_text, front_lang, front_text, created_at, indexed_at, stack_id)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8);
    ",
        )
        .bind(&self.uri)
        .bind(&self.author_did)
        .bind(&self.back_lang)
        .bind(&self.back_text)
        .bind(&self.front_lang)
        .bind(&self.front_text)
        .bind(self.created_at)
        .bind(self.indexed_at)
        .bind(&self.stack_id)
        .execute(executor)
        .await?;
        Ok(())
    }
    pub async fn upsert(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        let existing = sqlx::query("SELECT uri FROM card WHERE uri = $1")
            .bind(&self.uri)
            .fetch_optional(&mut *tx)
            .await?;
        if existing.is_some() {
            sqlx::query(
                "
    UPDATE card SET author_did = $2, back_lang = $3, front_lang = $4, label = $5, indexed_at = $6, stack_id = $7
    WHERE uri = $1",
            )
            .bind(&self.uri)
            .bind(&self.author_did)
            .bind(&self.back_lang)
            .bind(&self.back_text)
            .bind(&self.front_lang)
            .bind(&self.front_text)
            .bind(self.indexed_at)
            .bind(&self.stack_id)
            .execute(&mut *tx)
            .await?;
        } else {
            self.save_with_executor(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }
    pub async fn delete_by_uri(uri: &str, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM card WHERE uri = $1")
            .bind(uri)
            .execute(pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardArgs {
    pub uri: String,
    pub author_did: String,
    pub back_lang: String,
    pub back_text: String,
    pub front_lang: String,
    pub front_text: String,
    pub indexed_at: Option<DateTime<Utc>>,
    pub stack_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DisplayCard {
    pub uri: String,
    pub back_lang: String,
    pub back_text: String,
    pub front_lang: String,
    pub front_text: String,
}

impl DisplayCard {
    pub async fn stack_cards(stack_uri: &str, pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as(
            "
SELECT uri, back_lang, back_text, front_lang, front_text FROM card WHERE stack_uri = $1
",
        )
        .bind(stack_uri)
        .fetch_all(pool)
        .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StackDetails {
    pub uri: String,
    pub back_lang: Option<String>,
    pub front_lang: Option<String>,
    pub label: String,
}

impl StackDetails {
    pub async fn user_stacks(did: &str, pool: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as(
            "
SELECT uri, back_lang, front_lang, label FROM stack WHERE author_did = $1",
        )
        .bind(did)
        .fetch_all(pool)
        .await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthSession {
    pub key: String,
    pub session: String,
}

impl AuthSession {
    pub fn new<V: Serialize>(key: String, session: V) -> Self {
        let session = serde_json::to_string(&session).unwrap();
        Self { key, session }
    }
    pub async fn get_by_did(did: &str, pool: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as(
            "
SELECT key, session FROM auth_session WHERE key = $1 LIMIT 1
",
        )
        .bind(did)
        .fetch_optional(pool)
        .await
    }
    pub async fn save_or_update(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        let existing = sqlx::query("SELECT key FROM auth_session WHERE key = $1")
            .bind(&self.key)
            .fetch_optional(&mut *tx)
            .await?;
        let q = if existing.is_some() {
            sqlx::query(
                "
UPDATE auth_session SET session = $2 WHERE key = $1
",
            )
        } else {
            sqlx::query("INSERT INTO auth_session (key, session) VALUES ($1, $2)")
        };
        q.bind(&self.key)
            .bind(&self.session)
            .execute(&mut *tx)
            .await?;
        Ok(())
    }
    pub async fn delete_all(pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM auth_session")
            .execute(pool)
            .await?;
        Ok(())
    }
    pub async fn delete_by_did(did: &str, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM auth_session WHERE key = $1")
            .bind(did)
            .execute(pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthState {
    pub key: String,
    pub state: String,
}

impl AuthState {
    pub fn new<V: Serialize>(key: String, state: V) -> Self {
        let state = serde_json::to_string(&state).unwrap();
        Self { key, state }
    }
    pub async fn get_by_did(did: &str, pool: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as(
            "
SELECT key, state FROM auth_state WHERE key = $1 LIMIT 1
",
        )
        .bind(did)
        .fetch_optional(pool)
        .await
    }
    pub async fn save_or_update(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;
        let existing = sqlx::query("SELECT key FROM auth_state WHERE key = $1")
            .bind(&self.key)
            .fetch_optional(&mut *tx)
            .await?;
        let q = if existing.is_some() {
            sqlx::query(
                "
UPDATE auth_state SET state = $2 WHERE key = $1
",
            )
        } else {
            sqlx::query("INSERT INTO auth_state (key, state) VALUES ($1, $2)")
        };
        q.bind(&self.key)
            .bind(&self.state)
            .execute(&mut *tx)
            .await?;
        Ok(())
    }
    pub async fn delete_all(pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM auth_state").execute(pool).await?;
        Ok(())
    }
    pub async fn delete_by_did(did: &str, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM auth_state WHERE key = $1")
            .bind(did)
            .execute(pool)
            .await?;
        Ok(())
    }
    pub async fn get_by_key(key: &str, pool: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let res = sqlx::query_as(
            "
        SELECT key, session FROM auth_state WHERE key = $1 LIMIT 1
                ",
        )
        .bind(key)
        .fetch_optional(pool)
        .await?;
        Ok(res)
    }
    pub async fn delete_by_key(key: &str, pool: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query(
            "
DELETE FROM auth_state WHERE key = $1
",
        )
        .bind(key)
        .execute(pool)
        .await?;
        Ok(())
    }
}
