use actix_web::{web::{self, Json}, HttpResponse};
use crate::models::booking_model::{Booking, BookingRequest, BookingResponse,BookingUpdateRequest};
use crate::services::db::Database;

// -----------------------------------
// CREATE 
// Create Booking -> receive POST method on /bookings  with Json data representing a BookingRequest Object
#[actix_web::post("/bookings")]
pub async fn create_booking(db: web::Data<Database>, request: web::Json<BookingRequest> ) -> HttpResponse {
    println!("Creating new Booking");

    let booking_req = request.into_inner();
    match db.create_booking(
        Booking::try_from(
            booking_req
            //BookingRequest{ owner: request.owner.clone(), start_time: request.start_time.clone(), duration_minutes: request.duration_minutes.clone(), }
        ).expect("Error converting BookingRequest to Booking.")
    ).await
    {
        // returns a BookingResponse
        Ok(booking) => HttpResponse::Ok().json(BookingResponse::from(booking)),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// -----------------------------------
// READS
// LIST Bookings  -> receive GET method on /bookings
#[actix_web::get("/bookings")]
pub async fn list_bookings(db: web::Data<Database>) -> HttpResponse {
    println!("Reading all Bookings");

    match db.read_bookings().await {
        Ok(booking) => {
            let booking_responses = booking.into_iter().map(|x| BookingResponse::from(x)).collect::<Vec<BookingResponse>>();
            HttpResponse::Ok().json(booking_responses)
        },
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// List spcific Booking -> receive GET method on /bookings/{id} 
#[actix_web::get("/bookings/{id}")]
pub async fn list_booking(path: web::Path<String>, db: web::Data<Database> ) -> HttpResponse {
    // id received must be String because it is a Hexadecimal string
    let booking_id = path.into_inner();
    println!("Reading Booking with id: {:?}", booking_id);

    match db.read_booking(&booking_id).await {
        Some(booking_reponse ) => HttpResponse::Ok().json(booking_reponse),
        _ => HttpResponse::NotFound().body("{}"),
    }
}

// -----------------------------------
// UPDATES
// Update specific Booking -> receive PUT method on /bookings/{id}  + a Json data representing a BookingUpdateRequest Object

#[actix_web::put("/bookings/{id}")]
pub async fn update_booking(path: web::Path<String>, db: web::Data<Database>, request: Json<BookingUpdateRequest>, ) -> HttpResponse {

    let booking_id = path.into_inner();
    println!("Updating Booking id {:?}", booking_id);

    match db.update_booking(&booking_id, request.into_inner()).await {
        Ok(true) => HttpResponse::Ok().body("Owner updated"),
        Ok(false) => HttpResponse::BadRequest().body("No valid fields provided or owner not found"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

// -----------------------------------
// DELETION
// Delete specific Booking -> receive DELETE method on /bookings/{id}
#[actix_web::delete("/bookings/{id}")]
pub async fn delete_booking(path: web::Path<String>, db: web::Data<Database>) -> HttpResponse {

    let booking_id = path.into_inner();
    println!("Deleting id {:?}", booking_id);

    match db.delete_booking(&booking_id).await {
        Ok(true) => HttpResponse::Ok().body("Booking deleted."),
        Ok(false) => HttpResponse::BadRequest().body("No valid fields provided or Booking not found"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}