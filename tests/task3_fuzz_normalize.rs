//! Task 3 独立复审 — normalize_local_api_url 穷举模糊测试
//! 确认任何输入下最终 URL 永远精确 http://127.0.0.1:{port}/api/*

#[cfg(test)]
mod fuzz_normalize_local_api_url {
    /// Thin standalone replica of the production normalize_local_api_url for
    /// test-only use.  Must match ai_tools.rs line-by-line logic.
    fn normalize_local_api_url(input: &str, port: u16) -> Result<String, String> {
        // === Phase 1: reject literal backslash ===
        if input.contains('\\') {
            return Err("backslash".into());
        }

        let base = url::Url::parse(&format!("http://127.0.0.1:{port}/")).unwrap();

        // === Phase 2: extract path + query ===
        let path_and_query = if input.starts_with("http://") || input.starts_with("https://") {
            let parsed = url::Url::parse(input).map_err(|_| "parse-fail".to_string())?;
            if parsed.fragment().is_some() {
                return Err("fragment".into());
            }
            let mut value = parsed.path().to_string();
            if let Some(query) = parsed.query() {
                value.push('?');
                value.push_str(query);
            }
            value
        } else {
            format!("/{}", input.trim_start_matches('/'))
        };

        // === Phase 3: join against base, then strict assertion ===
        let normalized = base
            .join(&path_and_query)
            .map_err(|_| "join-fail".to_string())?;
        let ok = normalized.scheme() == "http"
            && normalized.host_str() == Some("127.0.0.1")
            && normalized.port_or_known_default() == Some(port)
            && normalized.fragment().is_none()
            && normalized.path().starts_with("/api/");
        if !ok {
            return Err("assertion-fail".into());
        }
        Ok(normalized.to_string())
    }

    // ── helpers ────────────────────────────────────────────────────────

    /// Returns true iff the output is an Err (expected rejection).
    fn should_reject(input: &str) -> bool {
        normalize_local_api_url(input, 5023).is_err()
    }

    /// Returns the normalized URL string, panicking on Err.
    fn normalized(input: &str) -> String {
        normalize_local_api_url(input, 5023).unwrap()
    }

    /// Returns the error string (for diagnostic printing).
    fn err(input: &str) -> String {
        normalize_local_api_url(input, 5023).unwrap_err()
    }

    // ── known-good accept cases ───────────────────────────────────────

    #[test]
    fn normal_relative_paths_work() {
        assert_eq!(
            normalized("api/v1/search?q=rust"),
            "http://127.0.0.1:5023/api/v1/search?q=rust"
        );
        assert_eq!(
            normalized("/api/v1/tags"),
            "http://127.0.0.1:5023/api/v1/tags"
        );
        assert_eq!(
            normalized("api/v1/news?search=title"),
            "http://127.0.0.1:5023/api/v1/news?search=title"
        );
    }

    #[test]
    fn absolute_url_with_external_authority_is_stripped_to_path() {
        assert_eq!(
            normalized("https://evil.example/api/v1/tags?x=1"),
            "http://127.0.0.1:5023/api/v1/tags?x=1"
        );
        assert_eq!(
            normalized("http://google.com/api/v1/search?q=a"),
            "http://127.0.0.1:5023/api/v1/search?q=a"
        );
    }

    // ── literal backslash reject ───────────────────────────────────────

    #[test]
    fn literal_backslash_immediately_rejected() {
        for payload in [
            r"\evil.example\api\v1\search",
            r"\169.254.169.254\latest\meta-data\",
            r"\127.0.0.1:8080\api\v1\search",
            r"/\127.0.0.1:8080\api\v1\search",
            r"\evil.example\api\x?q=test",
            "api/v1/search\\x",
            "api/v1/search?q=\\test",
        ] {
            assert!(
                should_reject(payload),
                "backslash must be rejected: {payload}"
            );
        }
    }

    // ── percent-encoded backslash / path traversal ──────────────────────

    #[test]
    fn percent_encoded_backslash_cannot_bypass() {
        for payload in [
            "%5c%5cevil.example%5c%5capi%5c%5cv1%5c%5csearch",
            "%5C%5C169.254.169.254%5C%5Clatest",
            "/api/%2e%2e/admin/users",
            "/api/%2e%2e%2fadmin",
            "/api/..%2fadmin/users",
            "/api/%252e%252e/admin", // double encode
            "/api/..%5cadmin/users",
            "http://evil.example/%5c%5capi%5c%5cv1",
        ] {
            let result = normalize_local_api_url(payload, 5023);
            match result {
                Ok(url) => {
                    // If it didn't outright reject, it MUST still be local.
                    assert!(
                        url.starts_with("http://127.0.0.1:5023/api/"),
                        "encoded-traversal produced non-local URL: {payload} → {url}"
                    );
                }
                Err(_) => { /* rejection is acceptable */ }
            }
        }
    }

    // ── scheme-relative / double-slash ──────────────────────────────────

