use crate::{
    db,
    lang::{is_lang, lang_choices},
    lexicons::{
        record::KnownRecord,
        xyz::flatshcards::{Card, card},
    },
    routes::{AtS, OAuthClientType, get_session_agent_and_did},
    templates::{self, ErrorTemplate},
};
use actix_session::Session;
use actix_web::{
    HttpRequest, HttpResponse, Responder, delete, post, put,
    web::{self, Redirect},
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
        if !form.front_valid() {
            let error_html = templates::FormError {

            }
        }
        
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
    fn lang_valid(lang: &str) -> bool {
        is_lang(lang)
    }
    fn front_valid(&self) -> bool {
        CardForm::lang_valid(self.front_lang)
    }
    fn back_valid(&self) -> bool {
        CardForm::lang_valid(self.back_lang)
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
    stack_uri: web::Path<CardUriPath>,
) -> HttpResponse {
    todo!()
}

#[put("/cards/edit/{card_uri}")]
pub(crate) async fn put_card(
    request: HttpRequest,
    session: Session,
    oauth_client: web::Data<OAuthClientType>,
    db_pool: web::ThinData<PgPool>,
    stack_uri: web::Path<CardUriPath>,
    form: web::Form<CardForm>,
) -> HttpResponse {
    todo!()
}
