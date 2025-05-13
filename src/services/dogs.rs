
use bson::{doc, oid::ObjectId, Bson};
use futures::StreamExt;

use crate::{app_errors::errors::AppError, 
            models::dog_model::{Dog, DogUpdateRequest}};

//use mongodb::Database; 
use crate::services::db::AppDatabase;






    // ---------------
    // CRUD FOR Dog
    // ---------------

    // CREATE for Dog
    pub async fn create_dog(db: &AppDatabase, dog: Dog) -> Result<Dog, AppError> {

         // Execute operation in the DB
        let dog_collection = db.get_dogs_collection();

        let result = dog_collection
            .insert_one(&dog)
            .await?;

         //  evaluate result and return 
        match result.inserted_id {
            Bson::ObjectId(_oid) =>  Ok(dog),  
            other => Err(AppError::DatabaseError(format!("Failed to Create new Dog: {:?}", other))),
        }
    }

    // READ for Dog
    pub async fn read_dogs(db: &AppDatabase) ->  Result<Vec<Dog>, AppError> {

         // Execute operation in the DB
        let dog_collection = db.get_dogs_collection();

        let mut result_cursor = dog_collection
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
     pub async fn read_dog(db: &AppDatabase, dog_id: &str) -> Result<Dog , AppError> {

        //let obj_id = ObjectId::parse_str(dog_id).expect("Failed parsing Dog id.");
        // Verify/Parse received ID 
        let obj_id = match ObjectId::parse_str(dog_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };
        // Create query filter
        let filter = doc! { "_id": obj_id};

         // Execute operation in the DB
        let dog_collection = db.get_dogs_collection();

        let result = dog_collection
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
    pub async fn update_dog(db: &AppDatabase, dog_id: &str, dog_update: DogUpdateRequest) ->  Result<String, AppError>  {

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
        let dog_collection = db.get_dogs_collection();

        let result = dog_collection
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
    pub async fn delete_dog  (db: &AppDatabase, dog_id: &str) -> Result<String, AppError> {

        // Verify/Parse received ID 
        //let obj_id = ObjectId::parse_str(dog_id).expect("Failed to parse booking_id"); 
        let obj_id = match ObjectId::parse_str(dog_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };
        
        // create query filter
        let filter = doc! { "_id": obj_id, };

        // Execute operation in the DB
        let dog_collection = db.get_dogs_collection();
        
        let result = dog_collection
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

