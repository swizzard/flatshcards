use crate::{
    db::create_tables_in_database,
    ingester::start_ingester,
    routes::{
        cards::{create_card, delete_card, put_card},
        home,
        stacks::{
            clone_stack, create_stack, create_stack_page, delete_stack, edit_stack_page, put_stack,
        },
        user_management::{login, login_post, logout, oauth_callback},
    },
    storage::{DbSessionStore, DbStateStore},
};
use actix_files::Files;
use actix_session::{SessionMiddleware, config::PersistentSession, storage::CookieSessionStore};
use actix_web::{
    App, HttpServer,
    cookie::{self, Key},
    middleware, web,
};
use atrium_identity::{
    did::{CommonDidResolver, CommonDidResolverConfig, DEFAULT_PLC_DIRECTORY_URL},
    handle::{AtprotoHandleResolver, AtprotoHandleResolverConfig},
};
use atrium_oauth::{
    AtprotoLocalhostClientMetadata, DefaultHttpClient, KnownScope, OAuthClient, OAuthClientConfig,
    OAuthResolverConfig, Scope,
};
use dotenv::dotenv;
use resolver::HickoryDnsTxtResolver;
use sqlx::postgres::PgPool;
use std::{io::Error, sync::Arc};

extern crate dotenv;

mod db;
mod ingester;
mod lang;
mod lexicons;
mod resolver;
mod routes;
mod storage;
mod templates;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    //Uses a default sqlite db path or use the one from env
    let db_connection_string = std::env::var("DB_URL").unwrap();

    //Crates a db pool to share resources to the db
    let pool = match PgPool::connect(&db_connection_string).await {
        Ok(pool) => pool,
        Err(err) => {
            log::error!("Error creating the db pool: {err}");
            return Err(Error::other("db pool could not be created."));
        }
    };

    //Creates the DB and tables
    create_tables_in_database(&pool)
        .await
        .expect("Could not create the database");

    //Create a new handle resolver for the home page
    let http_client = Arc::new(DefaultHttpClient::default());

    let handle_resolver = CommonDidResolver::new(CommonDidResolverConfig {
        plc_directory_url: DEFAULT_PLC_DIRECTORY_URL.to_string(),
        http_client: http_client.clone(),
    });
    let handle_resolver = Arc::new(handle_resolver);

    // Create a new OAuth client
    let http_client = Arc::new(DefaultHttpClient::default());
    let config = OAuthClientConfig {
        client_metadata: AtprotoLocalhostClientMetadata {
            redirect_uris: Some(vec![format!(
                //This must match the endpoint you use the callback function
                "http://{host}:{port}/oauth/callback"
            )]),
            scopes: Some(vec![
                Scope::Known(KnownScope::Atproto),
                Scope::Known(KnownScope::TransitionGeneric),
            ]),
        },
        keys: None,
        resolver: OAuthResolverConfig {
            did_resolver: CommonDidResolver::new(CommonDidResolverConfig {
                plc_directory_url: DEFAULT_PLC_DIRECTORY_URL.to_string(),
                http_client: http_client.clone(),
            }),
            handle_resolver: AtprotoHandleResolver::new(AtprotoHandleResolverConfig {
                dns_txt_resolver: HickoryDnsTxtResolver::default(),
                http_client: http_client.clone(),
            }),
            authorization_server_metadata: Default::default(),
            protected_resource_metadata: Default::default(),
        },
        state_store: DbStateStore::new(pool.clone()),
        session_store: DbSessionStore::new(pool.clone()),
    };
    let client = Arc::new(OAuthClient::new(config).expect("failed to create OAuth client"));
    //Spawns the ingester that listens for other's Statusphere updates
    let ingester_pool = pool.clone();
    tokio::spawn(async move {
        start_ingester(ingester_pool).await;
    });
    log::info!("starting HTTP server at http://{host}:{port}");
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(client.clone()))
            .app_data(web::ThinData(pool.clone()))
            .app_data(web::Data::new(handle_resolver.clone()))
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    //TODO will need to set to true in production
                    .cookie_secure(false)
                    // customize session and cookie expiration
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(cookie::time::Duration::days(14)),
                    )
                    .build(),
            )
            .service(Files::new("/css", "public/css").show_files_listing())
            .service(oauth_callback)
            .service(login)
            .service(login_post)
            .service(logout)
            .service(home)
            .service(create_card)
            .service(delete_card)
            .service(put_card)
            .service(clone_stack)
            .service(create_stack)
            .service(create_stack_page)
            .service(delete_stack)
            .service(edit_stack_page)
            .service(put_stack)
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
