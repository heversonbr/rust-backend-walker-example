use bson::{doc, oid::ObjectId, Bson};
use futures::StreamExt;
use crate::{app_errors::errors::AppError, 
            models::sitter_model::{Sitter, SitterUpdateRequest}};
//use mongodb::Database; 
use crate::services::db::AppDatabase;


    // ----------------
    // CRUD for Sitter
    // ----------------


    // CREATE for Sitter: In mongodb, you can insert a document into a collection by calling the insert_one() method on a Collection instance.
    pub async fn create_sitter(db: &AppDatabase, sitter: Sitter) -> Result<Sitter, AppError> {
        // REF: mongodb insert_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/insertOne/
 
        // Execute operation in the DB
        let sitter_collection = db.get_sitters_collection();

        let result = sitter_collection
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
     pub async fn read_sitters(db: &AppDatabase) ->  Result<Vec<Sitter>, AppError> {
         // REF: find multiple documents -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/find/
         // The find() method returns a Cursor type, which you can iterate through to retrieve individual documents/
        
        // Execute operation in the DB
        let sitter_collection = db.get_sitters_collection();

        let mut result_cursor = sitter_collection
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
     pub async fn read_sitter(db: &AppDatabase, sitter_id: &str) -> Result<Sitter , AppError > {
 
        //let obj_id = ObjectId::parse_str(sitter_id).expect("Failed parsing sitter id.");
        // Verify/Parse received ID 
        let obj_id = match ObjectId::parse_str(sitter_id) {
            Ok(id) => id,
            Err(_) => return Err(AppError::InvalidId),
        };

        // Create query filter
        let filter = doc! { "_id": obj_id};

        // Execute operation in the DB
        let sitter_collection = db.get_sitters_collection();

        let result = sitter_collection
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
    pub async fn update_sitter(db: &AppDatabase, sitter_id: &str, sitter_update: SitterUpdateRequest) ->  Result<String, AppError> {
 
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
        let sitter_collection = db.get_sitters_collection();

        let result = sitter_collection
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
    pub async fn delete_sitter(db: &AppDatabase, sitter_id: &str) -> Result<String, AppError> {
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
        let sitter_collection = db.get_sitters_collection();
        
        let result = sitter_collection
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

    