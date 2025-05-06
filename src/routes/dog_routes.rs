use actix_web::{web::{self, Json}, HttpResponse};
use crate::models::dog_model::{Dog, DogRequest, DogResponse, DogUpdateRequest};
use crate::services::db::Database;

// -----------------------------------
// CREATE 
// Create Dog -> receive POST method on /dogs with Json data representing a DogRequest Object
#[actix_web::post("/dogs")]
pub async fn create_dog(db: web::Data<Database>, request: web::Json<DogRequest> ) -> HttpResponse {
    
    let dog_req = request.into_inner();  // request data is of type web::Json<MyStruct>,  Json<OwnerRequest> in this case, into_inner() unwraps into inner 'T' value
    
    match db.create_dog(
        Dog::try_from(
            dog_req
            ).expect("Error converting DogRequest to Dog.")
    ).await
    {
        // returns a DogResponse
        Ok(dog) => HttpResponse::Ok().json(DogResponse::from(dog)),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// -----------------------------------
// READS
// LIST All Dogs  -> receive GET method on /dogs
#[actix_web::get("/dogs")]
pub async fn list_dogs(db: web::Data<Database>) -> HttpResponse {
    println!("Read all Dogs");
    match db.read_dogs().await {
        Ok(dog_vec) => {
            let dogs_responses = dog_vec.into_iter().map(|x| {DogResponse::from(x) }).collect::<Vec<DogResponse>>();
            HttpResponse::Ok().json(dogs_responses)
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// List a spcific Dog -> receive GET method on /dogs/{id} 
#[actix_web::get("/dogs/{id}")]
pub async fn list_dog(path: web::Path<String>, db: web::Data<Database> ) -> HttpResponse {
    // id received must be String because it is a Hexadecimal string
    let dog_id = path.into_inner();
    println!("Read Dog id {:?}", dog_id);
    match db.read_dog(&dog_id).await {
        Some(dog ) => HttpResponse::Ok().json(DogResponse::from(dog)),
        _ => HttpResponse::NotFound().body("{}"),
    }
}

// -----------------------------------
// UPDATES
// Update specific Dog -> receive PUT method on /dogs/{id} + a Json data representing a DogUpdateRequest Object
#[actix_web::put("/dogs/{id}")]
pub async fn update_dog(path: web::Path<String>, db: web::Data<Database>, request: Json<DogUpdateRequest>, ) -> HttpResponse {

    let dog_id = path.into_inner();
    //println!("Updating Dog id {:?}", owner_id);
    match db.update_dog(&dog_id, request.into_inner()).await {
        Ok(true) => HttpResponse::Ok().body("Dog updated"),
        Ok(false) => HttpResponse::BadRequest().body("No valid fields provided or Dog not found"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
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
        Ok(true) => HttpResponse::Ok().body("Dog deleted."),
        Ok(false) => HttpResponse::BadRequest().body("No valid fields provided or Dog id not found"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}