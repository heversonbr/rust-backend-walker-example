use actix_web::{web::{self, Json}, HttpResponse};
use crate::{json_response::api_responses::{ErrorJsonApiResponse, JsonApiResponse}, 
            models::booking_model::{Booking, BookingRequest, BookingResponse,BookingUpdateRequest}, services::bookings};

//use mongodb::AppDatabase; 
use crate::services::db::AppDatabase;  //  ← again, use the actual type


// -----------------------------------
// CREATE 
// Create Booking -> receive POST method on /bookings  with Json data representing a BookingRequest Object
#[actix_web::post("/bookings")]
pub async fn create_booking(
    db: web::Data<AppDatabase>, 
    request: Result<web::Json<BookingRequest>, 
    actix_web::Error> ) -> HttpResponse {
    println!("Creating new Booking");

    //let booking_req = request.into_inner();
     // Validate Request
     let booking_req = match request {
        Ok(valid_json) => valid_json.into_inner(),
        Err(e) => {
            println!("JSON error: {:?}", e);
            return ErrorJsonApiResponse::bad_request("Invalid input. Missing required fields or wrong types.");
        }
    }; 

    // Convert BookingRequest to Booking and Validate Convertion
    let validated_booking = match Booking::try_from(booking_req) {
        Ok(booking) => booking,
        Err(_e) => return ErrorJsonApiResponse::bad_request("Invalid Booking: Error converting BookingRequest to Booking."),
    };
    // Note: by validating the convertion before we avoid using this 'expect()' method here below
    // because Booking structure is already validated and an error is propagated if errors happen.
    // we use the validated_booking next, instead of  // Booking::try_from(booking_req ).expect("Error converting BookingRequest to Booking.")

    match bookings::create_booking(&db, validated_booking).await {
        //Ok(booking) => HttpResponse::Ok().json(BookingResponse::from(booking)), 
        Ok(inserted_booking) => JsonApiResponse::success(BookingResponse::from(inserted_booking)),
        //Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        Err(error) => ErrorJsonApiResponse::internal_server_error(&error.to_string()),
    }
}

// -----------------------------------
// READS
// LIST Bookings  -> receive GET method on /bookings
#[actix_web::get("/bookings")]
pub async fn list_bookings(db: web::Data<AppDatabase>) -> HttpResponse {
    println!("Reading all Bookings");

    match bookings::read_bookings(&db).await {
        Ok(booking) => {
            let booking_responses = booking.into_iter().map(|x| BookingResponse::from(x)).collect::<Vec<BookingResponse>>();
            //HttpResponse::Ok().json(booking_responses)
            JsonApiResponse::success(booking_responses)
        },
        //Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }
}

// List spcific Booking -> receive GET method on /bookings/{id} 
#[actix_web::get("/bookings/{id}")]
pub async fn list_booking(path: web::Path<String>, db: web::Data<AppDatabase> ) -> HttpResponse {
   
    // id received must be String because it is a Hexadecimal string
    let booking_id = path.into_inner();

    match bookings::read_booking(&db, &booking_id).await {
        Ok(booking ) =>  JsonApiResponse::success(BookingResponse::from(booking)),
        Err(app_error) => ErrorJsonApiResponse::not_found(&app_error.to_string()),
    }
}

// -----------------------------------
// UPDATES
// Update specific Booking -> receive PUT method on /bookings/{id}  + a Json data representing a BookingUpdateRequest Object

#[actix_web::put("/bookings/{id}")]
pub async fn update_booking(path: web::Path<String>, db: web::Data<AppDatabase>, request: Result< Json<BookingUpdateRequest>, actix_web::Error> ) -> HttpResponse {

     // Validating request
     let booking_update = match request {
        Ok(update) => update.into_inner(),
        Err(_) => { return ErrorJsonApiResponse::bad_request("Invalid input: could not parse JSON payload."); }
    };

     // Validate that at least one field is Some
     if booking_update.owner.is_none()
        && booking_update.start_time.is_none()
         && booking_update.duration_minutes.is_none()
        && booking_update.cancelled.is_none()
     {
         return ErrorJsonApiResponse::bad_request("No fields provided to update.");
     }

    let booking_id = path.into_inner();
    
     // Invoking database layer 
    match bookings::update_booking(&db, &booking_id, booking_update).await {
        Ok(id) => JsonApiResponse::with_message(&format!("Booking Update Sucessful: {}", id)),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }
}

// -----------------------------------
// DELETION
// Delete specific Booking -> receive DELETE method on /bookings/{id}
#[actix_web::delete("/bookings/{id}")]
pub async fn delete_booking(path: web::Path<String>, db: web::Data<AppDatabase>) -> HttpResponse {

    let booking_id = path.into_inner();
    println!("Deleting id {:?}", booking_id);

    match bookings::delete_booking(&db, &booking_id).await {
        Ok(id) => JsonApiResponse::with_message(&format!("Booking Deleted: {}", id)),
        Err(app_error) => ErrorJsonApiResponse::internal_server_error(&app_error.to_string()),
    }
}