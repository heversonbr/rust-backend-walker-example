use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};  

// ============================================================================
// Separating database and API input schemas: common pattern in backend design (see above):
// A sitter is a pet walker, I decide to call sitter instead of walker
// ============================================================================
// Sitter: Represents the data stored in MongoDB. 
// Analogy: what your system stores (includes ID, timestamps, metadata, etc.)
// note that we are using the crate 'validator' to validate the fields here. (https://github.com/Keats/validator)
// Contains ObjectId, Might expose Mongo types, Used internally / for DB
#[derive(Debug, Serialize, Deserialize)]
pub struct Sitter {
    pub _id: ObjectId,        
    pub firstname: String,
    pub lastname: String,
    pub gender: String,
    pub email: String,
    pub phone: String,
    pub address: String,

}

// -> maybe Endorsements: personal recommendations from friends, family, clients, coworkers, and other community members that help Pet Caregivers build credibility and trust with clients

// SitterRequest: Represents the incoming data from the client (e.g. from an HTTP POST/PUT body):
// It does not include _id, because the client doesn’t know or set it.
// Analogy: this is a form someone fills in (just name, email, etc.)
#[derive(Debug, Serialize, Deserialize)]
pub struct SitterRequest { 
    pub firstname: String,
    pub lastname: String,
    pub gender: String,
    pub email: String,
    pub phone: String,
    pub address: String,
}

// use TryFrom for 'validated' or 'fallible' mappings (like request → domain struct)
impl TryFrom<SitterRequest> for Sitter{
    type Error = Box<dyn std::error::Error >;
    fn try_from(request: SitterRequest) -> Result<Self, Self::Error> {
        Ok(Self{
            _id: ObjectId::new(),  // Create a new _id for MongoDB
            firstname: request.firstname,
            lastname: request.lastname,
            gender: request.gender,
            email: request.email,
            phone: request.phone,
            address: request.address,  
        })
    }
}

// SitterResponse: Used to send clean, flattened JSON to clients (`Response` structs)
// Converts _id to String, Returns clean JSON-friendly values, Used for sending to API clients
#[derive(Debug, Serialize, Deserialize)]
pub struct SitterResponse {
    pub _id: String,        
    pub firstname: String,
    pub lastname: String,
    pub gender: String,
    pub email: String,
    pub phone: String,
    pub address: String,
}

// use From as it is a safe mapping (from database 'Sitter' struct → response 'SitterResponse' struct)
impl From<Sitter> for SitterResponse {
    fn from(sitter: Sitter) -> Self {
        Self {
            _id: sitter._id.to_hex(),
            firstname: sitter.firstname,
            lastname: sitter.lastname,
            gender: sitter.gender,
            email: sitter.email,
            phone: sitter.phone,
            address: sitter.address,
        }
    }
}

// Updates: for updates we use specific structs for:
// - clarity: each struct clearly expresses its purpose 
// - avoid Accidental Overwrites
// SitterUpdateRequest
#[derive(Debug, Serialize, Deserialize)]
pub struct SitterUpdateRequest{
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub gender: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
}
// SitterUpdateResponse, we can create a new struct here for consistency reasons but SitterResponse seems to have the same effect. 
