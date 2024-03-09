pub mod routes;
pub mod error;
pub mod prelude;

use actix_web::{web, App, HttpServer};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Database;
use surrealdb::Surreal;

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
    
    // Creates the API server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .service(
                web::scope("/api/v1")
                    .service(routes::auth::scope())
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