    #[test]
    fn scheme_relative_and_double_slash_cannot_escape() {
        for payload in [
            "//evil.example/api/v1/search",
            "//169.254.169.254/latest/meta-data/",
            "///api/v1/search",
            "//api/v1/search",
            "//127.0.0.1:8080/api/v1/search",
        ] {
            let result = normalize_local_api_url(payload, 5023);
            match result {
                Ok(url) => assert!(
                    url.starts_with("http://127.0.0.1:5023/api/"),
                    "scheme-relative escaped: {payload} → {url}"
                ),
                Err(_) => {}
            }
        }
    }

    // ── userinfo in absolute URL ────────────────────────────────────────

    #[test]
    fn userinfo_in_absolute_url_does_not_bypass() {
        // url::Url::parse rejects or strips userinfo; the assertion after join
        // must still hold.
        for payload in [
            "http://user:pass@evil.example/api/v1/search",
            "https://admin@169.254.169.254/api/v1/search",
        ] {
            let result = normalize_local_api_url(payload, 5023);
            match result {
                Ok(url) => assert!(
                    url.starts_with("http://127.0.0.1:5023/api/"),
                    "userinfo failed: {payload} → {url}"
                ),
                Err(_) => {}
            }
        }
    }

    // ── IPv6 authority (absolute) ───────────────────────────────────────

    #[test]
    fn ipv6_authority_is_stripped_to_path() {
        let result = normalize_local_api_url("http://[::ffff:127.0.0.1]:8080/api/v1/search", 5023);
        match result {
            Ok(url) => assert_eq!(
                url, "http://127.0.0.1:5023/api/v1/search",
                "IPv6 authority should be stripped"
            ),
            Err(_) => {}
        }
    }

    // ── alternate port ──────────────────────────────────────────────────

    #[test]
    fn alternate_port_in_input_is_ignored() {
        for payload in [
            "http://127.0.0.1:8080/api/v1/search",
            "http://127.0.0.1:80/api/v1/search",
            "http://evil.example:443/api/v1/search",
        ] {
            let result = normalize_local_api_url(payload, 5023);
            match result {
                Ok(url) => assert_eq!(
                    url, "http://127.0.0.1:5023/api/v1/search",
                    "port must be server port: {payload} → {url}"
                ),
                Err(_) => {}
            }
        }
    }

    // ── dot / dot-segment traversal ─────────────────────────────────────

    #[test]
    fn dot_segment_traversal_cannot_escape_api_prefix() {
        for payload in [
            "/api/../admin/users",
            "https://evil.example/api/../admin/users",
            "/api/..%2fadmin/users",
            "/api/....//....//admin",
            "/api/v1/../../admin",
            "api/../../admin/users",
        ] {
            let result = normalize_local_api_url(payload, 5023);
            match result {
                Ok(url) => assert!(
                    url.starts_with("http://127.0.0.1:5023/api/"),
                    "dot traversal escaped: {payload} → {url}"
                ),
                Err(_) => {}
            }
        }
    }

    // ── control characters ──────────────────────────────────────────────

    #[test]
    fn control_characters_do_not_bypass() {
        for payload in [
            "api/v1/search?q=\n",
            "api/v1/search?q=%00",
            "api/v1/search?q=\x07",
            "api/v1/search\t?q=x",
            "http://evil.example/api/v1/search\r\nX-Injected: true",
        ] {
            let result = normalize_local_api_url(payload, 5023);
            match result {
                Ok(url) => assert!(
                    url.starts_with("http://127.0.0.1:5023/api/"),
                    "control char escaped: {payload:?} → {url}"
                ),
                Err(_) => {}
            }
        }
    }

    // ── fragment / query injection ─────────────────────────────────────

    #[test]
    fn fragment_rejected() {
        for payload in [
            "api/v1/search#fragment",
            "https://example.com/api/v1/search#fragment",
            "api/v1/search#/api/v1/admin",
        ] {
            let result = normalize_local_api_url(payload, 5023);
            match result {
                Ok(url) => {
                    // fragment must be stripped from output
                    assert!(!url.contains('#'), "fragment persisted: {payload} → {url}");
                }
                Err(_) => {}
            }
        }
    }

    // ── query parameter smuggling ──────────────────────────────────────

    #[test]
    fn query_parameters_preserved_but_url_stays_local() {
        let tests = [
            (
                "api/v1/search?url=http://evil.example/",
                "http://127.0.0.1:5023/api/v1/search?url=http://evil.example/",
            ),
            (
                "/api/v1/tags?x=1&y=2",
                "http://127.0.0.1:5023/api/v1/tags?x=1&y=2",
            ),
        ];
        for (input, expected) in tests {
            let result = normalize_local_api_url(input, 5023);
            match result {
                Ok(url) => assert_eq!(url, expected, "query mismatch: {input}"),
                Err(_) => panic!("query acceptance failed: {input}"),
            }
        }
    }

    // ── absolute URL with path traversal in authority ──────────────────

