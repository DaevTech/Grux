use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::{Request, Response};
use hyper::body::Bytes;
use crate::grux_configuration_struct::Sites;
use crate::grux_http_util::{full};
use crate::grux_database::{LoginRequest, authenticate_user, create_session, verify_session_token, invalidate_session};
use log::{info, error, debug};
use serde_json;


pub async fn handle_login_request(req: Request<hyper::body::Incoming>, _admin_site: &Sites) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // Check if this is a POST request
    if req.method() != hyper::Method::POST {
        let mut resp = Response::new(full("Method not allowed"));
        *resp.status_mut() = hyper::StatusCode::METHOD_NOT_ALLOWED;
        return Ok(resp);
    }

    // Read the request body
    let body_bytes = match req.collect().await {
        Ok(body) => body.to_bytes(),
        Err(_) => {
            let mut resp = Response::new(full("Failed to read request body"));
            *resp.status_mut() = hyper::StatusCode::BAD_REQUEST;
            return Ok(resp);
        }
    };

    // Parse JSON body
    let login_request: LoginRequest = match serde_json::from_slice(&body_bytes) {
        Ok(req) => req,
        Err(e) => {
            error!("Failed to parse login request: {}", e);
            let mut resp = Response::new(full("Invalid JSON format"));
            *resp.status_mut() = hyper::StatusCode::BAD_REQUEST;
            return Ok(resp);
        }
    };

    debug!("Login attempt for username: {}", login_request.username);

    // Authenticate user
    let user = match authenticate_user(&login_request.username, &login_request.password) {
        Ok(Some(user)) => user,
        Ok(None) => {
            info!("Failed login attempt for username: {}", login_request.username);
            let mut resp = Response::new(full(r#"{"error": "Invalid username or password"}"#));
            *resp.status_mut() = hyper::StatusCode::UNAUTHORIZED;
            resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
            return Ok(resp);
        }
        Err(e) => {
            error!("Database error during authentication: {}", e);
            let mut resp = Response::new(full(r#"{"error": "Internal server error"}"#));
            *resp.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
            resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
            return Ok(resp);
        }
    };

    // Create session
    let session = match create_session(&user) {
        Ok(session) => session,
        Err(e) => {
            error!("Failed to create session: {}", e);
            let mut resp = Response::new(full(r#"{"error": "Failed to create session"}"#));
            *resp.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
            resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
            return Ok(resp);
        }
    };

    info!("Successful login for user: {}", user.username);

    // Return success response with session token
    let response_json = serde_json::json!({
        "success": true,
        "message": "Login successful",
        "session_token": session.token,
        "username": session.username,
        "expires_at": session.expires_at.to_rfc3339()
    });

    let mut resp = Response::new(full(response_json.to_string()));
    *resp.status_mut() = hyper::StatusCode::OK;
    resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
    Ok(resp)
}

pub async fn handle_logout_request(req: Request<hyper::body::Incoming>, _admin_site: &Sites) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // Check if this is a POST request
    if req.method() != hyper::Method::POST {
        let mut resp = Response::new(full("Method not allowed"));
        *resp.status_mut() = hyper::StatusCode::METHOD_NOT_ALLOWED;
        return Ok(resp);
    }

    // Get the session token from Authorization header or request body
    let token = get_session_token_from_request(&req).await;

    if let Some(token) = token {
        match invalidate_session(&token) {
            Ok(true) => {
                info!("Successfully logged out session");
                let response_json = serde_json::json!({
                    "success": true,
                    "message": "Logout successful"
                });
                let mut resp = Response::new(full(response_json.to_string()));
                *resp.status_mut() = hyper::StatusCode::OK;
                resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
                Ok(resp)
            }
            Ok(false) => {
                let mut resp = Response::new(full(r#"{"error": "Session not found"}"#));
                *resp.status_mut() = hyper::StatusCode::NOT_FOUND;
                resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
                Ok(resp)
            }
            Err(e) => {
                error!("Failed to logout session: {}", e);
                let mut resp = Response::new(full(r#"{"error": "Internal server error"}"#));
                *resp.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
                resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
                Ok(resp)
            }
        }
    } else {
        let mut resp = Response::new(full(r#"{"error": "No session token provided"}"#));
        *resp.status_mut() = hyper::StatusCode::BAD_REQUEST;
        resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
        Ok(resp)
    }
}

pub fn admin_get_configuration_endpoint(_req: &Request<hyper::body::Incoming>, _admin_site: &Sites) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // Here we can handle the get configuration requests
    let mut resp = Response::new(full("Get configuration endpoint not implemented yet"));
    *resp.status_mut() = hyper::StatusCode::OK;
    Ok(resp)
}

pub fn admin_post_configuration_endpoint(_req: &Request<hyper::body::Incoming>, _admin_site: &Sites) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // Here we can handle the post configuration requests
    let mut resp = Response::new(full("Post configuration endpoint not implemented yet"));
    *resp.status_mut() = hyper::StatusCode::OK;
    Ok(resp)
}

// Helper function to extract session token from request
async fn get_session_token_from_request(req: &Request<hyper::body::Incoming>) -> Option<String> {
    // First, check for Authorization header (Bearer token)
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }

    // If no Authorization header, try to read from request body (for POST requests)
    if req.method() == hyper::Method::POST {
        // We can't consume the body here without cloning/moving it
        // For now, we'll only support Authorization header
        // In a full implementation, you might want to parse the body for a token field
    }

    None
}

// Helper function to verify session token and return session info
pub fn verify_session(token: &str) -> Result<Option<crate::grux_database::Session>, String> {
    verify_session_token(token)
}

// Middleware-like function to check if request is authenticated
pub async fn require_authentication(req: &Request<hyper::body::Incoming>) -> Result<Option<crate::grux_database::Session>, Response<BoxBody<Bytes, hyper::Error>>> {
    let token = get_session_token_from_request(req).await;

    if let Some(token) = token {
        match verify_session(&token) {
            Ok(Some(session)) => Ok(Some(session)),
            Ok(None) => {
                let mut resp = Response::new(full(r#"{"error": "Invalid or expired session"}"#));
                *resp.status_mut() = hyper::StatusCode::UNAUTHORIZED;
                resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
                Err(resp)
            }
            Err(e) => {
                error!("Failed to verify session: {}", e);
                let mut resp = Response::new(full(r#"{"error": "Internal server error"}"#));
                *resp.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
                resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
                Err(resp)
            }
        }
    } else {
        let mut resp = Response::new(full(r#"{"error": "Authentication required"}"#));
        *resp.status_mut() = hyper::StatusCode::UNAUTHORIZED;
        resp.headers_mut().insert("Content-Type", "application/json".parse().unwrap());
        Err(resp)
    }
}
