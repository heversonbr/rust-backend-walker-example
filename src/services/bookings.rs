
use futures::stream::StreamExt;
use chrono::Utc;
use std::time::SystemTime;
use mongodb::bson::{Bson,doc, oid::ObjectId,DateTime};
use crate::{app_errors::errors::AppError, 
            models::booking_model::{Booking, BookingUpdateRequest}};
//use mongodb::Database; 
use crate::services::db::AppDatabase;


    // -----------------
    // CRUD FOR Booking
    // -----------------
    // CREATE for Booking
    pub async fn create_booking(db: &AppDatabase, booking: Booking) -> Result<Booking, AppError> {
        // REF: mongodb insert_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/insertOne/
    
            // Execute operation in the DB
            let booking_collection = db.get_bookings_collection();

            let result = booking_collection
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
        pub async fn read_bookings(db: &AppDatabase) ->  Result<Vec<Booking>, AppError> {
            // REF: find multiple documents -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/find/
            // The find() method returns a Cursor type, which you can iterate through to retrieve individual documents/
            
            // Execute operation in the DB
            let booking_collection = db.get_bookings_collection();

            let mut result_cursor = booking_collection
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
        pub async fn read_booking(db: &AppDatabase, booking_id: &str) -> Result<Booking , AppError > {
    
            // Verify/Parse received ID 
            // let obj_id = ObjectId::parse_str(booking_id).expect("Failed parsing Booking id.");
            let obj_id = match ObjectId::parse_str(booking_id) {
                Ok(id) => id,
                Err(_) => return Err(AppError::InvalidId),
            };
    
            // Create query filter
            let filter = doc! { "_id": obj_id};
            
            // Execute operation in the DB
            let booking_collection = db.get_bookings_collection();

            let result = booking_collection
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
        pub async fn update_booking(db: &AppDatabase, booking_id: &str, booking_update: BookingUpdateRequest) ->   Result<String, AppError>{
    
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
            let booking_collection = db.get_bookings_collection();

            let result = booking_collection
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
        pub async fn delete_booking(db: &AppDatabase, booking_id: &str) -> Result<String, AppError>{
            // REF: delete_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/deleteOne/
           
            //let obj_id = ObjectId::parse_str(booking_id).expect("Failed to parse booking_id");   // parse ObjectId
            // Verify/Parse received ID 
            let obj_id = match ObjectId::parse_str(booking_id) {
                Ok(id) => id,
                Err(_) => return Err(AppError::InvalidId),
            };
    
            // create query filter
            let filter = doc! { "_id": obj_id, }; // query filter
    
            // Execute operation in the DB
            let booking_collection = db.get_bookings_collection();

            let result = booking_collection
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
    