use crate::models::{AdmissionRequest, AdmissionResponse, AdmissionReview};
use serde_json;
use std::fs;
#[test]
fn test_noop() -> () {}

#[test]
fn deserialize_review_request() {
    // tests that our AdmissionReview request parsing matches the specified documentation
    // see kubernetes docs for dynamic admission control, 'webhook request and response'
    let review_json = fs::read_to_string("./src/tests/admission-review-not-pod.json")
        .expect("Unable to read file!");
    let review: AdmissionReview = serde_json::from_str(review_json.as_str()).unwrap();

    // decompose the payload into sub objects for individual testing
    let request: AdmissionRequest = review.request.unwrap();
    let object = request.object.unwrap();
    let old_object = request.old_object.unwrap();
    let options = request.options.unwrap();

    assert_eq!(request.uid, "705ab4f5-6393-11e8-b7cc-42010a800002");
    assert_eq!(request.kind.version, "v1");
    assert_eq!(request.resource.group, "apps");
    assert_eq!(request.user_info.get("username").unwrap(), "admin");
    assert_eq!(object.get("kind").unwrap(), "Scale");
    assert_eq!(old_object.get("kind").unwrap(), "Scale");
    assert_eq!(options.get("kind").unwrap(), "UpdateOptions");
}

#[test]
fn serialize_response_to_api() {
    let mut response = AdmissionResponse::default();

    response.uid = String::from("foo");

    // build the object before serializing.
    let mut review: AdmissionReview = AdmissionReview::default();
    review.response = Some(response);
    let json = serde_json::ser::to_string(&review).unwrap();
    assert_eq!(json, "{\"response\":{\"uid\":\"foo\",\"allowed\":false}}");
}
