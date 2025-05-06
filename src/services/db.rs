use crate::models::{booking_model::{Booking, BookingResponse, BookingUpdateRequest}, dog_model::{Dog, DogUpdateRequest}, owner_model::{Owner, OwnerUpdateRequest}};
use mongodb::{bson::{doc, oid::ObjectId, raw::Error,DateTime}, Client, Collection};
use std::env;
use futures::stream::{StreamExt};

use chrono::Utc;
use std::time::SystemTime;

#[allow(dead_code)]
pub struct Database {
    booking_collection: Collection<Booking>,
    dog_collection: Collection<Dog>,
    owner_collection: Collection<Owner>,
}

impl Database {

    pub async fn init() -> Self {
        let uri = match env::var("MONGODB_URI") {
            Ok(v) => v.to_string(),
            Err(_) => "mongodb://localhost:27017/?directConnection=true".to_string(),
        };

        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("dog_walking");

        let booking_collection: Collection<Booking> = db.collection("booking");
        let owner_collection: Collection<Owner> = db.collection("owner");
        let dog_collection: Collection<Dog> = db.collection("dog");

        Database {
            booking_collection,
            dog_collection, 
            owner_collection,
        }
    }
    
    // CRUD operations: CREATE READ UPDATE DELETE 
    // ----------------
    // CRUD for Owner
    // ----------------

    // CREATE for Owner: In mongodb, you can insert a document into a collection by calling the insert_one() method on a Collection instance.
    pub async fn create_owner(&self, owner: Owner) -> Result<Owner, Error> {
       // REF: mongodb insert_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/insertOne/

            let _result = self
            .owner_collection
            .insert_one(&owner)
            .await
            .ok()
            .expect("Error creating owner in the database");   

        
           //Ok(result)
           Ok(owner)

    }
   
