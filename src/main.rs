use log::{ info,  LevelFilter};  // A lightweight logging facade for Rust
use env_logger::{Builder, Target};  // Builder to configure logging programatically; Target to choose output


use actix_web::{get, App, web, HttpResponse, HttpServer, Responder};
use routes::{booking_routes::{create_booking, delete_booking, list_booking, list_bookings, update_booking}, 
                 dog_routes::{create_dog, delete_dog, list_dog, list_dogs, update_dog}, 
                 owner_routes::{create_owner, delete_owner, list_owner, list_owners, update_owner}, 
                 sitter_routes::{create_sitter, delete_sitter, list_sitter, list_sitters, update_sitter}};


mod services;
mod models;
mod routes;
mod json_response;
mod app_errors;


//  use environment variables defined in a .env file with the help of the dotenv crate. 
//  This is commonly used in combination with the standard std::env module to access environment variables at runtime.
use dotenv::dotenv;
//use std::env;

#[get("/")]
async fn hello() -> impl Responder{ 
    HttpResponse::Ok().body("Hello from App Root /")
}

fn set_logger() {
    let mut builder = Builder::new();        // Setup config
    builder.target(Target::Stdout);                  // Set output to stdout
    builder.filter_level(LevelFilter::max());        // Choose level
    builder.init();                                  // Register global logger
} // builder is dropped here — but the logger is now active globally


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok(); // Load variables from `.env` into the environment
    set_logger();  // set and init logger


    let address = "localhost";
    let port = 8080;

    let db = services::db::AppDatabase::init().await;
    let db_data = web::Data::new(db);        // type: web::Data<service::db::AppDatabase>

    info!("Starting server at {} , port: {}", address, port);
    HttpServer::new(move || App::new()
        .app_data(db_data.clone())     // register it here 
        .service(create_owner)
        .service(list_owners)
        .service(list_owner)
        .service(update_owner)
        .service(delete_owner)
        .service(create_dog)
        .service(list_dogs)
        .service(list_dog)
        .service(update_dog)
        .service(delete_dog)
        .service(create_booking)
        .service(list_bookings)
        .service(list_booking)
        .service(update_booking)
        .service(delete_booking)
        .service(create_sitter)
        .service(list_sitters)
        .service(list_sitter)
        .service(update_sitter)
        .service(delete_sitter)
        )
        .bind((address, port))?
        .run()
        .await

}
