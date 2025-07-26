#[macro_use]
pub mod model;
pub mod api;
pub mod schema;
use crate::api::routes::user::{create_user, get_user_by_id, get_users};
use actix_web::{middleware, web, App, HttpServer};
use diesel::{prelude::*, r2d2};

type DbPool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<SqliteConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be valid path to SQLite DB file")
}

// fn create_user(conn: &mut SqliteConnection, nu: &NewUser) -> QueryResult<User> {
//     use crate::schema::USER;
//
//     diesel::insert_into(USER::table)
//         .values(nu)
//         .returning(User::as_returning())
//         .get_result(conn)
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let pool = initialize_db_pool();
    log::info!("starting HTTP server at http://localhost:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(get_users)
            .service(create_user)
            .service(get_user_by_id)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
