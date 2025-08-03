use super::OAuthClientType;
use crate::{resolver::HickoryDnsTxtResolver, storage::DbSessionStore};
use actix_session::Session;
use atrium_api::{agent, types::string::Did};
use atrium_identity::{did::CommonDidResolver, handle::AtprotoHandleResolver};
use atrium_oauth::DefaultHttpClient;
use atrium_oauth::OAuthSession;

pub(super) type Agent = agent::Agent<
    OAuthSession<
        DefaultHttpClient,
        CommonDidResolver<DefaultHttpClient>,
        AtprotoHandleResolver<HickoryDnsTxtResolver, DefaultHttpClient>,
        DbSessionStore,
    >,
>;

pub(super) struct AtS {
    pub(super) agent: Agent,
    pub(super) did: Did,
}

pub(super) async fn get_session_agent_and_did(
    oauth_client: &OAuthClientType,
    session: &Session,
) -> Option<AtS> {
    if let Some(did_string) = session.get::<String>("did").unwrap_or(None) {
        let did = Did::new(did_string).expect("invalid did");
        match oauth_client.restore(&did).await {
            Ok(s) => {
                let agent = Agent::new(s);
                Some(AtS { agent, did })
            }
            Err(err) => {
                log::error!(
                    "Error restoring session, we are removing the session from the cookie: {err}"
                );
                session.purge();
                None
            }
        }
    } else {
        None
    }
}
