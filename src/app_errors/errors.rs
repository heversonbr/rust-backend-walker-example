


// In a typical web application like yours, using Actix-Web + MongoDB, you interact with multiple domains of failure, each originating at different layers
// We usually have 3  “Layers” of Errors: 
//
//  1. Low-Level / System Errors
//     std::io::Error → e.g., file access failure, networking issues.
//     	•	bson::oid::Error → when parsing ObjectIds.
//     	•	serde_json::Error → for (de)serialization problems.
//     	•	chrono::ParseError → date-time parsing.
//     	•	mongodb::error::Error → internal database failures (e.g., connection dropped, invalid query, etc).
//     
//     These are library-defined, often verbose, and not ideal to expose directly to your API consumers.
//     
// 2. Your Application Errors – Custom Enum
//      This is where you take control.
//      You define a custom enum, like AppError, that wraps and translates low-level errors into a consistent form.
//
// 3. Client-Facing HTTP Errors
// 
// At the outermost layer — in your Actix-Web handlers — you map your AppError into:
//      HttpResponse::BadRequest().json(ApiError { ... })
//      HttpResponse::InternalServerError().json(ApiError { ... })
// This ensures your API always responds with consistent JSON error messages (e.g., { "error": "Owner not found" }), and avoids leaking internal errors to clients.

//  Best Practices
//  	1.	Never expose raw MongoDB or std::io errors to the client.
//  	2.	Use your own AppError enum to:
//  	•	Cleanly convert all internal errors.
//  	•	Decide how to log, wrap, and respond.
//  	3.	In routes, convert AppError to JSON responses (with correct status codes).

use std::fmt;

#[allow(dead_code)]
// creating an enum to represent the different kinds of errors your app might encounter
#[derive(Debug)]
pub enum AppError {
    DatabaseError(String),
    InvalidId,
    NotFound,
    ParseError(String),
    InternalError,
}

// Implementing the Display trait to allow the control how your error appears when printed or logged
// Required to implement std::error::Error (next step).
// Controls what users/devs see in logs, terminal, or HTTP responses.
impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::InvalidId => write!(f, "Invalid ID format"),
            AppError::NotFound => write!(f, "Item not found"),
            AppError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AppError::InternalError => write!(f, "Internal server error"),
        }
    }
}

// Implement std::error::Error for your error type
// this lets your error type participate in Rust’s standard error system
// It lets you use AppError in any place that expects a type implementing Error, including standard Result<T, E> patterns
// So, when you implement it you’re telling the compiler: Hey, AppError behaves like a normal Rust error. You can use it anywhere a generic Error trait is expected.”
impl std::error::Error for AppError {}


// Implement From for other error types (this is optional but powerful)
// It allows seamless conversion from lower-level errors to your AppError, which makes the operator '?' work in your functions to convert lower-level errors to oour AppError
// the idea is to implement From<T> for each external or lower-level error type you expect to work with and want to convert automatically into your AppError.
// can (and should) implement From for any error type you’re handling in your code if:
// a) we commonly use the '?' operator with that type
// b) we want uniform error handling
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

impl From<bson::oid::Error> for AppError {
    fn from(err: bson::oid::Error) -> Self {
        AppError::ParseError(err.to_string())
    }
}

impl From<mongodb::error::Error> for AppError {
    fn from(err: mongodb::error::Error) -> Self {
        AppError::DatabaseError(err.to_string())
    }
}

// impl From<dyn serde::de::Error> for AppError {
//     fn from(err: serde_json::Error) -> Self {
//         AppError::ParseError(err.to_string())
//     }
// }

