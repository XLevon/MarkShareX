mod common;

use axum::http::{Method, StatusCode};
use common::TestApp;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn library_exposes_the_production_router_factory() {
    let _factory = marksharex::build_router;
}

fn allow_methods(response: &axum_test::TestResponse) -> BTreeSet<String> {
    response
        .header("allow")
        .to_str()
        .expect("405 response must include a valid Allow header")
        .split(',')
        .map(|method| method.trim().to_string())
        .collect()
}

fn expected_allow(methods: &BTreeSet<String>) -> BTreeSet<String> {
    let mut expected = methods.clone();
    if expected.contains("GET") {
        expected.insert("HEAD".to_string());
    }
    expected
}

fn concrete_path(path: &str) -> String {
    path.split('/')
        .map(|segment| {
            if !segment.starts_with(':') {
                segment
            } else if segment == ":id" || segment.ends_with("_id") {
                "2147483647"
            } else {
                "missing-value"
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

async fn catalog_methods(app: &TestApp) -> BTreeMap<String, BTreeSet<String>> {
    let response = app.server.get("/api/v1/").await;
    response.assert_status_ok();
    let body = response.json::<Value>();
    let mut catalog = BTreeMap::<String, BTreeSet<String>>::new();
    for endpoint in body["data"]
        .as_array()
        .expect("discovery data must be an array")
    {
        catalog
            .entry(endpoint["path"].as_str().unwrap().to_string())
            .or_default()
            .insert(endpoint["method"].as_str().unwrap().to_string());
    }
    catalog
}

#[tokio::test]
async fn every_shared_path_preserves_merged_methods_and_precise_allow_headers() -> anyhow::Result<()>
{
    let app = TestApp::new().await?;
    let catalog = catalog_methods(&app).await;
    let shared = catalog
        .iter()
        .filter(|(_, methods)| methods.len() > 1)
        .collect::<Vec<_>>();
    assert_eq!(
        shared.len(),
        39,
        "review shared-path matrix changes explicitly"
    );

    for (path, methods) in shared {
        let concrete = concrete_path(path);
        let response = app.server.method(Method::TRACE, &concrete).await;
        assert_eq!(
            response.status_code(),
            StatusCode::METHOD_NOT_ALLOWED,
            "TRACE must be rejected by the production router for {path}"
        );
        assert_eq!(
            allow_methods(&response),
            expected_allow(methods),
            "merged method set drifted for {path}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn every_static_parameter_sibling_prefers_the_static_route() -> anyhow::Result<()> {
    let app = TestApp::new().await?;
    let catalog = catalog_methods(&app).await;
    let mut pairs = Vec::new();
    for (static_path, static_methods) in &catalog {
        let static_segments = static_path.split('/').collect::<Vec<_>>();
        if static_segments
            .iter()
            .any(|segment| segment.starts_with(':'))
        {
            continue;
        }
        for (parameter_path, parameter_methods) in &catalog {
            let parameter_segments = parameter_path.split('/').collect::<Vec<_>>();
            if static_segments.len() == parameter_segments.len()
                && parameter_segments
                    .iter()
                    .any(|segment| segment.starts_with(':'))
                && static_segments
                    .iter()
                    .zip(&parameter_segments)
                    .all(|(actual, pattern)| actual == pattern || pattern.starts_with(':'))
            {
                pairs.push((
                    static_path,
                    static_methods,
                    parameter_path,
                    parameter_methods,
                ));
            }
        }
    }
    assert_eq!(
        pairs.len(),
        14,
        "review static/parameter siblings explicitly"
    );

    for (static_path, static_methods, parameter_path, parameter_methods) in pairs {
        let response = app.server.method(Method::TRACE, static_path).await;
        assert_eq!(response.status_code(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(
            allow_methods(&response),
            expected_allow(static_methods),
            "static path {static_path} was not routed with its own methods instead of {parameter_path}"
        );

        if static_methods == parameter_methods && static_methods.contains("GET") {
            let response = app.server.get(static_path).await;
            assert_ne!(
                response.status_code(),
                StatusCode::BAD_REQUEST,
                "static path {static_path} fell through to parameter extractor {parameter_path}"
            );
        }
    }
    Ok(())
}