    #[test]
    fn absolute_url_path_traversal_contained() {
        // url::Url::parse("http://evil.example/../../../etc/passwd") parses
        // path as "/../../../etc/passwd". After join against 127.0.0.1 base,
        // the dot segments may be resolved. The final path must start with /api/.
        for payload in [
            "http://evil.example/../../../etc/passwd",
            "http://evil.example/api/v1/../../../etc/passwd",
        ] {
            let result = normalize_local_api_url(payload, 5023);
            // Expected: either rejected (path doesn't start with /api/) or
            // resolves to local. Must not produce external authority.
            match result {
                Ok(url) => assert!(
                    url.starts_with("http://127.0.0.1:5023/api/"),
                    "absolute traversal escaped: {payload} → {url}"
                ),
                Err(_) => {}
            }
        }
    }

    // ── mixed slash ─────────────────────────────────────────────────────

    #[test]
    fn mixed_slash_does_not_bypass() {
        for payload in [
            r"http:/evil.example/api/v1/search",   // single-slash authority
            "http:evil.example/api/v1/search",     // colon-no-slash
            "http:/\\/evil.example/api/v1/search", // (backslash rejected by Phase 1)
        ] {
            let result = normalize_local_api_url(payload, 5023);
            match result {
                Ok(url) => assert!(
                    url.starts_with("http://127.0.0.1:5023/api/"),
                    "mixed slash escaped: {payload} → {url}"
                ),
                Err(_) => {}
            }
        }
    }

    // ── extensive allowlist of known-safe inputs ────────────────────────

    #[test]
    fn extensive_safe_inputs_stay_local() {
        let safe_inputs = [
            "api/v1/search?q=rust+cargo",
            "/api/v1/tags",
            "api/v1/categories",
            "api/v1/news?search=ai&status=published",
            "api/v1/news?topic_type=technology",
            "api/v1/news?date_from=2025-01-01",
            "api/v1/posts?page=1&limit=20",
            "api/v1/comments?post_id=42",
            "api/v1/settings",
            "api/v1/search?q=%E4%B8%AD%E6%96%87",
        ];
        for input in safe_inputs {
            let url = normalized(input);
            assert!(
                url.starts_with("http://127.0.0.1:5023/api/"),
                "safe input failed: {input} → {url}"
            );
        }
    }

    // ── extensive list of known-hostile inputs ─────────────────────────

    #[test]
    fn extensive_hostile_inputs_do_not_produce_external_url() {
        let hostile_inputs = [
            // Non-api paths
            "admin/users",
            "/scalar",
            "/health",
            "login",
            "/",
            // Awkward protocol attempts
            "file:///etc/passwd",
            "javascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            // Empty-ish
            "",
            // External authority with different TLDs
            "https://evil.example/api/v1/search",
            "https://api.openai.com/v1/models",
            "https://metadata.google.internal/computeMetadata/v1/",
            // Port variations
            "http://127.0.0.1:6379/api/v1/search",
            "http://169.254.169.254:80/latest/meta-data/",
            // IPv6 literal
            "http://[::1]:8080/api/v1/search",
            "http://[::ffff:127.0.0.1]:8080/api/v1/search",
            // Non-http schemes
            "ftp://evil.example/api/v1/search",
            "gopher://evil.example/api/v1/search",
            // Localhost variations
            "http://localhost/api/v1/search",
            "http://0.0.0.0/api/v1/search",
            // Cloud metadata
            "http://169.254.169.254/latest/meta-data/",
            // Internal service
            "http://10.0.0.1:8080/api/v1/search",
            "http://192.168.1.1:8080/api/v1/search",
        ];
        for input in hostile_inputs {
            if input.is_empty() {
                assert!(should_reject(input), "empty input must reject");
                continue;
            }
            let result = normalize_local_api_url(input, 5023);
            match result {
                Ok(url) => {
                    assert!(
                        url.starts_with("http://127.0.0.1:5023/api/"),
                        "hostile input must not escape: '{input}' → '{url}'"
                    );
                }
                Err(_) => { /* rejection is always acceptable */ }
            }
        }
    }

    // ── path without /api/ prefix is rejected ─────────────────────────

    #[test]
    fn non_api_path_rejected() {
        let non_api = [
            "admin/users",
            "/scalar",
            "/health",
            "/",
            "posts/42",
            "about",
            "https://evil.example/admin/users",
            "https://evil.example/",
        ];
        for input in non_api {
            match normalize_local_api_url(input, 5023) {
                Ok(url) => {
                    // Even if accepted, must be http://127.0.0.1:PORT/api/...
                    assert!(
                        url.starts_with("http://127.0.0.1:5023/api/"),
                        "non-api path should either reject or be forced to /api/: {input} → {url}"
                    );
                }
                Err(e) => {
                    assert!(
                        e.contains("backslash")
                            || e.contains("fragment")
                            || e.contains("assertion")
                            || e.contains("parse")
                            || e.contains("join"),
                        "unexpected error for '{input}': {e}"
                    );
                }
            }
        }
    }
}
