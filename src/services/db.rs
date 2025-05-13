use crate::models::{booking_model::Booking, 
                     dog_model::Dog, 
                     owner_model::Owner, 
                     sitter_model::Sitter};

use log::{info,error};
use mongodb::{Client, Collection};
use std::{env, process};




// Remind: Clean and idiomatic approach in Rust:
// • The DB layer (services/db.rs) stays focused purely on DB logic and returns Result<T, mongodb::error::Error>.
// • The routing layer handles interpreting DB errors and mapping them into proper HTTP responses (e.g., 404 Not Found, 500 Internal Server Error).
// That separation of concerns is a great design.

#[allow(dead_code)]
pub struct AppDatabase {
    booking_collection: Collection<Booking>,
    dog_collection: Collection<Dog>,
    owner_collection: Collection<Owner>,
    sitter_collection: Collection<Sitter>,
}

impl AppDatabase {

    pub async fn init() -> Self {

        // set URI string to connect into the database
        let uri = match env::var("MONGODB_URI") {
            Ok(v) => v.to_string(),
            Err(_) => "mongodb://localhost:27017/?directConnection=true".to_string(),
        };
        info!("Initializing database connection in : {} ...", uri);

        //let client = Client::with_uri_str(uri).await.unwrap();
        // Instantiate DB client 
        let client = match  Client::with_uri_str(uri).await {
            Ok(client) => client,
            Err(error) => {
                error!("Database initialization failed: {}", error);
                process::exit(1) // Exit with error code
            }
        };

        // Gets a handle to a database specified by name
        // we dont need to use match here This does not verify the DB exists.
		// The database will only be created (or an error triggered) when you actually perform an operation, like inserting or querying.
        let db = client.database("dog_walking");

        // set collections 
        let booking_collection: Collection<Booking> = db.collection("booking");
        let owner_collection: Collection<Owner> = db.collection("owner");
        let dog_collection: Collection<Dog> = db.collection("dog");
        let sitter_collection: Collection<Sitter> = db.collection("sitter");


        AppDatabase {
            booking_collection,
            dog_collection, 
            owner_collection,
            sitter_collection,
        }
    }


    pub fn get_owners_collection(&self) -> &Collection<Owner> {
        &self.owner_collection
    }

    pub fn get_dogs_collection(&self) -> &Collection<Dog> {
        &self.dog_collection
    }

    pub fn get_bookings_collection(&self) -> &Collection<Booking> {
        &self.booking_collection
    }

    pub fn get_sitters_collection(&self) -> &Collection<Sitter> {
        &self.sitter_collection
    }
    


}