    // READ for Owner: 
    // In mongodb, you can query for multiple documents in a collection by calling the 'find()' method on a Collection instance.
    // 1) READ ALL: 
    pub async fn read_owners(&self) ->  Result<Vec<Owner>, Error> {
        // REF: find multiple documents -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/find/
        // The find() method returns a Cursor type, which you can iterate through to retrieve individual documents/

        let mut result_cursor = self
            .owner_collection
            .find(doc!{})
            .await
            .ok()
            .expect("Error while reading owners from database.");

        let mut vec_of_owners = Vec::<Owner>::new();

        while let Some(result) = result_cursor.next().await {
            match result {
                Ok(owner_entry) => vec_of_owners.push(owner_entry),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Ok(vec_of_owners)
    }
    // READ single owner
    pub async fn read_owner(&self, owner_id: &str) -> Option<Owner> {

        let obj_id = ObjectId::parse_str(owner_id).expect("Failed parsing owner id.");

        let filter = doc! { "_id": obj_id};

        let result = self
            .owner_collection
            .find_one(filter)
            .await
            .ok()
            .expect("Failed reading owner Collection from Database.");

       // match result and return 
       match result {
        Some(owner) => Some(owner) ,
        _ => None,
       }

    }


    // UPDATE for Owner:
    pub async fn update_owner(&self, owner_id: &str, owner_update: OwnerUpdateRequest) ->  Result<bool, mongodb::error::Error> {

        let obj_id = ObjectId::parse_str(owner_id).expect("Update owner: failed parsing owner id.");
        let mut update_fields = doc! {};
        // Select fields sent in the UpdateRequest
        if let Some(name) = owner_update.name {update_fields.insert("name", name); }
        if let Some(email) = owner_update.email {update_fields.insert("email", email);}
        if let Some(phone) = owner_update.phone {update_fields.insert("phone", phone);}
        if let Some(address) = owner_update.address {update_fields.insert("address", address);}

        if update_fields.is_empty() { return Ok(false); } // Or return custom error 
    
        let filter = doc! { "_id": obj_id };
        let update =  doc! { "$set": update_fields };
       
        let result = self
            .owner_collection
            .update_one(filter, update)
            .await
            .ok()
            .expect("Error updating owners name.");

        Ok(result.matched_count == 1)
    }

    // DELETE for Owner: 
    // In mongodb, you can delete a document from a collection by calling the delete_one() method on a Collection instance.
    pub async fn delete_owner(&self, owner_id: &str) -> Result<bool, bson::raw::Error> {
        // REF: delete_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/deleteOne/
        // parse ObjectId
        let obj_id = ObjectId::parse_str(owner_id).expect("Failed to parse booking_id"); 
        // query filter
        let filter = doc! { "_id": obj_id, };

        let result = self
                .owner_collection
                .delete_one(filter)
                .await
                .ok()
                .expect("Error deleting booking");
        Ok(result.deleted_count>0)
    }
    
    
    // ---------------
    // CRUD FOR Dog
    // ---------------
    // CREATE for Dog
    pub async fn create_dog(&self, dog: Dog) -> Result<Dog, Error> {
     // REF: mongodb insert_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/insertOne/

        let _result = self
        .dog_collection
        .insert_one(&dog)
        .await
        .ok()
        .expect("Error creating owner");   

        Ok(dog)
    }

    // READ for Dog
    pub async fn read_dogs(&self) ->  Result<Vec<Dog>, Error> {
        // REF: find multiple documents -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/find/
        // The find() method returns a Cursor type, which you can iterate through to retrieve individual documents/
        let mut result_cursor = self
            .dog_collection
            .find(doc!{})
            .await
            .ok()
            .expect("Error while reading dogs from database.");
        let mut vec_of_dogs = Vec::<Dog>::new();
        while let Some(result) = result_cursor.next().await {
            match result {
                Ok(dog_entry) => vec_of_dogs.push(dog_entry),
                Err(e) => eprintln!("Error: {}", e),    //change this
            }
        }
        Ok(vec_of_dogs)
    }

     // READ single dog
     pub async fn read_dog(&self, dog_id: &str) -> Option<Dog> {

        let obj_id = ObjectId::parse_str(dog_id).expect("Failed parsing Dog id.");

        let filter = doc! { "_id": obj_id};
        
        let result = self
            .dog_collection
            .find_one(filter)
            .await
            .ok()
            .expect("Failed reading dog Collection from Database.");

       // match result and return 
       match result {
        Some(dog) => Some(dog) ,
        _ => None,
       }

    }

    // UPDATE for Dog:
    pub async fn update_dog(&self, dog_id: &str, dog_update: DogUpdateRequest) ->  Result<bool, mongodb::error::Error> {

        let obj_id = ObjectId::parse_str(dog_id).expect("Update Dog: failed parsing Dog id.");
        let mut update_fields = doc! {};
        // Select fields sent in the DogUpdateRequest 
        if let Some(owner) = dog_update.owner{ update_fields.insert("owner", owner); }
        if let Some(name) =  dog_update.name { update_fields.insert("name", name);}
        if let Some(age) =       dog_update.age  { update_fields.insert("age", age as u32);}
        if let Some(breed) = dog_update.breed{ update_fields.insert("breed", breed);}

        if update_fields.is_empty() { return Ok(false); } // Or return custom error 
    
        let filter = doc! { "_id": obj_id };
        let update =  doc! { "$set": update_fields };
       
        let result = self
            .dog_collection
            .update_one(filter, update)
            .await
            .ok()
            .expect("Error updating Dog.");

        Ok(result.matched_count == 1)
    }

    // DELETE for Dog
    pub async fn delete_dog  (&self, dog_id: &str) -> Result<bool, bson::raw::Error> {
        // REF: delete_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/deleteOne/
        // parse ObjectId
        let obj_id = ObjectId::parse_str(dog_id).expect("Failed to parse booking_id"); 
        // query filter
        let filter = doc! { "_id": obj_id, };

        let result = self
                .dog_collection
                .delete_one(filter)
                .await
                .ok()
                .expect("Error deleting Dog");
            Ok(result.deleted_count>0)
    }



    // -----------------
    // CRUD FOR Booking
    // -----------------
    // CREATE for Booking
    pub async fn create_booking(&self, booking: Booking) -> Result<Booking, Error> {
    // REF: mongodb insert_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/insertOne/
        let _result = self
                .booking_collection
                .insert_one(&booking)
                .await
                .ok()
                .expect("Error creating owner");   

        Ok(booking)
    }
  
    // READ for Booking
    pub async fn read_bookings(&self) ->  Result<Vec<Booking>, Error> {
        // REF: find multiple documents -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/find/
        // The find() method returns a Cursor type, which you can iterate through to retrieve individual documents/
        let mut result_cursor = self
            .booking_collection
            .find(doc!{})
            .await
            .ok()
            .expect("Error while reading bookings from database.");
        let mut vec_of_bookings = Vec::<Booking>::new();
        while let Some(result) = result_cursor.next().await {
            match result {
                Ok(booking_entry) => vec_of_bookings.push(booking_entry),
                Err(e) => eprintln!("Error: {}", e),    //change this
            }
        }
        Ok(vec_of_bookings)
    }

    // READ single booking
    pub async fn read_booking(&self, booking_id: &str) -> Option<BookingResponse> {

        let obj_id = ObjectId::parse_str(booking_id).expect("Failed parsing Booking id.");

        let filter = doc! { "_id": obj_id};
        
        let result = self
            .booking_collection
            .find_one(filter)
            .await
            .ok()
            .expect("Failed reading booking Collection from Database.");

       // match result and return 
       match result {
        Some(booking) => Some(BookingResponse::from(booking)) ,
        _ => None,
       }

    }

    // UPDATE for Booking
    pub async fn update_booking(&self, booking_id: &str, booking_update: BookingUpdateRequest) ->  Result<bool, mongodb::error::Error> {

        let booking_obj_id = ObjectId::parse_str(booking_id).expect("Update Booking: failed parsing Booking id.");
        let mut update_fields = doc! {};
       
        // Select fields sent in the BookingUpdateRequest
        // update owner's Id if there's one:
        if let Some(owner_str) =   booking_update.owner {
            let obj_ownder_id = ObjectId::parse_str(owner_str).expect("Update Booking: failed parsing owner id.");
            update_fields.insert("owner", obj_ownder_id); 
        }
        // update start_time if there's some:
        if let Some(start_time) = booking_update.start_time {
            let chrono_datatime: SystemTime = 
                    chrono::DateTime::parse_from_rfc3339(&start_time)
                    .map_err(|err| format!("Failed to parse start time: {} ", err)).expect("Failed to parsing start time")
                    .with_timezone(&Utc).into();
            let bson_start_time = DateTime::from(chrono_datatime);
            update_fields.insert("start_time", bson_start_time);
        }
        // update duration_minutes if there's some:
        if let Some(duration_minutes) = booking_update.duration_minutes{update_fields.insert("duration_minutes", duration_minutes as u32);}
        // update canceled field if there's one in thre request:
        if let Some(cancelled) = booking_update.cancelled     {update_fields.insert("cancelled", cancelled);}

        if update_fields.is_empty() {return Ok(false); } // Or return custom error, no fields 
    
        let filter = doc! { "_id": booking_obj_id };
        let update =  doc! { "$set": update_fields };
       
        let result = self
            .booking_collection
            .update_one(filter, update)
            .await
            .ok()
            .expect("Error updating owners name.");

        Ok(result.matched_count == 1)
    }

    // DELETE for Booking
    pub async fn delete_booking(&self, booking_id: &str) -> Result<bool, bson::raw::Error> {
        // REF: delete_one() -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/deleteOne/
       
        let obj_id = ObjectId::parse_str(booking_id).expect("Failed to parse booking_id");   // parse ObjectId
        let filter = doc! { "_id": obj_id, }; // query filter

        let result = self
                .booking_collection
                .delete_one(filter)
                .await
                .ok()
                .expect("Error deleting booking");
            Ok(result.deleted_count>0)
    }




    
    // UPDATE: 
    // In mongodb, you can update a document in a collection by calling the update_one() method on a Collection instance.
    // You can update multiple documents in a collection by calling the update_many() method on a Collection instance.
    // both method return an UpdateResult type. 
    // updating dog's name
    //pub async fn update_dog_name(&self, dog_id: &str, new_name: &str) -> Result<UpdateResult, Error> {
    //    let obj_id = ObjectId::parse_str(dog_id).expect("Update failed parsing new dogs name.");
    //    let filter = doc! { "_id": obj_id };
    //    let update = doc! { "$set": doc! { "name": new_name }};
    //    let result = self.dog_collection
    //        .update_one(filter, update)
    //        .await.ok()
    //        .expect("Error updating dogs name");
    //    Ok(result)
    //}

    // updating dog's age
    //pub async fn update_dog_age(&self, dog_id: &str, new_age: u8) -> Result<UpdateResult, Error> {
    //    let obj_id = ObjectId::parse_str(dog_id).expect("Update failed parsing new dogs age.");
    //    let filter = doc! { "_id": obj_id };
    //    let update = doc! { "$set": doc! { "age": new_age as i32 }};
    //    let result = self.dog_collection
    //        .update_one(filter, update)
    //        .await.ok()
    //        .expect("Error updating dogs age");
    //    Ok(result)
    //}

    // updating dog's breed
    //pub async fn update_dog_breed(&self, dog_id: &str, new_breed: &str) -> Result<UpdateResult, Error> {
    //    let obj_id = ObjectId::parse_str(dog_id).expect("Update failed parsing new dogs breed.");
    //    let filter = doc! { "_id": obj_id };
    //    let update = doc! { "$set": doc! { "breed": new_breed }};
    //    let result = self.dog_collection
    //        .update_one(filter, update)
    //        .await.ok()
    //        .expect("Error updating dogs breed");
    //    Ok(result)
    //}

    // UPDATE: 
    // Cancel in this case below means updating the status of the booking to cancelled: true
    //pub async fn cancel_booking(&self, booking_id: &str ) -> Result<UpdateResult, Error>{
    //    // REF: mongodb update -> https://www.mongodb.com/docs/drivers/rust/current/usage-examples/updateOne/
    //        let obj_id = ObjectId::parse_str(booking_id).expect("Failed to parse booking_id"); // correctly parse ObjectId
    //        let filter = doc! { "_id": obj_id, };
    //        let update = doc! { "$set": doc! { "cancelled" : true } };
    //        let result = self.booking_collection
    //            .update_one(filter, update )
    //            .await.ok()
    //            .expect("Error cancelling brooking");
    //        Ok(result) 
    //}

    
    // read using aggregation
    //pub async fn get_bookings(){
        // REF: mongodb aggregation -> https://www.mongodb.com/docs/drivers/rust/current/fundamentals/aggregation/
    //}


    

    






}
