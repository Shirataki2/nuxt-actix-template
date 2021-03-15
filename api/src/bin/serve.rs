#[macro_use]
extern crate log;

// use actix_redis::RedisSession;
use actix_session::CookieSession;
use actix_web::{get, middleware::Logger, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use dotenv::dotenv;
use sqlx::PgPool;
use std::{env, ops::Deref};

use discord_api::{
    controller::set_routes,
    data::{DiscordOauthProviderBuilder, DiscordOauthScope, OauthClient, HttpClient},
};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("it works!")
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    // ** LOAD CONFIG ** //

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    let pool = PgPool::builder()
        .max_size(25)
        .build(&database_url)
        .await
        .expect("Failed to connect to database");

    let host = env::var("HOST").unwrap_or(String::from("0.0.0.0"));
    let port = env::var("PORT").unwrap_or(String::from("4040"));

    // let redis_host = env::var("REDIS_HOST").expect("REDIS_HOST is not set.");
    let session_key = env::var("SESSION_KEY").expect("SESSION_KEY is not set");

    let client_id = env::var("DISCORD_CLIENT_ID").expect("DISCORD_CLIENT_ID is not set");
    let client_secret =
        env::var("DISCORD_CLIENT_SECRET").expect("DISCORD_CLIENT_SECRET is not set");
    let redirect_url = env::var("DISCORD_REDIRECT_URL").expect("DISCORD_REDIRECT_URL is not set");

    // ** END OF LOAD CONFIG ** //

    use DiscordOauthScope::*;

    let provider = DiscordOauthProviderBuilder::default()
        .client_id(client_id)
        .client_secret(client_secret)
        .scopes(vec![Identify, Guilds])
        .redirect_url(redirect_url)
        .build()
        .expect("Failed to build oauth provider");

    let oauth = OauthClient::new(provider);

    let client = HttpClient::new();

    info!("Service started on http://{}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(oauth.clone())
            .data(client.clone())
            .wrap(Logger::default())
            // .wrap(RedisSession::new(&redis_host, session_key.deref().as_bytes()))
            .wrap(CookieSession::signed(session_key.deref().as_bytes()).secure(false))
            .service(index)
            .configure(set_routes)
    })
    .bind(format!("{}:{}", host, port))
    .expect("Failed to bind server")
    .run()
    .await?;

    Ok(())
}
