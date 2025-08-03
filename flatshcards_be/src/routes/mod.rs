mod atproto_agent;
pub(crate) mod cards;
pub(crate) mod stacks;
pub(crate) mod user_management;

pub(crate) use user_management::OAuthClientType;

use crate::routes::atproto_agent::{AtS, get_session_agent_and_did};
use crate::{
    db,
    templates::{HomeTemplate, Profile},
};
use actix_session::Session;
use actix_web::{Responder, Result, get, web};
use askama::Template;
use sqlx::postgres::PgPool;

#[get("/")]
pub(crate) async fn home(
    session: Session,
    oauth_client: web::Data<user_management::OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
) -> Result<impl Responder> {
    const TITLE: &str = "Home";

    if let Some(AtS { agent, did }) = get_session_agent_and_did(&oauth_client, &session).await {
        let stacks = db::StackDetails::user_stacks(&did, &db_pool)
            .await
            .unwrap_or_else(|err| {
                log::error!("Error loading statuses: {err}");
                vec![]
            });
        // Fetch additional information about the logged-in user
        let profile = agent
            .api
            .app
            .bsky
            .actor
            .get_profile(
                atrium_api::app::bsky::actor::get_profile::ParametersData {
                    actor: atrium_api::types::string::AtIdentifier::Did(did),
                }
                .into(),
            )
            .await;
        let mut pr = None;
        match profile {
            Ok(profile) => {
                pr = {
                    let profile_data = Profile {
                        did: profile.did.to_string(),
                        display_name: profile.display_name.clone(),
                    };
                    Some(profile_data)
                }
            }
            Err(err) => {
                log::error!("Error accessing profile: {err}");
            }
        }
        let html = HomeTemplate {
            title: TITLE,
            stacks,
            profile: pr,
        }
        .render()
        .expect("template should be valid");

        Ok(web::Html::new(html))
    } else {
        let html = HomeTemplate {
            title: TITLE,
            profile: None,
            stacks: Vec::new(),
        }
        .render()
        .expect("template should be valid");

        Ok(web::Html::new(html))
    }
}
