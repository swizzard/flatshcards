pub(crate) mod cards;
pub(crate) mod stacks;
pub(crate) mod user_management;

pub(crate) use user_management::OAuthClientType;

use crate::db;
use actix_session::Session;
use actix_web::{Responder, Result, get, web};
use atrium_api::{agent::Agent, types::string::Did};
use sqlx::postgres::PgPool;

#[get("/")]
pub(crate) async fn home(
    session: Session,
    oauth_client: web::Data<user_management::OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
) -> Result<impl Responder> {
    const TITLE: &str = "Home";

    // If the user is signed in, get an agent which communicates with their server
    match session.get::<String>("did").unwrap_or(None) {
        Some(did) => {
            let did = Did::new(did).expect("failed to parse did");
            let stacks = db::StackDetails::user_stacks(&did, &db_pool)
                .await
                .unwrap_or_else(|err| {
                    log::error!("Error loading statuses: {err}");
                    vec![]
                });
            // gets the user's session from the session store to resume
            match oauth_client.restore(&did).await {
                Ok(session) => {
                    //Creates an agent to make authenticated requests
                    let agent = Agent::new(session);

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
                    let mut error = None;
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
                            error = Some("Can't get profile: {err}");
                        }
                    }
                    let html = HomeTemplate {
                        title: TITLE,
                        stacks,
                        profile: pr,
                        lang_choices: lang_choices(),
                        error,
                    }
                    .render()
                    .expect("template should be valid");

                    Ok(web::Html::new(html))
                }
                Err(err) => {
                    // Destroys the system or you're in a loop
                    session.purge();
                    log::error!("Error restoring session: {err}");
                    let error_html = ErrorTemplate {
                        title: "Error",
                        error: "Was an error resuming the session, please check the logs.",
                    }
                    .render()
                    .expect("template should be valid");
                    Ok(web::Html::new(error_html))
                }
            }
        }

        None => {
            let html = HomeTemplate {
                title: TITLE,
                profile: None,
                stacks: Vec::new(),
                lang_choices: lang_choices(),
                error: None,
            }
            .render()
            .expect("template should be valid");

            Ok(web::Html::new(html))
        }
    }
}
