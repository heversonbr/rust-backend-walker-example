use actix_web::{delete, get, post, put, web::{self, Data, Json}, HttpResponse};

use crate::models::owner_model::{Owner, OwnerRequest, OwnerResponse, OwnerUpdateRequest};
use crate::services::db::Database;


// API Route Naming Guidelines (Memo)
// -----------------------------------
// Use this as a reference when defining new API routes.
//
// 1. Use Nouns, Not Verbs:
//    - Routes represent resources, not actions.
//    - Example: GET /users (not /getUsers)
//
// 2. Use Plural Nouns for Collections:
//    - Keep naming consistent and RESTful.
//    - Example: /users, /orders, /products
//
// 3. Nest Routes for Sub-resources:
//    - Reflect relationships between resources.
//    - Example: /users/{userId}/orders
//
// 4. Use Standard HTTP Methods:
//    - GET: Retrieve data
//    - POST: Create new resource
//    - PUT/PATCH: Update resource
//    - DELETE: Remove resource
//
// 5. Use Hyphens (-) for Multi-word Names:
//    - Improves readability.
//    - Example: /user-profiles (not /userProfiles)
//
// 6. Use Query Parameters for Filtering and Pagination:
//    - Example: /users?age=30&sort=name&page=2
//
// 7. Version the API When Necessary:
//    - Example: /v1/users
//
// 8. Be Predictable and Consistent:
//    - Avoid abbreviations and custom verbs.
//    - Stick to a clear and uniform naming pattern.
// -----------------------------------

// -----------------------------------
// CREATE 
// Create Owner -> receive POST method on /owners + a Json OwnerRequest obj
#[post("/owners")]
pub async fn create_owner(db: Data<Database>, request: Json<OwnerRequest> ) -> HttpResponse {
    
    let owner_req = request.into_inner();  // request data is of type web::Json<MyStruct>,  Json<OwnerRequest> in this case, into_inner() unwraps into inner 'T' value
    match db.create_owner(
        Owner::try_from(
            owner_req
            ).expect("Error converting DogRequest to Owner.")
    ).await
    {   // returns an OwnerResponse
        Ok(owner) => HttpResponse::Ok().json(OwnerResponse::from(owner)),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
// -----------------------------------
// READS
// List ALL Owners -> receive GET method on /owners
#[get("/owners")]
pub async fn list_owners(db: Data<Database>) -> HttpResponse {

    match db.read_owners().await {
        Ok(vec_owner) => {
            // map the Vec<Owner> received from the database handler 'read_owners' into a vector of OwnerResponse, to avoid exposing mongodb objects
            let owner_responses = vec_owner.into_iter().map(|x| OwnerResponse::from(x)).collect::<Vec<OwnerResponse>>();
            HttpResponse::Ok().json(owner_responses)
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// List specific Owner -> receive GET method on /owners/{id}
#[get("/owners/{id}")]
pub async fn list_owner(path: web::Path<String>, db: web::Data<Database>, ) -> HttpResponse {
    // id received must be String because it is a Hexadecimal string
    let id_str = path.into_inner();

    match db.read_owner(&id_str).await {
        Some(owner) =>  {
            HttpResponse::Ok().json(OwnerResponse::from(owner))
        },
       _ => HttpResponse::NotFound().body("{}"),
    }
}
// -----------------------------------
// UPDATES
// Update specific Owner -> receive PUT method on /owners/{id} + a Json data representing a OwnerUpdateRequest Object
#[put("/owners/{id}")]
pub async fn update_owner(path: web::Path<String>, db: web::Data<Database>, request: Json<OwnerUpdateRequest>, ) -> HttpResponse {

    let owner_id = path.into_inner();
    println!("Updating id {:?}", owner_id);
    match db.update_owner(&owner_id, request.into_inner()).await {
        Ok(true) => HttpResponse::Ok().body("Owner updated"),
        Ok(false) => HttpResponse::BadRequest().body("No valid fields provided or owner not found"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
// -----------------------------------
// DELETION
// Delete specific Owner -> receive DELETE method on /owners/{id}
#[delete("/owners/{id}")]
pub async fn delete_owner(path: web::Path<String>, db: web::Data<Database>) -> HttpResponse {

    let owner_id = path.into_inner();
    println!("Deleting id {:?}", owner_id);

    match db.delete_owner(&owner_id).await {
        Ok(true) => HttpResponse::Ok().body("Owner deleted."),
        Ok(false) => HttpResponse::BadRequest().body("No valid fields provided or owner not found"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}


