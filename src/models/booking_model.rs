use mongodb::bson::{oid::ObjectId, DateTime};
use validator::Validate; // a mature crate that works with serde for fields validation
use serde::{Deserialize, Serialize};   
// Serialize, Deserialize: Needed for Actix (for sending/receiving JSON) and MongoDB (bson conversion).
use chrono::Utc;
use std::time::SystemTime;


// Separating database and API input schemas: backend pattern design
#[derive(Debug, Serialize, Deserialize)]
pub struct Booking {
    pub _id: ObjectId,          // MongoDB needs an "_id" field, a unique identifier, ObjectId: Special ID format used by MongoDB
    pub owner: ObjectId,        // The ID of the user who made the booking
    pub start_time: DateTime,   // When the booking starts, DateTime (MongoDB BSON version): This is different from chrono::DateTime. MongoDB uses its own date format internally
    pub duration_minutes: u8,   // How long it lasts (in minutes)
    pub cancelled: bool,        // If the booking was cancelled
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BookingRequest { 
    pub owner: String,           // Client sends owner ID as string
    pub start_time: String,      // Client sends start time as a string (RFC3339 datetime string)
    pub duration_minutes: u8,    // Client sends duration

}

// use TryFrom for 'validated' or 'fallible' mappings (like request → domain struct)
impl TryFrom<BookingRequest> for Booking{
    type Error = Box<dyn std::error::Error >;

    fn try_from(booking_request: BookingRequest) -> Result<Self, Self::Error> {
    let chrono_datatime: SystemTime = 
            chrono::DateTime::parse_from_rfc3339(&booking_request.start_time)
            .map_err(|err| format!("Failed to parse start time: {} ", err))?
            .with_timezone(&Utc).into();
    // Parse the start_time from a string like "2025-04-28T12:00:00Z" to a chrono::DateTime<Utc>
	// and convert it to mongodb::bson::DateTime via DateTime::from_chrono()
    Ok(Self{
            _id: ObjectId::new(),  // Create a new _id for MongoDB
            owner: ObjectId::parse_str(&booking_request.owner).expect("Failed to parse owner!"),    // Parse owner string to an ObjectId
            start_time: DateTime::from(chrono_datatime),  
            duration_minutes: booking_request.duration_minutes,
            cancelled: false,
    })
    }

}

// BookingResponse: Used to send clean, flattened JSON to clients (`Response` structs)
// Converts _id to String, Returns clean JSON-friendly values, Used for sending to API clients
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BookingResponse {
    pub _id: String,          
    pub owner: String,        
    pub start_time: String,   // RFC3339 string
    pub duration_minutes: u8, 
    pub cancelled: bool,      
}

// use From as it is a safe mapping (from database 'Booking' struct → response 'BookingResponse' struct)
impl From<Booking> for BookingResponse {
    fn from(booking: Booking) -> Self {
        Self {
            _id: booking._id.to_hex(),          
            owner: booking.owner.to_hex(),     
            //start_time: booking.start_time.to_chrono(),
            start_time: booking.start_time.to_chrono().to_rfc3339(),  
            duration_minutes: booking.duration_minutes ,
            cancelled: booking.cancelled,
        }
    }
}


// Updates: for updates we use specific structs for:
// BookingUpdateRequest
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BookingUpdateRequest{
    pub owner:            Option<String>,        
    pub start_time:       Option<String>,   // RFC3339 string
    pub duration_minutes: Option<u8>, 
    pub cancelled:        Option<bool>,
}
// BookingUpdateResponse, we can create a new struct here for consistency reasons but BookingResponse seems to have the same effect. 
