use crate::{app_errors::errors::AppError, 
            models::{booking_model::{Booking, BookingUpdateRequest}, 
                     dog_model::{Dog, DogUpdateRequest}, 
                     owner_model::{Owner, OwnerUpdateRequest}, 
                     sitter_model::{Sitter, SitterUpdateRequest}}
        };

use bson::Bson;
use log::{info, error};
use mongodb::{bson::{doc, oid::ObjectId,DateTime}, Client, Collection};

use std::{env, process};   // process to quit the process in case of important fail
use futures::stream::StreamExt;
use chrono::Utc;
use std::time::SystemTime;




// Remind: Clean and idiomatic approach in Rust:
// • The DB layer (services/db.rs) stays focused purely on DB logic and returns Result<T, mongodb::error::Error>.
// • The routing layer handles interpreting DB errors and mapping them into proper HTTP responses (e.g., 404 Not Found, 500 Internal Server Error).
// That separation of concerns is a great design.

#[allow(dead_code)]
pub struct Database {
    booking_collection: Collection<Booking>,
    dog_collection: Collection<Dog>,
    owner_collection: Collection<Owner>,
    sitter_collection: Collection<Sitter>,
}

impl Database {

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


        Database {
            booking_collection,
            dog_collection, 
            owner_collection,
            sitter_collection,
        }
    }
    
    // CRUD operations: CREATE READ UPDATE DELETE 
    
    // ----------------
    // CRUD for Owner
    // ----------------

    // CREATE for Owner: In mongodb, you can insert a document into a collection by calling the insert_one() method on a Collection instance.
    pub async fn create_owner(&self, owner: Owner) -> Result<Owner, AppError> {
        // REF: mongodb insert_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/insertOne/
        // https://docs.rs/mongodb/3.2.3/mongodb/results/struct.InsertOneResult.html

        // Execute operation in the DB
        let result = self
            .owner_collection
            .insert_one(&owner)
            .await?;

        //  evaluate result and return 
        match result.inserted_id {
            Bson::ObjectId(_oid) =>  Ok(owner),  // some id received back 
            other => Err(AppError::DatabaseError(format!("Failed to Create new Owner: {:?}", other))),
        }
    }

    // READ for Owner: 
    // In mongodb, you can query for multiple documents in a collection by calling the 'find()' method on a Collection instance.
    // 1) READ ALL: 
    pub async fn read_owners(&self) ->  Result<Vec<Owner>, AppError> {
        // REF: find multiple documents -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/find/
        // The find() method returns a Cursor type, which you can iterate through to retrieve individual documents/

         // Execute operation in the DB
        let mut result_cursor = self
            .owner_collection
            .find(doc!{})
            .await?;
            //.ok()
            //.expect("Error while reading owners from database.");
        
        // Vector for results
        let mut vec_of_owners = Vec::<Owner>::new();

        while let Some(result) = result_cursor.next().await {
            match result {
                Ok(owner_entry) => vec_of_owners.push(owner_entry),
                Err(e) => return Err(AppError::DatabaseError(format!("Error reading Owner entries from DB: {}" , e.to_string()))),
            }
        }
        Ok(vec_of_owners)
    }

    // READ single owner
    // find_one: If a document matches the filter criteria, the method returns a Result<Option<T>> type with a value of Some. 
    //           If no documents match the filter criteria, find_one() returns a Result<Option<T>> type with a value of None.
    pub async fn read_owner(&self, owner_id: &str) -> Result<Owner , AppError > {

        // Verify/Parse received ID 
        //let obj_id = ObjectId::parse_str(owner_id).expect("Failed parsing owner id.");
        //let obj_id = ObjectId::parse_str(owner_id)?;
        let obj_id = match ObjectId::parse_str(owner_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };

        // Create query filter
        let filter = doc! { "_id": obj_id};

        // Execute operation in the DB
        let result = self
            .owner_collection
            .find_one(filter)
            .await?;
            //.ok()
            //.expect("Failed reading owner Collection from Database.");
        
       //  evaluate result and return     
        match result {
            Some(owner) => Ok(owner) ,
            None => Err(AppError::NotFound),
        }
        //Ok(result.unwrap())

    }

    // UPDATE for Owner:
    pub async fn update_owner(&self, owner_id: &str, owner_update: OwnerUpdateRequest) ->  Result<String, AppError> {

        // Verify/Parse received ID 
        let obj_id = match ObjectId::parse_str(owner_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };
        
        // Select fields sent in the UpdateRequest
        let mut update_fields = doc! {};
        if let Some(name) = owner_update.name {update_fields.insert("name", name); }
        if let Some(email) = owner_update.email {update_fields.insert("email", email);}
        if let Some(phone) = owner_update.phone {update_fields.insert("phone", phone);}
        if let Some(address) = owner_update.address {update_fields.insert("address", address);}
        // Check for empty request
        if update_fields.is_empty() { 
            return Err(AppError::ParseError("No Fields provided to Updated".to_string())); 
        } // this empty field shouldnt happen anymore because I added the request validation in the router
    
        // Prepare filter and update 
        let filter = doc! { "_id": obj_id };
        let update =  doc! { "$set": update_fields };

        // Execute operation in the DB
        let result = self
            .owner_collection
            .update_one(filter, update)
            .await;

        // evaluate result and return, if update ok, return id of the updated doc
        match result {
            Ok(_result) => Ok(obj_id.to_hex()),   
            Err(e) => Err(AppError::DatabaseError(format!("Failed to Update Owner: {}", e))),
        }

    }

    // DELETE for Owner: 
    // In mongodb, you can delete a document from a collection by calling the delete_one() method on a Collection instance.
    pub async fn delete_owner(&self, owner_id: &str) -> Result<String, AppError> {
        // REF: delete_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/deleteOne/
       
        // Verify/Parse received ID 
        //let obj_id = ObjectId::parse_str(owner_id).expect("Update owner: failed parsing owner id.");
        let obj_id = match ObjectId::parse_str(owner_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };

        // create query filter
        let filter = doc! { "_id": obj_id, };

        // Execute operation at DB
        let result = self
                .owner_collection
                .delete_one(filter)
                .await;
                //.ok()
                //.expect("Error deleting booking");

        //Ok(result.deleted_count>0)
        // evaluate result and return, if delete ok, return id of the updated doc
        match result {
            Ok(delete_result) if delete_result.deleted_count >= 1 => Ok(obj_id.to_hex()),
            Ok(delete_result) if delete_result.deleted_count == 0 => Err(AppError::NotFound),
            Ok(_) => Err(AppError::InternalError),
            Err(db_error) => Err(AppError::DatabaseError(format!("Failed to Delete Owner: {}", db_error))),
        }
    }
    
    
    // ---------------
    // CRUD FOR Dog
    // ---------------

    // CREATE for Dog
    pub async fn create_dog(&self, dog: Dog) -> Result<Dog, AppError> {

        // Execute operation in the DB
        let result = self
            .dog_collection
            .insert_one(&dog)
            .await?;

         //  evaluate result and return 
        match result.inserted_id {
            Bson::ObjectId(_oid) =>  Ok(dog),  
            other => Err(AppError::DatabaseError(format!("Failed to Create new Dog: {:?}", other))),
        }
    }

    // READ for Dog
    pub async fn read_dogs(&self) ->  Result<Vec<Dog>, AppError> {

        // Execute operation in the DB
        let mut result_cursor = self
            .dog_collection
            .find(doc!{})
            .await?;
        
        let mut vec_of_dogs = Vec::<Dog>::new();

        while let Some(result) = result_cursor.next().await {
            match result {
                Ok(dog_entry) => vec_of_dogs.push(dog_entry),
                Err(e) => return Err(AppError::DatabaseError(format!("Error reading Dog entries from DB: {}" , e.to_string()))),
            }
        }
        Ok(vec_of_dogs)
    }

     // READ single dog
     pub async fn read_dog(&self, dog_id: &str) -> Result<Dog , AppError> {

        //let obj_id = ObjectId::parse_str(dog_id).expect("Failed parsing Dog id.");
        // Verify/Parse received ID 
        let obj_id = match ObjectId::parse_str(dog_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };
        // Create query filter
        let filter = doc! { "_id": obj_id};

        // Execute operation in the DB
        let result = self
            .dog_collection
            .find_one(filter)
            .await?;
            //.ok()
            //.expect("Failed reading dog Collection from Database.");

       // match result and return 
       //match result {
       // Some(dog) => Some(dog) ,
       // _ => None,
       //}
        match result {
            Some(dog) => Ok(dog) ,
            None => Err(AppError::NotFound),
        }

    }

    // UPDATE for Dog:
    pub async fn update_dog(&self, dog_id: &str, dog_update: DogUpdateRequest) ->  Result<String, AppError>  {

        //let obj_id = ObjectId::parse_str(dog_id).expect("Update Dog: failed parsing Dog id.");
         // Verify/Parse received ID 
        let obj_id = match ObjectId::parse_str(dog_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };
        
         // Select fields sent in the UpdateRequest
        let mut update_fields = doc! {};
        // Select fields sent in the DogUpdateRequest 
        if let Some(owner) = dog_update.owner{ 
            // if some owner value was sent in the request, 
            // we need to validate it before updating the received value
            let received_owner  = match ObjectId::parse_str(owner) {
                Ok(id) => id,
                Err(e) => return Err(AppError::DatabaseError(format!("Update Failed: invalid owner ID: {}", e ))),
            };
            update_fields.insert("owner", received_owner); 
        }
        if let Some(name) =  dog_update.name { update_fields.insert("name", name);}
        if let Some(age) =       dog_update.age  { update_fields.insert("age", age as u32);}
        if let Some(breed) = dog_update.breed{ update_fields.insert("breed", breed);}
        // Check for empty request
        if update_fields.is_empty() { 
            //return Ok(false); 
            return Err(AppError::ParseError("No Fields provided to Updated".to_string())); 
        } // this empty field shouldnt happen anymore because I added the request validation in the router
    
        // Prepare filter and update 
        let filter = doc! { "_id": obj_id };
        let update =  doc! { "$set": update_fields };
       // Execute operation in the DB
        let result = self
            .dog_collection
            .update_one(filter, update)
            .await;
            //.ok()
            //.expect("Error updating Dog.");

        // evaluate result and return, if update ok, return id of the updated doc
        match result {
            Ok(_result) => Ok(obj_id.to_hex()),   
            Err(e) => Err(AppError::DatabaseError(format!("Failed to Update Dog: {}", e))),
        }
    }

    // DELETE for Dog
    pub async fn delete_dog  (&self, dog_id: &str) -> Result<String, AppError> {

        // Verify/Parse received ID 
        //let obj_id = ObjectId::parse_str(dog_id).expect("Failed to parse booking_id"); 
        let obj_id = match ObjectId::parse_str(dog_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };
        
        // create query filter
        let filter = doc! { "_id": obj_id, };

        // Execute operation at DB
        let result = self
                .dog_collection
                .delete_one(filter)
                .await;
                //.ok()
                //.expect("Error deleting Dog");

        // evaluate result and return, if delete ok, return id of the updated doc    
        //Ok(result.deleted_count>0)
        match result {
            Ok(delete_result) if delete_result.deleted_count >= 1 => Ok(obj_id.to_hex()),
            Ok(delete_result) if delete_result.deleted_count == 0 => Err(AppError::NotFound),
            Ok(_) => Err(AppError::InternalError),
            Err(db_error) => Err(AppError::DatabaseError(format!("Failed to Delete Dog: {}", db_error))),
        }

    }



    // -----------------
    // CRUD FOR Booking
    // -----------------
    // CREATE for Booking
    pub async fn create_booking(&self, booking: Booking) -> Result<Booking, AppError> {
    // REF: mongodb insert_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/insertOne/

        let result = self
                .booking_collection
                .insert_one(&booking)
                .await?;
                //.ok()
                //.expect("Error creating owner");   

        //Ok(booking)
        //  evaluate result and return 
        match result.inserted_id {
            Bson::ObjectId(_oid) =>  Ok(booking),  // some id received back 
            other => Err(AppError::DatabaseError(format!("Failed to Create new Booking: {:?}", other))),
        }
    }
  
    // READ for Booking
    pub async fn read_bookings(&self) ->  Result<Vec<Booking>, AppError> {
        // REF: find multiple documents -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/find/
        // The find() method returns a Cursor type, which you can iterate through to retrieve individual documents/
        
        // Execute operation in the DB
        let mut result_cursor = self
            .booking_collection
            .find(doc!{})
            .await?;
            //.ok()
            //.expect("Error while reading bookings from database.");

        // Vector for results   
        let mut vec_of_bookings = Vec::<Booking>::new();

        while let Some(result) = result_cursor.next().await {
            match result {
                Ok(booking_entry) => vec_of_bookings.push(booking_entry),
                Err(e) => return Err(AppError::DatabaseError(format!("Error reading Booking entries from DB: {}" , e.to_string()))),
            }
        }
        Ok(vec_of_bookings)
    }

    // READ single booking
    pub async fn read_booking(&self, booking_id: &str) -> Result<Booking , AppError > {

        // Verify/Parse received ID 
        // let obj_id = ObjectId::parse_str(booking_id).expect("Failed parsing Booking id.");
        let obj_id = match ObjectId::parse_str(booking_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };

        // Create query filter
        let filter = doc! { "_id": obj_id};
        
        // Execute operation in the DB
        let result = self
            .booking_collection
            .find_one(filter)
            .await?;
            //.ok()
            //.expect("Failed reading booking Collection from Database.");

       // match result and return 
       match result {
            Some(booking) => Ok(booking),
            None => Err(AppError::NotFound),
       }

    }

    // UPDATE for Booking
    pub async fn update_booking(&self, booking_id: &str, booking_update: BookingUpdateRequest) ->   Result<String, AppError>{

        //let booking_obj_id = ObjectId::parse_str(booking_id).expect("Update Booking: failed parsing Booking id.");
        // Verify/Parse received ID 
        let obj_id = match ObjectId::parse_str(booking_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };

        // Select fields sent in the UpdateRequest
        let mut update_fields = doc! {};
       
        // Select fields sent in the BookingUpdateRequest
        // REMEMBER : Validate the fields sent to update when required
        if let Some(owner_str) =   booking_update.owner {
             // if some owner value was sent in the request, 
            // we need to validate it before updating the received value
           // let received_ownder_id = ObjectId::parse_str(owner_str).expect("Update Booking: failed parsing owner id.");
           let received_owner  = match ObjectId::parse_str(owner_str) {  // here we validate the received owner id
                Ok(id) => id,
                Err(e) => return Err(AppError::DatabaseError(format!("Update Failed: invalid owner ID: {}", e ))),
            };
            update_fields.insert("owner", received_owner); 
        }
        // if received date, validate before inserting
        if let Some(start_time) = booking_update.start_time {
            //let chrono_datetime: SystemTime = 
            //        chrono::DateTime::parse_from_rfc3339(&start_time)
            //        .map_err(|err| format!("Failed to parse start time: {} ", err)).expect("Failed to parsing start time")
            //        .with_timezone(&Utc).into();
            let chrono_datetime: SystemTime = match chrono::DateTime::parse_from_rfc3339(&start_time) {
                Ok(dt) => dt.with_timezone(&Utc).into(),
                Err(err) if err.to_string().contains("expected date") => {
                    return Err(AppError::ParseError("Start time must include a date".into()))
                }
                Err(err) => {
                    return Err(AppError::ParseError(format!("Failed to parse start time: {}", err)))
                }
            };
            let bson_start_time = DateTime::from(chrono_datetime);

            update_fields.insert("start_time", bson_start_time);
        }

        if let Some(duration_minutes) = booking_update.duration_minutes{update_fields.insert("duration_minutes", duration_minutes as u32);}

        if let Some(cancelled) = booking_update.cancelled {update_fields.insert("cancelled", cancelled);}

        if update_fields.is_empty() { 
            return Err(AppError::ParseError("No Fields provided to Updated".to_string()));  // Or return custom error, no fields 
        }

        // Prepare filter and update 
        let filter = doc! { "_id": obj_id };
        let update =  doc! { "$set": update_fields };
       
       // Execute operation in the DB
        let result = self
            .booking_collection
            .update_one(filter, update)
            .await;
            //.ok()
            //.expect("Error updating owners name.");

         // evaluate result and return, if update ok, return id of the updated doc
         match result {
            Ok(_result) => Ok(obj_id.to_hex()),   
            Err(e) => Err(AppError::DatabaseError(format!("Failed to Update Booking: {}", e))),
        }
    }

    // DELETE for Booking
    pub async fn delete_booking(&self, booking_id: &str) -> Result<String, AppError>{
        // REF: delete_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/deleteOne/
       
        //let obj_id = ObjectId::parse_str(booking_id).expect("Failed to parse booking_id");   // parse ObjectId
        // Verify/Parse received ID 
        let obj_id = match ObjectId::parse_str(booking_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };

        // create query filter
        let filter = doc! { "_id": obj_id, }; // query filter

        // Execute operation at DB
        let result = self
                .booking_collection
                .delete_one(filter)
                .await;
                //.ok()
                //.expect("Error deleting booking");

         // evaluate result and return, if delete ok, return id of the updated doc 
        match result {
            Ok(delete_result) if delete_result.deleted_count >= 1 => Ok(obj_id.to_hex()),
            Ok(delete_result) if delete_result.deleted_count == 0 => Err(AppError::NotFound),
            Ok(_) => Err(AppError::InternalError),
            Err(db_error) => Err(AppError::DatabaseError(format!("Failed to Delete Booking: {}", db_error))),
        }
    }


    // ----------------
    // CRUD for Sitter
    // ----------------

    // CREATE for Sitter: In mongodb, you can insert a document into a collection by calling the insert_one() method on a Collection instance.
    pub async fn create_sitter(&self, sitter: Sitter) -> Result<Sitter, AppError> {
        // REF: mongodb insert_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/insertOne/
 
        // Execute operation in the DB
        let result = self
        .sitter_collection
        .insert_one(&sitter)
        .await?;
        //.ok()
        //.expect("Error creating sitter in the database");   

        //  evaluate result and return 
        match result.inserted_id {
            Bson::ObjectId(_oid) =>  Ok(sitter),  // some id received back 
            other => Err(AppError::DatabaseError(format!("Failed to Create new Owner: {:?}", other))),
        }
 
     }
    
     // READ for Sitter: 
     // In mongodb, you can query for multiple documents in a collection by calling the 'find()' method on a Collection instance.
     // 1) READ ALL: 
     pub async fn read_sitters(&self) ->  Result<Vec<Sitter>, AppError> {
         // REF: find multiple documents -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/find/
         // The find() method returns a Cursor type, which you can iterate through to retrieve individual documents/
        
        // Execute operation in the DB
        let mut result_cursor = self
            .sitter_collection
            .find(doc!{})
            .await?;
            //.ok()
            //.expect("Error while reading sitters from database.");
        
        // Vector for results
        let mut vec_of_sitters = Vec::<Sitter>::new();
 

         while let Some(result) = result_cursor.next().await {
             match result {
                 Ok(sitter_entry) => vec_of_sitters.push(sitter_entry),
                 Err(e) => return Err(AppError::DatabaseError(format!("Error reading Owner entries from DB: {}" , e.to_string()))),
             }
         }
         Ok(vec_of_sitters)
     }
     // READ single sitter
     pub async fn read_sitter(&self, sitter_id: &str) -> Result<Sitter , AppError > {
 
        //let obj_id = ObjectId::parse_str(sitter_id).expect("Failed parsing sitter id.");
        // Verify/Parse received ID 
        let obj_id = match ObjectId::parse_str(sitter_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };

        // Create query filter
        let filter = doc! { "_id": obj_id};

        // Execute operation in the DB
        let result = self
            .sitter_collection
            .find_one(filter)
            .await?;
            //.ok()
            //.expect("Failed reading sitter Collection from Database.");

        // match result and return 
        match result {
            Some(sitter) => Ok(sitter) ,
            None => Err(AppError::NotFound),
        }

     }
 
     // UPDATE for Sitter:
    pub async fn update_sitter(&self, sitter_id: &str, sitter_update: SitterUpdateRequest) ->  Result<String, AppError> {
 
         //let obj_id = ObjectId::parse_str(sitter_id).expect("Update sitter: failed parsing sitter id.");
        // Verify/Parse received ID 
        let obj_id = match ObjectId::parse_str(sitter_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };

        
        // Select fields sent in the UpdateRequest
        let mut update_fields = doc! {};
    
        if let Some(firstname) = sitter_update.firstname {update_fields.insert("firstname", firstname); }
        if let Some(lastname)  = sitter_update.lastname  {update_fields.insert("lastname", lastname); }
        if let Some(gender) = sitter_update.gender {update_fields.insert("gender", gender);}
        if let Some(email) = sitter_update.email {update_fields.insert("email", email);}
        if let Some(phone) = sitter_update.phone {update_fields.insert("phone", phone);}
        if let Some(address) = sitter_update.address {update_fields.insert("address", address);}
 
        if update_fields.is_empty() { 
            return Err(AppError::ParseError("No Fields provided to Updated".to_string())); 
        } // this empty field shouldnt happen anymore because I added the request validation in the router 
        
        // Prepare filter and update 
        let filter = doc! { "_id": obj_id };
        let update =  doc! { "$set": update_fields };
        
        // Execute operation in the DB
        let result = self
            .sitter_collection
            .update_one(filter, update)
            .await;
            //.ok()
            //.expect("Error updating sitters name.");
 
        //Ok(result.matched_count == 1)
        match result {
            Ok(_result) => Ok(obj_id.to_hex()),   
            Err(e) => Err(AppError::DatabaseError(format!("Failed to Update Sitter: {}", e))),
        }


     }
 
     // DELETE for Sitter: 
     // In mongodb, you can delete a document from a collection by calling the delete_one() method on a Collection instance.
    pub async fn delete_sitter(&self, sitter_id: &str) -> Result<String, AppError> {
         // REF: delete_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/deleteOne/
        
        // parse ObjectId
        //let obj_id = ObjectId::parse_str(sitter_id).expect("Failed to parse sitter_id"); 
        // Verify/Parse received ID 
        //let obj_id = ObjectId::parse_str(owner_id).expect("Update owner: failed parsing owner id.");
        let obj_id = match ObjectId::parse_str(sitter_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };

         // Create query filter
         let filter = doc! { "_id": obj_id, };
 
        // Execute operation at DB
        let result = self
                .sitter_collection
                .delete_one(filter)
                .await;
                //.ok()
                //.expect("Error deleting booking");
        
        
        //Ok(result.deleted_count>0)
        // evaluate result and return, if delete ok, return id of the updated doc
        match result {
            Ok(delete_result) if delete_result.deleted_count >= 1 => Ok(obj_id.to_hex()),
            Ok(delete_result) if delete_result.deleted_count == 0 => Err(AppError::NotFound),
            Ok(_) => Err(AppError::InternalError),
            Err(db_error) => Err(AppError::DatabaseError(format!("Failed to Delete Sitter: {}", db_error))),
        }
    }

    

    // read using aggregation
    // REF: mongodb aggregation -> https://www.mongodb.com/docs/drivers/rust/current/fundamentals/aggregation/


}
