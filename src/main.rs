#[macro_use]
pub mod model;
pub mod api;
pub mod schema;
pub mod settings;
use crate::api::auth::bearer_validator;
use crate::api::routes::event::{
    create_event, delete_event, get_event_by_id, get_events, update_event,
};
use crate::api::routes::login::login;
use crate::api::routes::message::{create_message, delete_message, get_messages, update_message};
use crate::api::routes::user::{create_user, get_me, get_user_by_id};
use crate::settings::get_settings;
use actix_web::{middleware, web, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;
use diesel::{prelude::*, r2d2};

type DbPool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<SqliteConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let settings = get_settings();
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let pool = initialize_db_pool();
    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(settings.clone()))
            .wrap(middleware::Logger::default())
            .service(login)
            .service(create_user)
            .service(
                web::scope("")
                    .wrap(HttpAuthentication::bearer(bearer_validator))
                    .service(get_me)
                    .service(get_user_by_id)
                    .service(get_events)
                    .service(create_event)
                    .service(get_event_by_id)
                    .service(update_event)
                    .service(delete_event)
                    .service(get_messages)
                    .service(update_message)
                    .service(delete_message)
                    .service(create_message),
            )
    })
    .bind(("127.0.0.1", settings.port))?
    .workers(settings.num_workers)
    .run()
    .await
}
