use crate::{
    db,
    lang::{is_lang, lang_choices},
    lexicons::{
        record::KnownRecord,
        xyz::flatshcards::{Card, card},
    },
    routes::{AtS, OAuthClientType, get_session_agent_and_did},
    templates,
};
use actix_session::Session;
use actix_web::{
    HttpRequest, HttpResponse, Responder, delete, post, put,
    web::{self, Redirect},
};
use askama::Template;
use atrium_api::com::atproto::repo::{create_record, delete_record, put_record};
use atrium_api::types::{
    Collection,
    string::{Datetime, RecordKey},
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[post("/cards/create")]
pub(crate) async fn create_card(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    form: web::Form<CardForm>,
) -> HttpResponse {
    if let Some(AtS { agent, did }) = get_session_agent_and_did(&oauth_client, &session).await {
        let form = form.clone();
        if let Some(ref error) = form.validate() {
            let error_html = templates::FormError { error }.render().unwrap();
            return HttpResponse::BadRequest().body(error_html);
        };
        let card = form.as_record();
        let db_did = did.clone().to_string();
        let create_result = agent
            .api
            .com
            .atproto
            .repo
            .create_record(
                create_record::InputData {
                    collection: Card::NSID.parse().unwrap(),
                    repo: did.into(),
                    rkey: None,
                    record: card.into(),
                    swap_commit: None,
                    validate: None,
                }
                .into(),
            )
            .await;
        match create_result {
            Ok(record) => {
                let stack_id = form.stack_id.clone();
                let args = form.as_args(record.uri.clone(), db_did);
                let card = db::DbCard::new(args);
                let _ = card.save(&db_pool).await;
                let html = templates::EditSingleCardTemplate {
                    lang_choices: lang_choices(),
                    card: card.into(),
                    stack_id,
                }
                .render()
                .unwrap();
                HttpResponse::Created().body(html)
            }
            Err(err) => {
                log::error!("error creating card in atmosphere {err}");
                let error_html = templates::FormError {
                    error: "An unknown error has occurred.",
                }
                .render()
                .unwrap();
                HttpResponse::InternalServerError().body(error_html)
            }
        }
    } else {
        Redirect::to("/")
            .temporary()
            .respond_to(&request)
            .map_into_boxed_body()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CardForm {
    front_lang: String,
    front_text: String,
    back_lang: String,
    back_text: String,
    stack_id: String,
}

impl CardForm {
    fn validate(&self) -> Option<String> {
        if !is_lang(&self.front_lang) {
            let l = &self.front_lang;
            Some(format!("Invalid front language {l}"))
        } else if !is_lang(&self.back_lang) {
            let l = &self.back_lang;
            Some(format!("Invalid back langauge {l}"))
        } else {
            None
        }
    }
    fn as_record(&self) -> KnownRecord {
        card::Card {
            back_lang: self.back_lang.clone(),
            back_text: self.back_text.clone(),
            front_lang: self.front_lang.clone(),
            front_text: self.front_text.clone(),
            stack_id: RecordKey::new(self.stack_id.clone()).unwrap(),
            created_at: Datetime::now(),
        }
        .into()
    }
    fn as_args(&self, uri: String, author_did: String) -> db::CardArgs {
        db::CardArgs {
            uri,
            author_did,
            back_lang: self.back_lang.clone(),
            back_text: self.back_text.clone(),
            front_lang: self.front_lang.clone(),
            front_text: self.front_text.clone(),
            indexed_at: None,
            stack_id: self.stack_id.clone(),
        }
    }
    fn as_display(&self, uri: String) -> db::DisplayCard {
        db::DisplayCard {
            uri,
            back_lang: self.back_lang.clone(),
            back_text: self.back_text.clone(),
            front_lang: self.front_lang.clone(),
            front_text: self.front_text.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CardUriPath {
    card_uri: String,
}

#[delete("/cards/edit/{card_uri}")]
pub(crate) async fn delete_card(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    card_uri: web::Path<CardUriPath>,
) -> HttpResponse {
    if let Some(AtS { agent, did }) = get_session_agent_and_did(&oauth_client, &session).await {
        let CardUriPath { card_uri } = card_uri.into_inner();
        match db::DbCard::is_owned_by(&did, &card_uri, &db_pool).await {
            Err(err) => {
                log::error!("error checking card ownership {err}");
                Redirect::to("/").respond_to(&request).map_into_boxed_body()
            }
            Ok(false) => Redirect::to("/").respond_to(&request).map_into_boxed_body(),
            Ok(true) => {
                let db_uri = card_uri.clone();
                let rkey = RecordKey::new(card_uri).unwrap();
                let at_result = agent
                    .api
                    .com
                    .atproto
                    .repo
                    .delete_record(
                        delete_record::InputData {
                            collection: Card::NSID.parse().unwrap(),
                            repo: did.into(),
                            rkey,
                            swap_commit: None,
                            swap_record: None,
                        }
                        .into(),
                    )
                    .await;
                if let Err(err) = at_result {
                    log::error!("error deleting card in atmosphere {err}");
                    let error_html = templates::FormError {
                        error: "Error deleting card.",
                    }
                    .render()
                    .unwrap();
                    return HttpResponse::InternalServerError().body(error_html);
                };
                if let Err(err) = db::DbCard::delete_by_uri(&db_uri, &db_pool).await {
                    log::error!("error deleting card from db {err}");
                    let error_html = templates::FormError {
                        error: "Error deleting card.",
                    }
                    .render()
                    .unwrap();
                    return HttpResponse::InternalServerError().body(error_html);
                };
                HttpResponse::NoContent().into()
            }
        }
    } else {
        Redirect::to("/").respond_to(&request).map_into_boxed_body()
    }
}

#[put("/cards/edit/{card_uri}")]
pub(crate) async fn put_card(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    card_uri: web::Path<CardUriPath>,
    form: web::Form<CardForm>,
) -> HttpResponse {
    if let Some(AtS { agent, did }) = get_session_agent_and_did(&oauth_client, &session).await {
        let CardUriPath { card_uri } = card_uri.into_inner();
        let form = form.clone();
        match db::DbCard::is_owned_by(&did, &card_uri, &db_pool).await {
            Err(err) => {
                log::error!("error checking card ownership {err}");
                Redirect::to("/").respond_to(&request).map_into_boxed_body()
            }
            Ok(false) => Redirect::to("/").respond_to(&request).map_into_boxed_body(),
            Ok(true) => {
                let db_uri = card_uri.clone();
                let rkey = RecordKey::new(card_uri).unwrap();
                let at_result = agent
                    .api
                    .com
                    .atproto
                    .repo
                    .put_record(
                        put_record::InputData {
                            collection: Card::NSID.parse().unwrap(),
                            record: form.as_record().into(),
                            repo: did.into(),
                            rkey,
                            swap_commit: None,
                            swap_record: None,
                            validate: None,
                        }
                        .into(),
                    )
                    .await;
                match at_result {
                    Err(err) => {
                        log::error!("error editing card in atmosphere {err}");
                        let error_html = templates::FormError {
                            error: "Error editing card",
                        }
                        .render()
                        .unwrap();
                        HttpResponse::InternalServerError().body(error_html)
                    }
                    Ok(_record) => {
                        let card = form.as_display(db_uri);
                        let html = templates::EditSingleCardTemplate {
                            lang_choices: lang_choices(),
                            card,
                            stack_id: form.stack_id.clone(),
                        }
                        .render()
                        .unwrap();
                        HttpResponse::Ok().body(html)
                    }
                }
            }
        }
    } else {
        Redirect::to("/").respond_to(&request).map_into_boxed_body()
    }
}
