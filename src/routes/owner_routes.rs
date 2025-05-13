use actix_web::{delete, get, post, put, web::{self, Json}, HttpResponse};

use crate::{json_response::api_responses::{ErrorJsonApiResponse, JsonApiResponse}, 
            models::owner_model::{Owner, OwnerRequest, OwnerResponse, OwnerUpdateRequest}, };
use crate::services::owners;

use crate::services::db::AppDatabase;// ← again, use the actual type


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
pub async fn create_owner(
        db: web::Data<AppDatabase>,   // ← must match exac
        request: Result<Json<OwnerRequest>, 
        actix_web::Error> ) -> HttpResponse {
    // Json is wrapped by a Result to allow validating the request locally here. 
    println!("CREATE ROUTER");
    //let owner_req = request.into_inner();  // request data is of type web::Json<MyStruct>,  Json<OwnerRequest> in this case, into_inner() unwraps into inner 'T' value
    // Validate Request
    let owner_req = match request {
        Ok(valid_json) => valid_json.into_inner(),
        Err(e) => {
            println!("JSON error: {:?}", e);
            return ErrorJsonApiResponse::bad_request("Invalid input. Missing required fields or wrong types.");
        }
    };  // request data is of type web::Json<MyStruct>,  Json<OwnerRequest> in this case, into_inner() unwraps into inner 'T' value
    
    // Convert OwnerRequest to Owner and Validate Convertion
    let validated_owner = match Owner::try_from(owner_req) {
        Ok(owner) => owner,
        Err(_e) => return ErrorJsonApiResponse::bad_request("Invalid Owner: Error converting OwnerRequest to Owner."),
    };

    println!("CREATE ROUTER: calling create_owner...");
    match owners::create_owner(&db, validated_owner).await
    {   // returns an OwnerResponse
        Ok(inserted_owner) => JsonApiResponse::success(OwnerResponse::from(inserted_owner)),
        Err(error) => ErrorJsonApiResponse::internal_server_error(&error.to_string()),
    }

}
// -----------------------------------
// READS
// List ALL Owners -> receive GET method on /owners
#[get("/owners")]
pub async fn list_owners(db: web::Data<AppDatabase>) -> HttpResponse {

    match owners::read_owners(&db).await {
        Ok(vec_owner) => {
            // map the Vec<Owner> received from the database handler 'read_owners' into a vector of OwnerResponse, to avoid exposing mongodb objects
            let owner_responses = vec_owner.into_iter().map(|x| OwnerResponse::from(x)).collect::<Vec<OwnerResponse>>();
            //HttpResponse::Ok().json(owner_responses)
            JsonApiResponse::success(owner_responses)
        },
       // Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
       Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }
}

// List specific Owner -> receive GET method on /owners/{id}
#[get("/owners/{id}")]
pub async fn list_owner(path: web::Path<String>, db: web::Data<AppDatabase>) -> HttpResponse {
    
    // id received must be String because it is a Hexadecimal string
    let id_str = path.into_inner();
  
    match owners::read_owner(&db, &id_str).await {
        Ok(owner) =>  {
           // HttpResponse::Ok().json(OwnerResponse::from(owner))
           JsonApiResponse::success(OwnerResponse::from(owner))
        },
        Err(app_error) => ErrorJsonApiResponse::not_found(&app_error.to_string()),
       // Err(e) => HttpResponse::NotFound().body("{}"),
    }
}
// -----------------------------------
// UPDATES
// Update specific Owner -> receive PUT method on /owners/{id} + a Json data representing a OwnerUpdateRequest Object
#[put("/owners/{id}")]
pub async fn update_owner(path: web::Path<String>, db: web::Data<AppDatabase>, request: Result<Json<OwnerUpdateRequest>, actix_web::Error>, ) -> HttpResponse {
    // initially the request wasnt a Result, but I wrapped it into a Result in order to validate it here
   
    // Validating request
    let owner_update = match request {
        Ok(update) => update.into_inner(),
        Err(_) => { return ErrorJsonApiResponse::bad_request("Invalid input: could not parse JSON payload."); }
    };
    // Validate that at least one field is Some
    if owner_update.name.is_none()
        && owner_update.email.is_none()
        && owner_update.phone.is_none()
        && owner_update.address.is_none()
    {
        return ErrorJsonApiResponse::bad_request("No fields provided to update.");
    }

    let owner_id = path.into_inner();
    println!("Updating id {:?}", owner_id);

    // Invoking database layer 
    match owners::update_owner(&db, &owner_id, owner_update).await {
        Ok(id) => JsonApiResponse::with_message(&format!("Owner Update Sucessful: {}", id)),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }
}
// -----------------------------------
// DELETION
// Delete specific Owner -> receive DELETE method on /owners/{id}
#[delete("/owners/{id}")]
pub async fn delete_owner(path: web::Path<String>, db: web::Data<AppDatabase>) -> HttpResponse {

    let owner_id = path.into_inner();
    println!("Deleting id {:?}", owner_id);

    match owners::delete_owner(&db, &owner_id).await {
        Ok(id) => JsonApiResponse::with_message(&format!("Owner Deleted: {}", id)),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }
}


