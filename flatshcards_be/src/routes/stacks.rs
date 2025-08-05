use crate::{
    db,
    lang::{is_lang, lang_choices},
    lexicons::{
        record::KnownRecord,
        xyz::flatshcards::{Card, Stack, card, stack},
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
use atrium_api::com::atproto::repo::{create_record, delete_record, put_record};
use atrium_api::types::{
    Collection, Object,
    string::{Datetime, Did, RecordKey},
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[get("/stacks/create")]
pub(crate) async fn create_stacks_page(
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
                create_record::InputData {
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
                    error: "There was an error creating the stack",
                }
                .render()
                .expect("template should be valid");
                HttpResponse::Ok().body(error_html)
            }
        }
    } else {
        let error_template = ErrorTemplate {
            title: "Error",
            error: "You must be logged in to create a stack.",
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
    fn to_update_args(&self, uri: String, author_did: String) -> db::StackUpdateArgs {
        db::StackUpdateArgs {
            uri,
            author_did,
            back_lang: self.back_lang.clone(),
            front_lang: self.front_lang.clone(),
            label: self.stack_label.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct StackUriPath {
    stack_uri: String,
}

#[get("/stacks/edit/{stack_uri}")]
pub(crate) async fn edit_stack_page(
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    stack_uri: web::Path<StackUriPath>,
) -> HttpResponse {
    if let Some(AtS { did, .. }) = get_session_agent_and_did(&oauth_client, &session).await {
        let StackUriPath { stack_uri } = stack_uri.into_inner();
        match db::DbStack::get_owned_by(&did, &stack_uri, &db_pool).await {
            Ok(Some(stack)) => {
                let lc = lang_choices();
                let st = stack.clone();
                let stack_id = st.uri.clone();
                let add_card = templates::AddCardTemplate {
                    lang_choices: lc.clone(),
                    stack: st,
                    error: None,
                };
                match db::DisplayCard::stack_cards(&stack_uri, &db_pool).await {
                    Err(err) => {
                        log::error!("error retrieving cards from db {err}");
                        let html = templates::EditStackTemplate {
                            title: "Edit Stack",
                            lang_choices: lc.clone(),
                            stack,
                            error: Some(err.to_string().as_ref()),
                            add_card,
                            edit_cards: templates::EditCardsTemplate {
                                lang_choices: lc.clone(),
                                cards: Vec::new(),
                                stack_id,
                            },
                        }
                        .render()
                        .unwrap();
                        HttpResponse::BadRequest().body(html)
                    }
                    Ok(cards) => {
                        let stack_id = stack.uri.clone();
                        let html = templates::EditStackTemplate {
                            title: "Edit Stack",
                            lang_choices: lc.clone(),
                            stack,
                            error: None,
                            add_card,
                            edit_cards: templates::EditCardsTemplate {
                                lang_choices: lc.clone(),
                                cards,
                                stack_id,
                            },
                        }
                        .render()
                        .unwrap();
                        HttpResponse::Ok().body(html)
                    }
                }
            }
            Ok(None) => {
                let error_html = ErrorTemplate::stack_not_found().render().unwrap();
                HttpResponse::NotFound().body(error_html)
            }
            Err(err) => {
                log::error!("error retrieving stack {err}");
                let error_html = ErrorTemplate {
                    title: "Error",
                    error: "Error retrieving stack",
                }
                .render()
                .unwrap();
                HttpResponse::InternalServerError().body(error_html)
            }
        }
    } else {
        let error_html = ErrorTemplate {
            title: "Error",
            error: "You must be logged in to edit stacks",
        }
        .render()
        .unwrap();
        HttpResponse::Unauthorized().body(error_html)
    }
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
                            delete_record::InputData {
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
                let error_html = ErrorTemplate::forbidden().render().unwrap();
                HttpResponse::Forbidden().body(error_html)
            }
            Err(err) => {
                log::error!("Error querying db: {err}");
                let error_html = ErrorTemplate::db_query().render().unwrap();
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
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    stack_uri: web::Path<StackUriPath>,
    form: web::Form<StackForm>,
) -> HttpResponse {
    if let Some(AtS { agent, did }) = get_session_agent_and_did(&oauth_client, &session).await {
        let form = form.clone();
        let db_form = form.clone();
        let db_did = did.clone();
        let StackUriPath { stack_uri } = stack_uri.into_inner();
        let db_uri = stack_uri.clone();
        match db::DbStack::is_owned_by(&db_did, &stack_uri, &db_pool).await {
            Ok(true) => {
                let update_result = agent
                    .api
                    .com
                    .atproto
                    .repo
                    .put_record(
                        put_record::InputData {
                            collection: Stack::NSID.parse().unwrap(),
                            record: form.to_record().into(),
                            repo: did.into(),
                            rkey: RecordKey::new(stack_uri.clone()).unwrap(),
                            swap_commit: None,
                            swap_record: None,
                            validate: None,
                        }
                        .into(),
                    )
                    .await;
                if let Err(err) = update_result {
                    log::error!("error updating stack in atmosphere {err}");
                    let error_html = ErrorTemplate {
                        title: "Error",
                        error: "Error updating stack",
                    }
                    .render()
                    .unwrap();
                    HttpResponse::InternalServerError().body(error_html)
                } else {
                    match db_form
                        .to_update_args(db_uri, db_did.to_string())
                        .update_owned(&db_pool)
                        .await
                    {
                        Ok(Some(updated)) => {
                            let lc = lang_choices();
                            match db::DisplayCard::stack_cards(&stack_uri, &db_pool).await {
                                Err(err) => {
                                    log::error!("error retrieving post-update stack cards {err}");
                                    let html = templates::EditStackTemplate {
                                        title: "Edit Stack",
                                        lang_choices: lc.clone(),
                                        stack: updated.clone(),
                                        error: Some("Could not retrieve cards, please try again."),
                                        add_card: templates::AddCardTemplate {
                                            lang_choices: lc.clone(),
                                            stack: updated.clone(),
                                            error: None,
                                        },
                                        edit_cards: templates::EditCardsTemplate {
                                            lang_choices: lc.clone(),
                                            cards: Vec::new(),
                                            stack_id: updated.uri.clone(),
                                        },
                                    }
                                    .render()
                                    .unwrap();
                                    HttpResponse::BadRequest().body(html)
                                }
                                Ok(cards) => {
                                    let html = templates::EditStackTemplate {
                                        title: "Edit Stack",
                                        lang_choices: lc.clone(),
                                        stack: updated.clone(),
                                        error: None,
                                        add_card: templates::AddCardTemplate {
                                            lang_choices: lc.clone(),
                                            stack: updated.clone(),
                                            error: None,
                                        },
                                        edit_cards: templates::EditCardsTemplate {
                                            lang_choices: lc.clone(),
                                            cards,
                                            stack_id: updated.uri.clone(),
                                        },
                                    }
                                    .render()
                                    .unwrap();
                                    HttpResponse::Ok().body(html)
                                }
                            }
                        }
                        Ok(None) => {
                            let error_html = ErrorTemplate::forbidden().render().unwrap();
                            HttpResponse::Forbidden().body(error_html)
                        }
                        Err(err) => {
                            log::error!("error updating stack {err}");
                            let error_html = ErrorTemplate {
                                title: "Error",
                                error: "Error updating stack",
                            }
                            .render()
                            .unwrap();
                            HttpResponse::InternalServerError().body(error_html)
                        }
                    }
                }
            }
            Ok(false) => {
                let error_html = ErrorTemplate::forbidden().render().unwrap();
                HttpResponse::Forbidden().body(error_html)
            }
            Err(err) => {
                log::error!("error querying database {err}");
                let error_html = ErrorTemplate::db_query().render().unwrap();
                HttpResponse::InternalServerError().body(error_html)
            }
        }
    } else {
        let error_html = ErrorTemplate::session_agent_did().render().unwrap();
        HttpResponse::Unauthorized().body(error_html)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CloneStackPath {
    src_uri: String,
}

#[post("/stacks/clone/{src_uri}")]
pub(crate) async fn clone_stack(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    path: web::Path<CloneStackPath>,
) -> HttpResponse {
    if let Some(AtS { agent, did }) = get_session_agent_and_did(&oauth_client, &session).await {
        let cl_did = did.clone();
        let CloneStackPath { src_uri } = path.into_inner();
        match db::DbStack::get_clone_data(&src_uri, &db_pool).await {
            Ok(Some(db::StackCloneData {
                back_lang,
                front_lang,
                label,
            })) => {
                let record: KnownRecord = stack::Stack {
                    back_lang,
                    front_lang,
                    label,
                    created_at: Datetime::now(),
                }
                .into();
                let create_result = agent
                    .api
                    .com
                    .atproto
                    .repo
                    .create_record(
                        create_record::InputData {
                            collection: Stack::NSID.parse().unwrap(),
                            repo: did.into(),
                            rkey: None,
                            record: record.into(),
                            swap_commit: None,
                            validate: None,
                        }
                        .into(),
                    )
                    .await;
                match create_result {
                    Ok(Object {
                        data:
                            create_record::OutputData {
                                uri: new_stack_uri, ..
                            },
                        ..
                    }) => match db::DbCard::get_clone_data(&src_uri, &db_pool).await {
                        Err(err) => {
                            log::error!("error getting cards to clone {err}");
                            let error_html = ErrorTemplate::db_query().render().unwrap();
                            HttpResponse::InternalServerError().body(error_html)
                        }
                        Ok(cards) => {
                            let nsu = new_stack_uri.clone();
                            if let Err(err) =
                                clone_stack_cards(cl_did, nsu, cards, &agent, &db_pool).await
                            {
                                log::error!("error cloning cards {err}");
                                let error_html = ErrorTemplate {
                                    title: "Error",
                                    error: "An error has occurred.",
                                }
                                .render()
                                .unwrap();
                                HttpResponse::InternalServerError().body(error_html)
                            } else {
                                let url =
                                    request.url_for("edit_stack_page", [new_stack_uri]).unwrap();
                                Redirect::to(url.as_str().to_owned())
                                    .see_other()
                                    .respond_to(&request)
                                    .map_into_boxed_body()
                            }
                        }
                    },
                    Err(err) => {
                        log::error!("error cloning stack in atmosphere {err}");
                        let error_html = ErrorTemplate {
                            title: "Error",
                            error: "An error has occurred.",
                        }
                        .render()
                        .unwrap();
                        HttpResponse::InternalServerError().body(error_html)
                    }
                }
            }
            Ok(None) => {
                let error_html = ErrorTemplate::stack_not_found().render().unwrap();
                HttpResponse::NotFound().body(error_html)
            }
            Err(err) => {
                log::error!("error getting clone data {err}");
                let error_html = ErrorTemplate::db_query().render().unwrap();
                HttpResponse::InternalServerError().body(error_html)
            }
        }
    } else {
        let error_html = ErrorTemplate::session_agent_did().render().unwrap();
        HttpResponse::Unauthorized().body(error_html)
    }
}

async fn clone_stack_cards(
    did: Did,
    new_stack_uri: String,
    cards: Vec<db::CardCloneData>,
    agent: &super::atproto_agent::Agent,
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut any_err: Option<Box<dyn std::error::Error>> = None;
    let mut work: std::collections::VecDeque<db::CardCloneData> = cards.into();
    while !work.is_empty() {
        let clone_data = work.pop_front().unwrap();
        let now = Datetime::now();
        let db_now = now.clone();
        let record_uri = new_stack_uri.clone();
        let db_uri = new_stack_uri.clone();
        let db_did = did.clone();
        let rec: KnownRecord = card::Card {
            back_lang: clone_data.back_lang.clone(),
            back_text: clone_data.back_text.clone(),
            created_at: now,
            front_lang: clone_data.front_lang.clone(),
            front_text: clone_data.front_text.clone(),
            stack_id: RecordKey::new(record_uri).unwrap(),
        }
        .into();
        let create_result = agent
            .api
            .com
            .atproto
            .repo
            .create_record(
                create_record::InputData {
                    collection: Card::NSID.parse().unwrap(),
                    repo: did.clone().into(),
                    rkey: None,
                    record: rec.into(),
                    swap_commit: None,
                    validate: None,
                }
                .into(),
            )
            .await;
        match create_result {
            Ok(Object {
                data: create_record::OutputData { uri, .. },
                ..
            }) => {
                let indexed_at: Option<chrono::DateTime<chrono::Utc>> =
                    Some(db_now.as_ref().to_utc());
                if let Err(err) = db::DbCard::new(db::CardArgs {
                    uri,
                    author_did: db_did.into(),
                    back_lang: clone_data.back_lang.clone(),
                    back_text: clone_data.back_text.clone(),
                    front_lang: clone_data.front_lang.clone(),
                    front_text: clone_data.front_text.clone(),
                    indexed_at,
                    stack_id: db_uri.clone(),
                })
                .save(pool)
                .await
                {
                    log::error!("error saving card in db, will ingest later {err}");
                };
            }
            Err(err) => {
                log::error!("error saving card in atmosphere, will try again {err}");
                if any_err.is_none() {
                    any_err.replace(Box::new(err));
                }
                work.push_back(clone_data)
            }
        }
    }
    if let Some(err) = any_err {
        Err(err)
    } else {
        Ok(())
    }
}
