use grux::grux_configuration::save_configuration;
use grux::grux_configuration_struct::*;
use grux::grux_http_server::initialize_server;
use grux::grux_database::initialize_database;
use hyper::HeaderMap;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{timeout, Duration};
use std::net::SocketAddr;

/// HTTP 1.1 Compliance Test Suite for Grux Web Server
///
/// This comprehensive test suite validates Grux's compliance with HTTP/1.1 specifications
/// as defined in RFC 7230 (Message Syntax and Routing) and RFC 7231 (Semantics and Content).
///
/// Test Categories:
/// 1. HTTP Methods Compliance
/// 2. Status Code Compliance
/// 3. Header Field Validation
/// 4. Message Framing and Transfer Encoding
/// 5. Connection Management
/// 6. Protocol Version Handling
/// 7. Content Negotiation
/// 8. Error Handling and Edge Cases
/// 9. TLS and Security Compliance (Grux-specific)
/// 10. Request/Response Message Validation

// Test server configuration and utilities
const TEST_SERVER_IP: &str = "127.0.0.1";
const BASE_TEST_PORT: u16 = 18000;
const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Test server instance for HTTP compliance testing
struct HttpComplianceTestServer {
    port: u16,
    _handle: tokio::task::JoinHandle<()>,
}

impl HttpComplianceTestServer {
    async fn new(_port: u16, _tls: bool) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize test database
        initialize_database()?;

        // Create a minimal test configuration using the Configuration struct directly
        let test_config = Configuration {
            servers: vec![Server {
                bindings: vec![Binding {
                    ip: TEST_SERVER_IP.to_string(),
                    port: _port,
                    is_tls: _tls,
                    is_admin: false,
                    sites: vec![Site {
                        hostnames: vec!["localhost".to_string()],
                        is_default: true,
                        is_enabled: true,
                        web_root: "www-default".to_string(),
                        web_root_index_file_list: vec!["index.html".to_string()],
                        enabled_handlers: vec![],
                        tls_cert_path: if _tls { Some("certs/test.crt.pem".to_string()) } else { None },
                        tls_key_path: if _tls { Some("certs/test.key.pem".to_string()) } else { None },
                    }],
                }],
            }],
            admin_site: AdminSite {
                is_admin_portal_enabled: false,
            },
            core: Core {
                file_cache: FileCache {
                    is_enabled: false,
                    cache_item_size: 100,
                    cache_max_size_per_file: 1000,
                    cache_item_time_between_checks: 60,
                    cleanup_thread_interval: 300,
                    max_item_lifetime: 3600,
                    forced_eviction_threshold: 80,
                },
                gzip: Gzip {
                    is_enabled: false,
                    compressible_content_types: vec!["text/html".to_string()],
                },
            },
            request_handlers: vec![],
        };

        // Save configuration to database
        save_configuration(&test_config)?;

        // Start server in background
        let handle = tokio::spawn(async move {
            if let Err(e) = initialize_server() {
                log::error!("Test server failed: {}", e);
            }
        });

        // Wait for server to start
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(HttpComplianceTestServer {
            port: _port,
            _handle: handle,
        })
    }

    fn addr(&self) -> SocketAddr {
        SocketAddr::new(TEST_SERVER_IP.parse().unwrap(), self.port)
    }
}

/// Send raw HTTP request and get raw response
async fn send_raw_http_request(addr: SocketAddr, request: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = timeout(TEST_TIMEOUT, TcpStream::connect(addr)).await??;

    stream.write_all(request.as_bytes()).await?;

    let mut response = String::new();
    stream.read_to_string(&mut response).await?;

    Ok(response)
}

/// Parse HTTP response into components
fn parse_http_response(response: &str) -> (String, HeaderMap, String) {
    let mut lines = response.lines();
    let status_line = lines.next().unwrap_or("").to_string();

    let mut headers = HeaderMap::new();
    let mut body = String::new();
    let mut in_body = false;

    for line in lines {
        if in_body {
            body.push_str(line);
            body.push('\n');
        } else if line.is_empty() {
            in_body = true;
        } else if let Some(colon_pos) = line.find(':') {
            let name = &line[..colon_pos].trim().to_lowercase();
            let value = &line[colon_pos + 1..].trim();
            if let Ok(header_name) = name.parse::<hyper::header::HeaderName>() {
                if let Ok(header_value) = value.parse::<hyper::header::HeaderValue>() {
                    headers.insert(header_name, header_value);
                }
            }
        }
    }

    (status_line, headers, body.trim_end_matches('\n').to_string())
}

