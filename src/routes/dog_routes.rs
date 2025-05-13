use actix_web::{web::{self, Json}, HttpResponse};
use crate::{json_response::api_responses::{ErrorJsonApiResponse, JsonApiResponse}, models::dog_model::{Dog, DogRequest, DogResponse, DogUpdateRequest}};
use crate::services::db::Database;

// -----------------------------------
// CREATE 
// Create Dog -> receive POST method on /dogs with Json data representing a DogRequest Object
#[actix_web::post("/dogs")]
pub async fn create_dog(db: web::Data<Database>, request: Result<web::Json<DogRequest>, actix_web::Error> ) -> HttpResponse {
    
    //let dog_req = request.into_inner();  // request data is of type web::Json<MyStruct>,  Json<OwnerRequest> in this case, into_inner() unwraps into inner 'T' value
    // Validate Request
    let dog_req = match request {
        Ok(valid_json) => valid_json.into_inner(),
        Err(e) => {
            println!("JSON error: {:?}", e);
            return ErrorJsonApiResponse::bad_request("Invalid Json input. Missing required fields or wrong types.");
        }
    };  // request data is of type web::Json<MyStruct>,  Json<OwnerRequest> in this case, into_inner() unwraps into inner 'T' value
    

    //match db.create_dog(Dog::try_from(dog_req).expect("Error converting DogRequest to Dog." ) ).await {
    //    Ok(dog) => JsonApiResponse::success(DogResponse::from(dog)),
    //    Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    //}

    // Convert DogRequest to Dog and Validate Convertion
    let validated_dog = match Dog::try_from(dog_req) {
        Ok(dog) => dog,
        Err(_e) => return ErrorJsonApiResponse::bad_request("Invalid Dog: Error converting DogRequest to Dog."),
        };

    match db.create_dog(validated_dog).await {
        Ok(created_dog) => JsonApiResponse::success(DogResponse::from(created_dog)),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }



}

// -----------------------------------
// READS
// LIST All Dogs  -> receive GET method on /dogs
#[actix_web::get("/dogs")]
pub async fn list_dogs(db: web::Data<Database>) -> HttpResponse {

    match db.read_dogs().await {
        Ok(dog_vec) => {
            let dogs_responses = dog_vec.into_iter().map(|x| {DogResponse::from(x) }).collect::<Vec<DogResponse>>();
            HttpResponse::Ok().json(dogs_responses)
        },
        //Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }
}

// List a spcific Dog -> receive GET method on /dogs/{id} 
#[actix_web::get("/dogs/{id}")]
pub async fn list_dog(path: web::Path<String>, db: web::Data<Database> ) -> HttpResponse {
    
    // id received must be String because it is a Hexadecimal string
    let dog_id = path.into_inner();
    
    match db.read_dog(&dog_id).await {
        Ok(dog) => JsonApiResponse::success(DogResponse::from(dog)),
        Err(app_error) => ErrorJsonApiResponse::not_found(&app_error.to_string()),
    }

}

// -----------------------------------
// UPDATES
// Update specific Dog -> receive PUT method on /dogs/{id} + a Json data representing a DogUpdateRequest Object
#[actix_web::put("/dogs/{id}")]
pub async fn update_dog(path: web::Path<String>, db: web::Data<Database>, request: Result<Json<DogUpdateRequest>, actix_web::Error> ) -> HttpResponse {

    // Validating request
    let dog_update = match request {
        Ok(update) => update.into_inner(),
        Err(actix_error) => { return ErrorJsonApiResponse::bad_request(&format!("Invalid input: could not parse JSON payload: {}", actix_error)); }
    };
    // Validate that at least one field is Some
    if dog_update.name.is_none()
        && dog_update.owner.is_none()
        && dog_update.age.is_none()
        && dog_update.breed.is_none()
    {
        return ErrorJsonApiResponse::bad_request("No fields provided to update.");
    }

    let dog_id = path.into_inner();


    //println!("Updating Dog id {:?}", dog_id);
    match db.update_dog(&dog_id, dog_update).await {
        Ok(id) => JsonApiResponse::with_message(&format!("Dog Update Sucessful: {}", id)),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }
}

// -----------------------------------
// DELETION
// Delete specific Dog -> receive DELETE method on /dogs/{id}
#[actix_web::delete("/dogs/{id}")]
pub async fn delete_dog(path: web::Path<String>, db: web::Data<Database>) -> HttpResponse {

    let dog_id = path.into_inner();
    println!("Deleting Dog id {:?}", dog_id);

    match db.delete_dog(&dog_id).await {
        Ok(id) => JsonApiResponse::with_message(&format!("Dog Deleted: {}", id)),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }
}