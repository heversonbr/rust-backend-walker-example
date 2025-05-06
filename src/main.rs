use actix_web::{get, App, web::Data, HttpResponse, HttpServer, Responder};
use routes::{booking_routes::{create_booking, delete_booking, list_booking, list_bookings, update_booking}, dog_routes::{create_dog, delete_dog, list_dog, list_dogs, update_dog}, owner_routes::{create_owner, delete_owner, list_owner, list_owners, update_owner}};
use services::db::Database;

mod services;
mod models;
mod routes;


//  use environment variables defined in a .env file with the help of the dotenv crate. 
//  This is commonly used in combination with the standard std::env module to access environment variables at runtime.
use dotenv::dotenv;
//use std::env;

#[get("/")]
async fn hello() -> impl Responder{ 
    HttpResponse::Ok().body("Hello from App Root /")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok(); // Load variables from `.env` into the environment
 
    let address = "localhost";
    let port = 8080;

    let db = Database::init().await;
    let db_data = Data::new(db);

    println!("Starting server at {} , port: {}", address, port);
    HttpServer::new(move || App::new()
        .app_data(db_data.clone())
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
            )
            .bind((address, port))?
            .run()
            .await

}