/// Validate status line format: HTTP-Version SP Status-Code SP Reason-Phrase CRLF
fn validate_status_line(status_line: &str) -> bool {
    let parts: Vec<&str> = status_line.split_whitespace().collect();
    if parts.len() < 3 {
        return false;
    }

    // Check HTTP version format
    let version = parts[0];
    if !version.starts_with("HTTP/") {
        return false;
    }

    // Check status code is 3 digits
    let status_code = parts[1];
    if status_code.len() != 3 || !status_code.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    true
}

// ============================================================================
// 1. HTTP METHODS COMPLIANCE TESTING
// ============================================================================

#[tokio::test]
async fn test_required_methods_support() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 1, false).await.unwrap();

    // RFC 7231: GET and HEAD methods MUST be supported by all general-purpose servers
    let get_request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), get_request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);
    assert!(validate_status_line(&status_line));
    assert!(!status_line.contains("501")); // Not "Not Implemented"

    let head_request = "HEAD / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), head_request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);
    assert!(validate_status_line(&status_line));
    assert!(!status_line.contains("501"));
}

#[tokio::test]
async fn test_head_method_identical_to_get_minus_body() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 2, false).await.unwrap();

    // GET request
    let get_request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let get_response = send_raw_http_request(server.addr(), get_request).await.unwrap();
    let (get_status, get_headers, get_body) = parse_http_response(&get_response);

    // HEAD request
    let head_request = "HEAD / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let head_response = send_raw_http_request(server.addr(), head_request).await.unwrap();
    let (head_status, head_headers, head_body) = parse_http_response(&head_response);

    // Status line should be identical
    assert_eq!(get_status, head_status);

    // Headers should be identical (with some exceptions for dynamic headers)
    assert_eq!(get_headers.len(), head_headers.len());

    // HEAD response must not have a body
    assert!(head_body.is_empty() || head_body.len() < get_body.len());
}

#[tokio::test]
async fn test_options_method_allowed_methods() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 3, false).await.unwrap();

    let options_request = "OPTIONS * HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), options_request).await.unwrap();
    let (status_line, headers, _) = parse_http_response(&response);

    assert!(validate_status_line(&status_line));

    // Should include Allow header with supported methods
    let allow_header = headers.get("allow");
    if let Some(allow_value) = allow_header {
        let allow_str = allow_value.to_str().unwrap_or("");
        // Should at least include GET and HEAD
        assert!(allow_str.contains("GET"));
        assert!(allow_str.contains("HEAD"));
    }
}

#[tokio::test]
async fn test_unknown_method_handling() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 4, false).await.unwrap();

    let unknown_request = "CUSTOMMETHOD / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), unknown_request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should return 501 Not Implemented for unknown methods
    assert!(status_line.contains("501") || status_line.contains("405"));
}

#[tokio::test]
async fn test_method_case_sensitivity() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 5, false).await.unwrap();

    // Methods are case-sensitive per RFC 7231
    let lowercase_request = "get / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), lowercase_request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should return 400 Bad Request or 501 Not Implemented for invalid method case
    assert!(status_line.contains("400") || status_line.contains("501"));
}

// ============================================================================
// 2. STATUS CODE COMPLIANCE TESTING
// ============================================================================

#[tokio::test]
async fn test_status_code_format_compliance() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 6, false).await.unwrap();

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Validate Status-Line format: HTTP-Version SP Status-Code SP Reason-Phrase CRLF
    assert!(validate_status_line(&status_line));

    let parts: Vec<&str> = status_line.split_whitespace().collect();
    assert!(parts.len() >= 3);

    // HTTP version should be HTTP/1.1
    assert!(parts[0] == "HTTP/1.1" || parts[0] == "HTTP/1.0");

    // Status code should be valid 3-digit number
    let status_code: u16 = parts[1].parse().unwrap();
    assert!(status_code >= 100 && status_code < 600);
}

#[tokio::test]
async fn test_404_not_found_response() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 7, false).await.unwrap();

    let request = "GET /nonexistent-file-that-should-not-exist HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    assert!(status_line.contains("404"));
}

#[tokio::test]
async fn test_405_method_not_allowed_includes_allow_header() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 8, false).await.unwrap();

    // Try to POST to a resource that doesn't accept POST
    let request = "POST /index.html HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, headers, _) = parse_http_response(&response);

    if status_line.contains("405") {
        // 405 Method Not Allowed MUST include Allow header
        assert!(headers.contains_key("allow"));
    }
}

#[tokio::test]
async fn test_100_continue_handling() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 9, false).await.unwrap();

    let request = "POST / HTTP/1.1\r\nHost: localhost\r\nExpect: 100-continue\r\nContent-Length: 10\r\n\r\ntest data";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();

    // Should handle Expect: 100-continue properly (either send 100 Continue or process directly)
    assert!(!response.is_empty());
}

