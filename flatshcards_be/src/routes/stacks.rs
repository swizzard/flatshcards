use crate::{
    db,
    lang::{is_lang, lang_choices},
    lexicons::{
        record::KnownRecord,
        xyz::flatshcards::{Stack, stack},
    },
    routes::OAuthClientType,
    templates::ErrorTemplate,
};
use actix_session::Session;
use actix_web::{
    HttpRequest, HttpResponse, Responder, delete, post, put,
    web::{self, Redirect},
};
use askama::Template;
use atrium_api::{
    agent::Agent,
    types::{
        Collection,
        string::{Datetime, Did},
    },
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

/// Creates a new stack
#[post("/stacks/create")]
pub(crate) async fn create_stack(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    form: web::Form<StackForm>,
) -> HttpResponse {
    // Check if the user is logged in
    match session.get::<String>("did").unwrap_or(None) {
        Some(did_string) => {
            let did = Did::new(did_string.clone()).expect("failed to parse did");
            // gets the user's session from the session store to resume
            match oauth_client.restore(&did).await {
                Ok(session) => {
                    let form = form.clone();
                    if !form.front_valid() {
                        let bad_lang = form.front_lang.unwrap(); // None is valid
                        let error_html = ErrorTemplate {
                            title: "Form Validation",
                            error: format!("Invalid front language {bad_lang}").as_ref(),
                        }
                        .render()
                        .expect("template should be valid");
                        return HttpResponse::Ok().body(error_html);
                    };
                    if !form.back_valid() {
                        let bad_lang = form.back_lang.unwrap(); // None is valid
                        let error_html = ErrorTemplate {
                            title: "Form Validation",
                            error: format!("Invalid back language {bad_lang}").as_ref(),
                        }
                        .render()
                        .expect("template should be valid");
                        return HttpResponse::Ok().body(error_html);
                    };
                    let agent = Agent::new(session);
                    //Creates a strongly typed ATProto record
                    let stack = form.to_record();

                    // TODO no data validation yet from esquema
                    // Maybe you'd like to add it? https://github.com/fatfingers23/esquema/issues/3

                    let create_result = agent
                        .api
                        .com
                        .atproto
                        .repo
                        .create_record(
                            atrium_api::com::atproto::repo::create_record::InputData {
                                collection: Stack::NSID.parse().unwrap(),
                                repo: did.into(),
                                rkey: None,
                                record: stack.into(),
                                swap_commit: None,
                                validate: None,
                            }
                            .into(),
                        )
                        .await;

                    match create_result {
                        Ok(record) => {
                            let args = form.to_args(record.uri.clone(), did_string);
                            let stack = db::DbStack::new(args);
                            let _ = stack.save(&db_pool).await;
                            Redirect::to("/")
                                .see_other()
                                .respond_to(&request)
                                .map_into_boxed_body()
                        }
                        Err(err) => {
                            log::error!("Error creating status: {err}");
                            let error_html = ErrorTemplate {
                                title: "Error",
                                error: "Was an error creating the status, please check the logs.",
                            }
                            .render()
                            .expect("template should be valid");
                            HttpResponse::Ok().body(error_html)
                        }
                    }
                }
                Err(err) => {
                    // Destroys the system or you're in a loop
                    session.purge();
                    log::error!(
                        "Error restoring session, we are removing the session from the cookie: {err}"
                    );
                    let error_html = ErrorTemplate {
                        title: "Error",
                        error: "Was an error resuming the session, please check the logs.",
                    }
                    .render()
                    .expect("template should be valid");
                    HttpResponse::Ok().body(error_html)
                }
            }
        }
        None => {
            let error_template = ErrorTemplate {
                title: "Error",
                error: "You must be logged in to create a status.",
            }
            .render()
            .expect("template should be valid");
            HttpResponse::Ok().body(error_template)
        }
    }
}

/// The post body for creating a new stack
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StackForm {
    back_lang: Option<String>,
    front_lang: Option<String>,
    stack_label: String,
}
impl StackForm {
    fn lang_valid(lang: &str) -> bool {
        is_lang(lang)
    }
    fn front_valid(&self) -> bool {
        if let Some(ref l) = self.front_lang {
            StackForm::lang_valid(l)
        } else {
            true
        }
    }
    fn back_valid(&self) -> bool {
        if let Some(ref l) = self.back_lang {
            StackForm::lang_valid(l)
        } else {
            true
        }
    }
    fn to_args(&self, uri: String, author_did: String) -> db::StackArgs {
        db::StackArgs {
            uri,
            author_did,
            back_lang: self.back_lang.clone(),
            front_lang: self.front_lang.clone(),
            label: self.stack_label.clone(),
            indexed_at: None,
        }
    }
    fn to_record(&self) -> KnownRecord {
        stack::Stack {
            back_lang: self.back_lang.clone(),
            front_lang: self.front_lang.clone(),
            label: self.stack_label.clone(),
            created_at: Datetime::now(),
        }
        .into()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct StackUriPath {
    card_uri: String,
}

#[delete("/stacks/edit")]
pub(crate) async fn delete_stack(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    stack_uri: web::Path<StackUriPath>,
) -> HttpResponse {
    todo!()
}
#[put("/stacks/edit")]
pub(crate) async fn put_stack(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    stack_uri: web::Path<StackUriPath>,
    form: web::Form<StackForm>,
) -> HttpResponse {
    todo!()
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CloneStackForm {
    src_uri: String,
    dest_uri: String,
}

#[post("/stacks/clone")]
pub(crate) async fn clone_stack(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    form: web::Form<CloneStackForm>,
) -> HttpResponse {
    todo!()
}
