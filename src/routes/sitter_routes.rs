use actix_web::{delete, get, post, put, web::{self, Data, Json}, HttpResponse};

use crate::{json_response::api_responses::{ErrorJsonApiResponse, JsonApiResponse}, models::sitter_model::{Sitter, SitterRequest, SitterResponse, SitterUpdateRequest}};
use crate::services::db::Database;

// -----------------------------------
// CREATE 
// Create Sitter -> receive POST method on /sitters + a Json SitterRequest obj
#[post("/sitters")]
pub async fn create_sitter(db: Data<Database>, request: Result<Json<SitterRequest>, actix_web::Error> ) -> HttpResponse {
    
    // let sitter_req = request.into_inner();  // request data is of type web::Json<MyStruct>,  Json<SitterRequest> in this case, into_inner() unwraps into inner 'T' value
    // Validate Request
    let sitter_req = match request {
        Ok(valid_json) => valid_json.into_inner(),
        Err(e) => {
            println!("JSON error: {:?}", e);
            return ErrorJsonApiResponse::bad_request("Invalid input. Missing required fields or wrong types.");
        }
    }; 

    // Convert OwnerRequest to Owner and Validate Convertion
    let validated_sitter = match Sitter::try_from(sitter_req) {
        Ok(sitter) => sitter,
        Err(_e) => return ErrorJsonApiResponse::bad_request("Invalid Sitter: Error converting SitterRequest to Sitter."),
    };


    match db.create_sitter(validated_sitter).await
    {   // returns an SitterResponse
        Ok(sitter) => JsonApiResponse::success(SitterResponse::from(sitter)),
        Err(err) => ErrorJsonApiResponse::internal_server_error(&err.to_string()),
    }
}
// -----------------------------------
// READS
// List ALL Sitters -> receive GET method on /sitters
#[get("/sitters")]
pub async fn list_sitters(db: Data<Database>) -> HttpResponse {

    match db.read_sitters().await {
        Ok(vec_sitter) => {
            // map the Vec<Sitter> received from the database handler 'read_sitters' into a vector of SitterResponse, to avoid exposing mongodb objects
            let sitter_responses = vec_sitter.into_iter().map(|x| SitterResponse::from(x)).collect::<Vec<SitterResponse>>();
            //HttpResponse::Ok().json(sitter_responses)
            JsonApiResponse::success(sitter_responses)
        },
        //Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }
}

// List specific Sitter -> receive GET method on /sitters/{id}
#[get("/sitters/{id}")]
pub async fn list_sitter(path: web::Path<String>, db: web::Data<Database>, ) -> HttpResponse {
    // id received must be String because it is a Hexadecimal string
    let id_str = path.into_inner();

    match db.read_sitter(&id_str).await {
        Ok(sitter) =>  {
            //HttpResponse::Ok().json(SitterResponse::from(sitter))
            JsonApiResponse::success(SitterResponse::from(sitter))
        },
       //_ => HttpResponse::NotFound().body("{}"),
       Err(app_error) => ErrorJsonApiResponse::not_found(&app_error.to_string()),
    }
}
// -----------------------------------
// UPDATES
// Update specific Sitter -> receive PUT method on /sitters/{id} + a Json data representing a SitterUpdateRequest Object
#[put("/sitters/{id}")]
pub async fn update_sitter(path: web::Path<String>, db: web::Data<Database>, request: Result<Json<SitterUpdateRequest>, actix_web::Error > ) -> HttpResponse {

    // Validating Json request format 
    let sitter_update = match request{
        Ok(update) => update.into_inner(), 
        Err(_) => return ErrorJsonApiResponse::bad_request("Invalid input: could not parse JSON payload."), 
    };

    // Verify if at least one of the fields was passed 
    if sitter_update.firstname.is_none()
        && sitter_update.lastname.is_none()
        && sitter_update.gender.is_none()
        && sitter_update.email.is_none()
        && sitter_update.phone.is_none()
        && sitter_update.address.is_none()
    {
        return ErrorJsonApiResponse::bad_request("No fields provided to update.");
    }

    let sitter_id = path.into_inner();
    println!("Updating id {:?}", &sitter_id);

    // Invoking database layer
    match db.update_sitter(&sitter_id,sitter_update).await {
        Ok(id) => JsonApiResponse::with_message(&format!("Sitter Update Sucessful: {}", id)),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }
}
// -----------------------------------
// DELETION
// Delete specific Sitter -> receive DELETE method on /sitters/{id}
#[delete("/sitters/{id}")]
pub async fn delete_sitter(path: web::Path<String>, db: web::Data<Database>) -> HttpResponse {

    let sitter_id = path.into_inner();
    println!("Deleting id {:?}", sitter_id);

    match db.delete_sitter(&sitter_id).await {
        Ok(id) => JsonApiResponse::with_message(&format!("Sitter Deleted: {}", id)),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),

    }
}


