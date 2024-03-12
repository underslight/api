pub mod routes;
pub mod error;
pub mod prelude;

use actix_cors::Cors;
use actix_session::{config::{CookieContentSecurity, PersistentSession, SessionLifecycle}, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::{Key, SameSite}, web, App, HttpServer};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Database;
use surrealdb::Surreal;
use routes::{auth, health};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    // Connects to the database
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await.unwrap();

    // Authenticates the DB connection
    db.signin(Database {
        namespace: "alpha",
        database: "auth",
        username: "dev",
        password: "ved",
    }) 
    .await
    .unwrap();

    let store_key = Key::generate();
    
    // Creates the API server
    HttpServer::new(move || {
        App::new()

            // Adds the database connection to the state
            .app_data(web::Data::new(db.clone()))

            // Adds the session management middleware
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    store_key.clone().into()
                )
                .cookie_content_security(CookieContentSecurity::Private)
                .cookie_same_site(SameSite::None)
                .session_lifecycle(SessionLifecycle::PersistentSession(PersistentSession::default()))
                .build()
            )

            // Sets up CORS policy
            .wrap(
                Cors::permissive()
            )

            // Registers the actual API
            .service(
                web::scope("/api/v1")
                    .service(auth::scope())
                    .service(health::health)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


/* API Routes:
 * 
 * /api/v1/auth/authenticate/EmailPassword
 * /api/v1/auth/register    /EmailPassword
 * /api/v1/auth/add         /EmailPassword
 * /api/v1/auth/add         /EmailPassword
 * /api/v1/auth/remove      /mfa/EmailPassword 
 * /api/v1/auth/remove      /mfa/Totp 
 */