
use bson::{doc, oid::ObjectId, Bson};
use futures::StreamExt;
use crate::{app_errors::errors::AppError, 
            models::owner_model::{Owner, OwnerUpdateRequest}};

//use mongodb::Database; 
use crate::services::db::AppDatabase;


    
    // ----------------
    // CRUD for Owner
    // ----------------

    // CREATE for Owner: In mongodb, you can insert a document into a collection by calling the insert_one() method on a Collection instance.
    pub async fn create_owner(db: &AppDatabase, owner: Owner) -> Result<Owner, AppError> {
        // REF: mongodb insert_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/insertOne/
        // https://docs.rs/mongodb/3.2.3/mongodb/results/struct.InsertOneResult.html

        // Execute operation in the DB
        let owner_collection =  db.get_owners_collection();

        let result = owner_collection
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
    pub async fn read_owners(db: &AppDatabase) ->  Result<Vec<Owner>, AppError> {
        // REF: find multiple documents -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/find/
        // The find() method returns a Cursor type, which you can iterate through to retrieve individual documents/

         // Execute operation in the DB
         let owner_collection = db.get_owners_collection();

        let mut result_cursor = owner_collection
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
    pub async fn read_owner(db: &AppDatabase, owner_id: &str) -> Result<Owner , AppError > {

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
        let owner_collection = db.get_owners_collection();
        let result = owner_collection
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
    pub async fn update_owner(db: &AppDatabase, owner_id: &str, owner_update: OwnerUpdateRequest) ->  Result<String, AppError> {

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
        let owner_collection = db.get_owners_collection();
        let result = owner_collection
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
    pub async fn delete_owner(db: &AppDatabase, owner_id: &str) -> Result<String, AppError> {
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
        let owner_collection = db.get_owners_collection();
        let result = owner_collection
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
    