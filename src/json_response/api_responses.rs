use actix_web::HttpResponse;
use serde::Serialize;

// Successful messages with/without data
#[derive(Debug, Serialize)]
pub struct JsonApiResponse<T> {
    pub success: bool,    
    pub data: Option<T>,
    pub message: Option<String>,
}
    // success: always true or false, for easy client-side checks
    // data: the actual data you return (like an OwnerResponse or a list)
    // message: optional string messages, e.g., “Owner created successfully.”
    // Using Option<T> lets us omit data or message when unnecessary

impl<T: Serialize> JsonApiResponse<T> {
    // when we will have a sucessful operation, and some data to send back
    // success(data): wraps some data in a 200 OK JSON response, without messages
    #[allow(dead_code)]
    pub fn success(data: T) -> HttpResponse {
        HttpResponse::Ok().json(
            Self{
                success: true,
                data: Some(data),
                message: None,
            }
        ) 
    }
}

impl JsonApiResponse<()> {
    // when we have a successful operation, without data to send back 
    // message(msg): returns a 200 code OK with a message instead of a complex payload(object) 
    #[allow(dead_code)]
    pub fn with_message(message: &str) -> HttpResponse {
        HttpResponse::Ok().json(
            Self{
                success: true,
                data: None,
                message: Some(message.to_string()),
            })
    }
}



// Error messages without data
#[derive(Debug, Serialize)]
pub struct ErrorJsonApiResponse {
    pub error: String,
}

    // Why a separate Error type ?
    // Error responses usually don’t include data.
    // Clients can rely on a known structure: { "error": "Something went wrong" }.

impl ErrorJsonApiResponse {
    // Using a new() method in Rust is a convention that usually means:
    //           “Give me a default or typical instance of this type.”
    // We’re saying:  “This is the most common/default error we return — a 400 Bad Request.”
    // It makes calling it simple and semantic:
    
    #[allow(dead_code)]
    pub fn bad_request(msg: &str) -> HttpResponse {
        HttpResponse::BadRequest().json(
            ErrorJsonApiResponse {
                error: msg.to_string(),
            }
        )
    }

    #[allow(dead_code)]
    pub fn internal_server_error(err: &str) -> HttpResponse {
        HttpResponse::InternalServerError().json(
            ErrorJsonApiResponse{
                error: err.to_string(),
            }
        )
    }

    #[allow(dead_code)]
    pub fn not_found(err: &str) -> HttpResponse {
        HttpResponse::NotFound().json(
            ErrorJsonApiResponse{
                error: err.to_string(),
            }
        )
    }

    #[allow(dead_code)]
    pub fn from_db_error(err: mongodb::error::Error) -> HttpResponse {
        ErrorJsonApiResponse::internal_server_error(&err.to_string())
    }




}
