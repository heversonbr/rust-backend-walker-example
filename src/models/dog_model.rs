use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize}; 
use validator::Validate;   
// Serialize, Deserialize: Needed for Actix (for sending/receiving JSON) and MongoDB (bson conversion).


// Dog: Represents the data stored in MongoDB. 
#[derive(Debug, Serialize, Deserialize)]
pub struct Dog {
    pub _id: ObjectId,        
    pub owner: ObjectId,
    pub name:  String,
    pub age:   Option<u8>,
    pub breed: Option<String>,
}


// DogRequest: Represents the incoming data from the client (e.g. from an HTTP POST/PUT body):
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct DogRequest { 
    
    pub owner: String,
    pub name:  String,
    pub age:   Option<u8>,
    pub breed: Option<String>,

}

// use TryFrom for 'validated' or 'fallible' mappings (DogRequest → Dog domain struct)
impl TryFrom<DogRequest> for Dog{
    type Error = Box<dyn std::error::Error >;

    fn try_from(item: DogRequest) -> Result<Self, Self::Error> {
        Ok(Self{
                _id: ObjectId::new(),  // Create a new _id for MongoDB
                owner: ObjectId::parse_str(&item.owner).expect("Failed to parse owner."),
                name: item.name,
                age: item.age,
                breed: item.breed,
        })
    }
}

// Struct used to send flattened responses for Dog struct , avoiddig sending mongodb types
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct DogResponse {
    pub _id: String,        
    pub owner: String,
    pub name:  String,
    pub age:   Option<u8>,
    pub breed: Option<String>,
}

// use From as it is a 'safe mapping' (from database 'Dog' struct → response 'DogResponse' struct)
impl From<Dog> for DogResponse {

    fn from(dog: Dog) -> Self {
        Self { 
            _id: dog._id.to_hex(),
            owner: dog.owner.to_hex(), 
            name: dog.name, 
            age: dog.age, 
            breed: dog.breed
        }
    }
}

// Updates: for updates we use specific structs for:
// - clarity: each struct clearly expresses its purpose 
// - avoid Accidental Overwrites
// DogUpdateRequest
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct DogUpdateRequest{
    pub owner: Option<String>,
    pub name:  Option<String>,
    pub age:   Option<u8>,
    pub breed: Option<String>,
}
// DogUpdateResponse, we can create a new struct here for consistency reasons but DogResponse seems to have the same effect. 
