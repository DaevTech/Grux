use grux::grux_http::file_pattern_matching::{get_blocked_file_pattern_matching, get_whitelisted_file_pattern_matching};

#[test]
fn test_blocked_file_pattern_matching() {
    let blocked_matching = get_blocked_file_pattern_matching();
    assert!(blocked_matching.is_file_pattern_blocked("index.php"));
    assert!(blocked_matching.is_file_pattern_blocked("test.tmp"));
    assert!(blocked_matching.is_file_pattern_blocked(".env"));
    assert!(blocked_matching.is_file_pattern_blocked(".env.example"));
    assert!(blocked_matching.is_file_pattern_blocked(".web.config"));
    assert!(blocked_matching.is_file_pattern_blocked("web.config"));
    assert!(!blocked_matching.is_file_pattern_blocked("index.html"));
    assert!(!blocked_matching.is_file_pattern_blocked("index.css"));
    assert!(blocked_matching.is_file_pattern_blocked("index.php.bak"));
    assert!(blocked_matching.is_file_pattern_blocked("mylog.log"));
    assert!(blocked_matching.is_file_pattern_blocked(".DS_Store"));
    assert!(blocked_matching.is_file_pattern_blocked(".whatever"));
}

#[test]
fn test_whitelisted_file_pattern_matching() {
    let whitelisted_matching = get_whitelisted_file_pattern_matching();
    assert!(whitelisted_matching.is_file_pattern_whitelisted("/var/www/html/.well-known/acme-challenge/token"));
    assert!(!whitelisted_matching.is_file_pattern_whitelisted("/var/www/html/.DS_STORE"));
    assert!(!whitelisted_matching.is_file_pattern_whitelisted("/var/www/html/.env"));

}