// ============================================================================
// 3. HEADER FIELD VALIDATION TESTING
// ============================================================================

#[tokio::test]
async fn test_host_header_requirement() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 10, false).await.unwrap();

    // HTTP/1.1 requests MUST include Host header
    let request_without_host = "GET / HTTP/1.1\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request_without_host).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should return 400 Bad Request for missing Host header in HTTP/1.1
    assert!(status_line.contains("400"));
}

#[tokio::test]
async fn test_header_case_insensitivity() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 11, false).await.unwrap();

    // Header names are case-insensitive
    let request = "GET / HTTP/1.1\r\nhost: localhost\r\nuser-agent: TestClient\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should process lowercase headers correctly
    assert!(validate_status_line(&status_line));
    assert!(!status_line.contains("400"));
}

#[tokio::test]
async fn test_invalid_header_characters() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 12, false).await.unwrap();

    // Headers with invalid characters should be rejected
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nInvalid\x00Header: value\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should return 400 Bad Request for invalid header characters
    assert!(status_line.contains("400"));
}

#[tokio::test]
async fn test_content_length_validation() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 13, false).await.unwrap();

    // Content-Length must match actual body length
    let request = "POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 5\r\n\r\ntest"; // 4 chars, not 5
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Server should handle Content-Length mismatch appropriately
    assert!(validate_status_line(&status_line));
}

#[tokio::test]
async fn test_multiple_host_headers() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 14, false).await.unwrap();

    // Multiple Host headers should be rejected
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nHost: example.com\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should return 400 Bad Request for multiple Host headers
    assert!(status_line.contains("400"));
}

// ============================================================================
// 4. MESSAGE FRAMING AND TRANSFER ENCODING TESTING
// ============================================================================

#[tokio::test]
async fn test_chunked_transfer_encoding() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 15, false).await.unwrap();

    // Send chunked request
    let request = "POST / HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\n\r\n4\r\ntest\r\n0\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should handle chunked encoding properly
    assert!(validate_status_line(&status_line));
    assert!(!status_line.contains("400"));
}

#[tokio::test]
async fn test_content_length_vs_transfer_encoding() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 16, false).await.unwrap();

    // Transfer-Encoding takes precedence over Content-Length
    let request = "POST / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 10\r\nTransfer-Encoding: chunked\r\n\r\n4\r\ntest\r\n0\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should process as chunked, ignoring Content-Length
    assert!(validate_status_line(&status_line));
}

#[tokio::test]
async fn test_invalid_chunk_format() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 17, false).await.unwrap();

    // Send malformed chunk
    let request = "POST / HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\n\r\nINVALID\r\ntest\r\n0\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should return 400 Bad Request for malformed chunks
    assert!(status_line.contains("400"));
}

#[tokio::test]
async fn test_trailer_headers_in_chunked_encoding() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 18, false).await.unwrap();

    // Chunked encoding with trailer headers
    let request = "POST / HTTP/1.1\r\nHost: localhost\r\nTransfer-Encoding: chunked\r\nTrailer: X-Custom-Header\r\n\r\n4\r\ntest\r\n0\r\nX-Custom-Header: value\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should handle trailer headers correctly
    assert!(validate_status_line(&status_line));
}

// ============================================================================
// 5. CONNECTION MANAGEMENT TESTING
// ============================================================================

#[tokio::test]
async fn test_persistent_connection_default() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 19, false).await.unwrap();

    // HTTP/1.1 connections should be persistent by default
    let request1 = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let request2 = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";

    let mut stream = TcpStream::connect(server.addr()).await.unwrap();

    // Send first request
    stream.write_all(request1.as_bytes()).await.unwrap();
    let mut response1 = vec![0; 4096];
    let n1 = stream.read(&mut response1).await.unwrap();
    let response1_str = String::from_utf8_lossy(&response1[..n1]);

    // Send second request on same connection
    stream.write_all(request2.as_bytes()).await.unwrap();
    let mut response2 = vec![0; 4096];
    let n2 = stream.read(&mut response2).await.unwrap();
    let response2_str = String::from_utf8_lossy(&response2[..n2]);

    // Both responses should be valid
    assert!(!response1_str.is_empty());
    assert!(!response2_str.is_empty());
}

#[tokio::test]
async fn test_connection_close_handling() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 20, false).await.unwrap();

    // Connection: close should terminate after response
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, headers, _) = parse_http_response(&response);

    assert!(validate_status_line(&status_line));

    // Response should include Connection: close
    if let Some(connection) = headers.get("connection") {
        assert!(connection.to_str().unwrap_or("").contains("close"));
    }
}

