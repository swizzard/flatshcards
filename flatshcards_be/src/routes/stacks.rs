use crate::{
    db,
    lang::{is_lang, lang_choices},
    lexicons::{
        record::KnownRecord,
        xyz::flatshcards::{Stack, stack},
    },
    routes::{AtS, OAuthClientType, get_session_agent_and_did},
    templates::{self, ErrorTemplate},
};
use actix_session::Session;
use actix_web::{
    HttpRequest, HttpResponse, Responder, delete, get, post, put,
    web::{self, Redirect},
};
use askama::Template;
use atrium_api::types::{
    Collection,
    string::{Datetime, RecordKey},
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[get("/stacks/create")]
pub(crate) async fn create_stacks_page(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
) -> HttpResponse {
    if get_session_agent_and_did(&oauth_client, &session)
        .await
        .is_some()
    {
        let html = templates::CreateStackTemplate {
            title: "Create Stack",
            lang_choices: lang_choices(),
            error: None,
        }
        .render()
        .unwrap();
        HttpResponse::Ok().body(html)
    } else {
        let error_html = ErrorTemplate::session_agent_did().render().unwrap();
        HttpResponse::Unauthorized().body(error_html)
    }
}

/// Creates a new stack
#[post("/stacks/create")]
pub(crate) async fn create_stack(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    form: web::Form<StackForm>,
) -> HttpResponse {
    if let Some(AtS { agent, did }) = get_session_agent_and_did(&oauth_client, &session).await {
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
        let stack = form.to_record();
        let db_did = did.clone().to_string();

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
                let args = form.to_args(record.uri.clone(), db_did);
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
    } else {
        let error_template = ErrorTemplate {
            title: "Error",
            error: "You must be logged in to create a status.",
        }
        .render()
        .expect("template should be valid");
        HttpResponse::Ok().body(error_template)
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
    stack_uri: String,
}

#[delete("/stacks/edit/{stack_uri}")]
pub(crate) async fn delete_stack(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    stack_uri: web::Path<StackUriPath>,
) -> HttpResponse {
    if let Some(AtS { agent, did }) = get_session_agent_and_did(&oauth_client, &session).await {
        let db_did = did.clone();
        let StackUriPath { stack_uri } = stack_uri.into_inner();
        match db::DbStack::is_owned_by(&db_did, &stack_uri, &db_pool).await {
            Ok(true) => {
                if let Err(err) = db::DbStack::delete_by_uri(&stack_uri, &db_pool).await {
                    log::error!("Error deleting stack from db: {err}");
                    let error_html = ErrorTemplate {
                        title: "Error",
                        error: "Error deleting stack from database.",
                    }
                    .render()
                    .unwrap();
                    HttpResponse::InternalServerError().body(error_html)
                } else {
                    let rkey = RecordKey::new(stack_uri).unwrap();
                    let delete_result = agent
                        .api
                        .com
                        .atproto
                        .repo
                        .delete_record(
                            atrium_api::com::atproto::repo::delete_record::InputData {
                                collection: Stack::NSID.parse().unwrap(),
                                repo: did.into(),
                                rkey,
                                swap_commit: None,
                                swap_record: None,
                            }
                            .into(),
                        )
                        .await;
                    if delete_result.is_err() {
                        log::error!("error deleting stack from repo");
                    };
                    Redirect::to("/")
                        .see_other()
                        .respond_to(&request)
                        .map_into_boxed_body()
                }
            }
            Ok(false) => {
                let error_html = ErrorTemplate {
                    title: "Forbidden",
                    error: "You do not have permimssion to perform this action",
                }
                .render()
                .unwrap();
                HttpResponse::Forbidden().body(error_html)
            }
            Err(err) => {
                log::error!("Error querying db: {err}");
                let error_html = ErrorTemplate {
                    title: "Error",
                    error: "Error querying database",
                }
                .render()
                .unwrap();
                HttpResponse::InternalServerError().body(error_html)
            }
        }
    } else {
        log::error!("error retrieving did and agent");
        let error_html = ErrorTemplate::session_agent_did().render().unwrap();
        HttpResponse::Unauthorized().body(error_html)
    }
}
#[put("/stacks/edit/{card_uri}")]
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
