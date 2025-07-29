/// Storage impls to persis OAuth sessions if you are not using the memory stores
/// https://github.com/bluesky-social/statusphere-example-app/blob/main/src/auth/storage.ts
use crate::db::{AuthSession, AuthState};
use atrium_api::types::string::Did;
use atrium_common::store::Store;
use atrium_oauth::store::session::SessionStore;
use atrium_oauth::store::state::StateStore;
use serde::Serialize;
use serde::de::DeserializeOwned;
use sqlx::postgres::PgPool;
use std::fmt::Debug;
use std::hash::Hash;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbStoreError {
    #[error("Invalid session")]
    InvalidSession,
    #[error("No session found")]
    NoSessionFound,
    #[error("Database error: {0}")]
    DatabaseError(sqlx::Error),
}

///Persistent session store in sqlite
impl SessionStore for DbSessionStore {}

pub struct DbSessionStore {
    db_pool: PgPool,
}

impl DbSessionStore {
    pub fn new(db: PgPool) -> Self {
        Self { db_pool: db }
    }
}

impl<K, V> Store<K, V> for DbSessionStore
where
    K: Debug + Eq + Hash + Send + Sync + 'static + From<Did> + AsRef<str>,
    V: Debug + Clone + Send + Sync + 'static + Serialize + DeserializeOwned,
{
    type Error = DbStoreError;
    async fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
        let did = key.as_ref();
        match AuthSession::get_by_did(did, &self.db_pool).await {
            Ok(Some(auth_session)) => {
                let deserialized_session: V = serde_json::from_str(&auth_session.session)
                    .map_err(|_| DbStoreError::InvalidSession)?;
                Ok(Some(deserialized_session))
            }
            Ok(None) => Err(DbStoreError::NoSessionFound),
            Err(db_error) => {
                log::error!("Database error: {db_error}");
                Err(DbStoreError::DatabaseError(db_error))
            }
        }
    }

    async fn set(&self, key: K, value: V) -> Result<(), Self::Error> {
        let did = key.as_ref().to_string();
        let auth_session = AuthSession::new(did, value);
        auth_session
            .save_or_update(&self.db_pool)
            .await
            .map_err(DbStoreError::DatabaseError)?;
        Ok(())
    }

    async fn del(&self, _key: &K) -> Result<(), Self::Error> {
        let did = _key.as_ref();
        AuthSession::delete_by_did(did, &self.db_pool)
            .await
            .map_err(DbStoreError::DatabaseError)?;
        Ok(())
    }

    async fn clear(&self) -> Result<(), Self::Error> {
        AuthSession::delete_all(&self.db_pool)
            .await
            .map_err(DbStoreError::DatabaseError)?;
        Ok(())
    }
}

///Persistent session state in sqlite
impl StateStore for DbStateStore {}

pub struct DbStateStore {
    db_pool: PgPool,
}

impl DbStateStore {
    pub fn new(db: PgPool) -> Self {
        Self { db_pool: db }
    }
}

impl<K, V> Store<K, V> for DbStateStore
where
    K: Debug + Eq + Hash + Send + Sync + 'static + From<Did> + AsRef<str>,
    V: Debug + Clone + Send + Sync + 'static + Serialize + DeserializeOwned,
{
    type Error = DbStoreError;
    async fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
        let key = key.as_ref();
        match AuthState::get_by_key(key, &self.db_pool).await {
            Ok(Some(auth_state)) => {
                let deserialized_state: V = serde_json::from_str(&auth_state.state)
                    .map_err(|_| DbStoreError::InvalidSession)?;
                Ok(Some(deserialized_state))
            }
            Ok(None) => Err(DbStoreError::NoSessionFound),
            Err(db_error) => {
                log::error!("Database error: {db_error}");
                Err(DbStoreError::DatabaseError(db_error))
            }
        }
    }

    async fn set(&self, key: K, value: V) -> Result<(), Self::Error> {
        let did = key.as_ref().to_string();
        let auth_state = AuthState::new(did, value);
        auth_state
            .save_or_update(&self.db_pool)
            .await
            .map_err(DbStoreError::DatabaseError)?;
        Ok(())
    }

    async fn del(&self, _key: &K) -> Result<(), Self::Error> {
        let key = _key.as_ref();
        AuthState::delete_by_key(key, &self.db_pool)
            .await
            .map_err(DbStoreError::DatabaseError)?;
        Ok(())
    }

    async fn clear(&self) -> Result<(), Self::Error> {
        AuthState::delete_all(&self.db_pool)
            .await
            .map_err(DbStoreError::DatabaseError)?;
        Ok(())
    }
}