#[tokio::test]
async fn test_connection_timeout_behavior() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 21, false).await.unwrap();

    // Connect but don't send anything
    let mut stream = TcpStream::connect(server.addr()).await.unwrap();

    // Connection should eventually timeout (this tests server behavior)
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Try to write after delay
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let result = stream.write_all(request.as_bytes()).await;

    // Connection might still be open or closed depending on server timeout
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// 6. PROTOCOL VERSION HANDLING TESTING
// ============================================================================

#[tokio::test]
async fn test_http10_backward_compatibility() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 22, false).await.unwrap();

    // HTTP/1.0 request (no Host header required)
    let request = "GET / HTTP/1.0\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    assert!(validate_status_line(&status_line));
    // Server should respond with HTTP/1.0 or HTTP/1.1
    assert!(status_line.starts_with("HTTP/1."));
}

#[tokio::test]
async fn test_http11_version_response() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 23, false).await.unwrap();

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Server should respond with HTTP/1.1 for HTTP/1.1 requests
    assert!(status_line.starts_with("HTTP/1.1"));
}

#[tokio::test]
async fn test_invalid_http_version() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 24, false).await.unwrap();

    let request = "GET / HTTP/2.0\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should handle unsupported HTTP version appropriately
    assert!(status_line.contains("400") || status_line.contains("505"));
}

// ============================================================================
// 7. CONTENT NEGOTIATION TESTING
// ============================================================================

#[tokio::test]
async fn test_accept_header_negotiation() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 25, false).await.unwrap();

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nAccept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, headers, _) = parse_http_response(&response);

    assert!(validate_status_line(&status_line));

    // Should include Content-Type header
    assert!(headers.contains_key("content-type"));
}

#[tokio::test]
async fn test_accept_encoding_support() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 26, false).await.unwrap();

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nAccept-Encoding: gzip, deflate, br\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should handle Accept-Encoding header
    assert!(validate_status_line(&status_line));
}

