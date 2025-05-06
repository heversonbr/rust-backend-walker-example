use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};  
use validator::{Validate};
// Serialize, Deserialize: Needed for Actix (for sending/receiving JSON) and MongoDB (bson conversion).

// Notes on Data Struct Separation: Domain vs API Layer
// ============================================================================
//
// This file defines two kinds of structs for each resource (e.g., `Owner`, `Dog`):
//
// 1. Domain structs (e.g., `Owner`, `Dog`)
//    - Used internally and for MongoDB interaction
//    - Contain database-native types like `ObjectId`, `DateTime`, etc.
//    - Represent complete entities as stored in the database
//
// 2. Request/Response structs:
//    - `OwnerRequest` / `DogRequest`: data received from clients (e.g., via POST/PUT)
//    - `ResponseOwner`: data sent to clients in clean, JSON-friendly form
//
// Why this separation?
// - Keeps internal DB representation decoupled from API surface
// - Prevents leaking BSON-specific types like `$oid`, `$date` into frontend responses
// - Makes validation, serialization, and evolution of the API easier
//   For safe mappings (like database struct → response struct): you can use From.
//
// Use `impl From<T> for U` / `impl TryFrom<T> for U`
// - Implementing `From<Owner> for ResponseOwner` lets you easily map DB structs to responses.
// - Implementing `TryFrom<OwnerRequest> for Owner` lets you validate and enrich incoming data 
//   (e.g., auto-generate `_id`, set default timestamps). For validated or fallible mappings
//   (like request → domain struct): use TryFrom.
//
// Example:  safe mapping (database struct → response struct)
// ```rust
// impl From<Owner> for ResponseOwner {
//     fn from(owner: Owner) -> Self {
//         ResponseOwner {
//             id: owner._id.to_hex(),
//             name: owner.name,
//             ...
//         }
//     }
// }
// ```
//
// Benefits:
// - Centralizes conversion logic (less duplication, fewer bugs)
// - Makes `.into()` and `.try_into()` ergonomic throughout your handlers
// - Keeps your route functions clean and focused
//
// ============================================================================

// Separating database and API input schemas: common pattern in backend design (see above):

// Owner: Represents the data stored in MongoDB. 
// Analogy: what your system stores (includes ID, timestamps, metadata, etc.)
// note that we are using the crate 'validator' to validate the fields here. (https://github.com/Keats/validator)
// Contains ObjectId, Might expose Mongo types, Used internally / for DB
#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    pub _id: ObjectId,        
    pub name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
}

// OwnerRequest: Represents the incoming data from the client (e.g. from an HTTP POST/PUT body):
// It does not include _id, because the client doesn’t know or set it.
// Analogy: this is a form someone fills in (just name, email, etc.)
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct OwnerRequest { 
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 7, message = "Phone number too short"))]
    pub phone: String,
    #[validate(length(min = 5))]
    pub address: String,
}

// use TryFrom for 'validated' or 'fallible' mappings (like request → domain struct)
impl TryFrom<OwnerRequest> for Owner{
    type Error = Box<dyn std::error::Error >;

    fn try_from(item: OwnerRequest) -> Result<Self, Self::Error> {

        Ok(Self{
            _id: ObjectId::new(),  // Create a new _id for MongoDB
            name: item.name,
            email: item.email,
            phone: item.phone,
            address: item.address,  
        })
    }
}

// OwnerResponse: Used to send clean, flattened JSON to clients (`Response` structs)
// Converts _id to String, Returns clean JSON-friendly values, Used for sending to API clients
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct OwnerResponse {
    pub _id: String,        
    pub name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
}

// use From as it is a safe mapping (from database 'Owner' struct → response 'OwnerResponse' struct)
impl From<Owner> for OwnerResponse {
    fn from(owner: Owner) -> Self {
        Self {
            _id: owner._id.to_hex(),
            name: owner.name,
            email: owner.email,
            phone: owner.phone,
            address: owner.address,
        }
    }
}


// Updates: for updates we use specific structs for:
// - clarity: each struct clearly expresses its purpose 
// - avoid Accidental Overwrites
// OwnerUpdateRequest
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct OwnerUpdateRequest{
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}
// OwnerUpdateResponse, we can create a new struct here for consistency reasons but OwnerResponse seems to have the same effect. 




// Why this pattern of separating data in transit and data on storage is useful:
// - Security / Control: You don’t want users to send or control the _id. This prevents them from: Overwriting others’ data and Spoofing document identities
// - Flexibility in Serialization: You can serialize/deserialize PersonRequest directly from JSON (from HTTP), avoidding complications from MongoDB-specific fields like _id
// - Clarity in API Contracts: Having a separate PersonRequest struct: Clearly defines what data the client is allowed to send and Keeps your database schema and API input schema explicitly separated

// Note: 
// even for updates, we keep the separation
// - ID: comes from the path (/owner/{id})
// - Data to update: comes from body (PersonReque
// This keeps things secure, simple, and well-structured.