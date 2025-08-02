use crate::{
    db,
    lang::{is_lang, lang_choices},
    lexicons::{
        record::KnownRecord,
        xyz::flatshcards::{Card, card},
    },
    routes::OAuthClientType,
    templates::ErrorTemplate,
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
    todo!()
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CardForm {
    front_lang: String,
    front_text: String,
    back_lang: String,
    back_text: String,
    stack_uri: String,
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