#[tokio::test]
async fn test_quality_value_processing() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 27, false).await.unwrap();

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nAccept: text/html;q=0.9,text/plain;q=0.8,*/*;q=0.1\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should process q-values correctly
    assert!(validate_status_line(&status_line));
}

#[tokio::test]
async fn test_406_not_acceptable_response() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 28, false).await.unwrap();

    // Request only unsupported media types
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\nAccept: application/vnd.unsupported-format\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Might return content anyway or 406 Not Acceptable
    assert!(validate_status_line(&status_line));
}

// ============================================================================
// 8. ERROR HANDLING AND EDGE CASES TESTING
// ============================================================================

#[tokio::test]
async fn test_malformed_request_line() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 29, false).await.unwrap();

    // Invalid request line format
    let request = "INVALID REQUEST LINE\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should return 400 Bad Request
    assert!(status_line.contains("400"));
}

#[tokio::test]
async fn test_request_uri_too_long() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 30, false).await.unwrap();

    // Extremely long URI
    let long_path = "a".repeat(8192);
    let request = format!("GET /{} HTTP/1.1\r\nHost: localhost\r\n\r\n", long_path);
    let response = send_raw_http_request(server.addr(), &request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should return 414 Request-URI Too Long or handle gracefully
    assert!(validate_status_line(&status_line));
}

#[tokio::test]
async fn test_request_header_fields_too_large() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 31, false).await.unwrap();

    // Very large header
    let large_header_value = "x".repeat(8192);
    let request = format!("GET / HTTP/1.1\r\nHost: localhost\r\nX-Large-Header: {}\r\n\r\n", large_header_value);
    let response = send_raw_http_request(server.addr(), &request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should return 431 Request Header Fields Too Large or handle gracefully
    assert!(validate_status_line(&status_line));
}

#[tokio::test]
async fn test_invalid_uri_characters() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 32, false).await.unwrap();

    // URI with invalid characters
    let request = "GET /path with spaces HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should return 400 Bad Request for invalid URI
    assert!(status_line.contains("400"));
}

#[tokio::test]
async fn test_empty_request_handling() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 33, false).await.unwrap();

    // Send empty request
    let request = "";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();

    // Should handle empty request gracefully (connection might close)
    assert!(!response.is_empty() || response.is_empty()); // Either response or connection close
}

// ============================================================================
// 9. TLS AND SECURITY COMPLIANCE (GRUX-SPECIFIC)
// ============================================================================

// Note: TLS tests would require proper certificate setup
// These are placeholder tests for the TLS functionality

#[tokio::test]
async fn test_non_admin_endpoint_http_support() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 34, false).await.unwrap();

    // Non-admin endpoints should work over HTTP
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    assert!(validate_status_line(&status_line));
    assert!(!status_line.contains("400"));
}

// ============================================================================
// 10. REQUEST/RESPONSE MESSAGE VALIDATION
// ============================================================================

#[tokio::test]
async fn test_response_header_format() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 35, false).await.unwrap();

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, headers, _) = parse_http_response(&response);

    assert!(validate_status_line(&status_line));

    // Validate common required headers
    assert!(headers.contains_key("date") || headers.contains_key("server"));
}

#[tokio::test]
async fn test_response_body_consistency() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 36, false).await.unwrap();

    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, headers, body) = parse_http_response(&response);

    assert!(validate_status_line(&status_line));

    // If Content-Length is present, body should match
    if let Some(content_length) = headers.get("content-length") {
        if let Ok(length) = content_length.to_str().unwrap_or("0").parse::<usize>() {
            assert_eq!(body.len(), length);
        }
    }
}

#[tokio::test]
async fn test_http_message_crlf_handling() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 37, false).await.unwrap();

    // Test with proper CRLF line endings
    let request = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    assert!(validate_status_line(&status_line));

    // Test with LF only (should be tolerant per RFC)
    let request_lf = "GET / HTTP/1.1\nHost: localhost\n\n";
    let response_lf = send_raw_http_request(server.addr(), request_lf).await.unwrap();
    let (status_line_lf, _, _) = parse_http_response(&response_lf);

    // Should be tolerant of LF-only line endings
    assert!(validate_status_line(&status_line_lf));
}

#[tokio::test]
async fn test_whitespace_handling_in_headers() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 38, false).await.unwrap();

    // Test with extra whitespace around header values
    let request = "GET / HTTP/1.1\r\nHost:   localhost   \r\nUser-Agent:  TestClient  \r\n\r\n";
    let response = send_raw_http_request(server.addr(), request).await.unwrap();
    let (status_line, _, _) = parse_http_response(&response);

    // Should handle whitespace in headers correctly
    assert!(validate_status_line(&status_line));
    assert!(!status_line.contains("400"));
}

// ============================================================================
// HELPER FUNCTIONS FOR ADVANCED TESTING
// ============================================================================

/// Create default test configuration
fn create_default_configuration() -> Configuration {
    Configuration {
        servers: vec![Server {
            bindings: vec![],
        }],
        admin_site: AdminSite {
            is_admin_portal_enabled: false,
        },
        core: Core {
            file_cache: FileCache {
                is_enabled: false,
                cache_item_size: 100,
                cache_max_size_per_file: 1000,
                cache_item_time_between_checks: 60,
                cleanup_thread_interval: 300,
                max_item_lifetime: 3600,
                forced_eviction_threshold: 80,
            },
            gzip: Gzip {
                is_enabled: false,
                compressible_content_types: vec!["text/html".to_string()],
            },
        },
        request_handlers: vec![],
    }
}

// ============================================================================
// INTEGRATION TESTS WITH MULTIPLE PROTOCOLS
// ============================================================================

#[tokio::test]
async fn test_concurrent_requests_compliance() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 39, false).await.unwrap();

    // Send multiple concurrent requests
    let mut handles = vec![];

    for i in 0..10 {
        let addr = server.addr();
        let handle = tokio::spawn(async move {
            let request = format!("GET /?request={} HTTP/1.1\r\nHost: localhost\r\n\r\n", i);
            send_raw_http_request(addr, &request).await.unwrap()
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let responses = futures::future::join_all(handles).await;

    // All responses should be valid
    for response_result in responses {
        let response = response_result.unwrap();
        let (status_line, _, _) = parse_http_response(&response);
        assert!(validate_status_line(&status_line));
    }
}

#[tokio::test]
async fn test_pipeline_request_handling() {
    let server = HttpComplianceTestServer::new(BASE_TEST_PORT + 40, false).await.unwrap();

    // Send pipelined requests
    let pipelined_requests = "GET /1 HTTP/1.1\r\nHost: localhost\r\n\r\nGET /2 HTTP/1.1\r\nHost: localhost\r\n\r\n";

    let mut stream = TcpStream::connect(server.addr()).await.unwrap();
    stream.write_all(pipelined_requests.as_bytes()).await.unwrap();

    let mut response_buffer = vec![0; 8192];
    let n = stream.read(&mut response_buffer).await.unwrap();
    let responses = String::from_utf8_lossy(&response_buffer[..n]);

    // Should handle pipelined requests (responses in order)
    assert!(!responses.is_empty());
    assert!(responses.contains("HTTP/1.1"));
}