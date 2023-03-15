#[cfg(test)]
#[ctor::ctor]
fn init() {
    pretty_env_logger::init();
}

use crate::models::AdmissionReview;
use crate::mutation::mutate_handler;
use actix_web::{test, App};
use serde_json;
use std::fs;
use crate::manifest::validate_manifest;

#[test]
async fn test_manifest_validator_v2() {
    let expected = "arm64".to_string();
    let results = validate_manifest("nginx:latest".to_string()).await.unwrap();
    assert!(results.contains(&expected));
}
#[test]
// redundant because I fixed docker.io schema v1 lookup, but, didn't want to remove the code just
// in case
async fn test_manifest_validator_v1() {
    let expected = "amd64".to_string();
    let results = match validate_manifest("nginx:latest".to_string()).await {
        Some(s) => s,
        None => vec![]
    };
    assert!(results.contains(&expected));
}

#[actix_web::test]
async fn test_mutate_handler_non_pod() {
    let app = test::init_service(App::new().service(mutate_handler)).await;
    let review_json = fs::read_to_string("./src/tests/admission-review-not-pod.json")
        .expect("Unable to read file!");
    let review: AdmissionReview = serde_json::from_str(review_json.as_str()).unwrap();
    let req = test::TestRequest::post()
        .uri("/mutate")
        .set_json(review)
        .to_request();
    let resp: AdmissionReview = test::call_and_read_body_json(&app, req).await;
    let rs_resp = resp.response.unwrap();
    assert_eq!(&rs_resp.allowed, &true);
    assert!(&rs_resp.patch.is_none());
}

#[actix_web::test]
async fn test_mutate_handler_as_pod_not_arm() {
    std::env::set_var("TOLERABLE_SUPPORTED_ARCHITECTURES","arm64");
    let app = test::init_service(App::new().service(mutate_handler)).await;
    let review_json =
        fs::read_to_string("./src/tests/admission-review-pod.json").expect("Unable to read file!");
    let review: AdmissionReview = serde_json::from_str(review_json.as_str()).unwrap();
    let req = test::TestRequest::post()
        .uri("/mutate")
        .set_json(review)
        .to_request();
    let resp: AdmissionReview = test::call_and_read_body_json(&app, req).await;
    let rs_resp = resp.response.unwrap();
    assert_eq!(&rs_resp.allowed, &true);
    assert!(&rs_resp.patch.is_none());
}
#[actix_web::test]
async fn test_mutate_handler_as_pod_matches_arm() {
    std::env::set_var("TOLERABLE_SUPPORTED_ARCHITECTURES","arm64");
    let app = test::init_service(App::new().service(mutate_handler)).await;
    let review_json = fs::read_to_string("./src/tests/admission-review-pod-match.json")
        .expect("Unable to read file!");
    let review: AdmissionReview = serde_json::from_str(review_json.as_str()).unwrap();
    let req = test::TestRequest::post()
        .uri("/mutate")
        .set_json(review)
        .to_request();
    let resp: AdmissionReview = test::call_and_read_body_json(&app, req).await;
    let rs_resp = resp.response.unwrap();
    assert_eq!(&rs_resp.allowed, &true);
    assert!(&rs_resp.patch.is_some());
}
