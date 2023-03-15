use crate::models::{AdmissionResponse, AdmissionReview, GroupVersionKind, StatusResult};
use crate::SETTINGS;
use actix_web::{post, web};
use base64::{engine::general_purpose, Engine as _};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::manifest::validate_manifest;
use array_tool::vec::Union;


fn generate_error_response(uid: String, msg: &str) -> AdmissionReview {
    let message = StatusResult {
        code: Some(400),
        reason: Some(msg.to_string()),
        message: Some(msg.to_string()),
        details: None,
        status: Some(msg.to_string()),
    };
    let messages = vec![message];
    let mut response = AdmissionResponse::default();
    response.uid = uid;
    response.allowed = false;
    response.status = Some(messages);
    let mut review = AdmissionReview::default();
    review.response = Some(response);
    review
}


#[post("/mutate")]
pub async fn mutate_handler(
    incoming_review: web::Json<AdmissionReview>,
) -> web::Json<AdmissionReview> {
    let req = incoming_review.request.clone().unwrap();
    let object = req.object.clone().unwrap();
    let mut patches: Vec<Value> = Vec::new();
    // is this an actual kubernetes object or junk?
    let kind: GroupVersionKind = req.kind;

    // build response object
    let mut response = AdmissionResponse::default();
    response.uid = req.uid.clone();
    response.allowed = true;

    // figure out if we should mutate
    if kind.kind == "Pod" {
        // A pod either has a name or a generateName.
        match object.get("metadata").unwrap().get("name") {
            Some(name) => { info!("Considering pod {}",name) }
            None => { info!("Considering generateName {}", object.get("metadata").unwrap().get("generateName").unwrap()) }
        };

        let spec = match object.get("spec") {
            Some(s) => s,
            None => {
                warn!("We think object is a pod, but it has no spec?");
                return web::Json(generate_error_response(req.uid.clone(), "Pod has no spec?"));
            }
        };
        let containers = match spec.get("containers") {
            Some(c) => c.as_array().unwrap(),
            None => {
                warn!("We think the object is a pod, but it has no containers?");
                return web::Json(generate_error_response(
                    req.uid.clone(),
                    "PodSpec has no containers?",
                ));
            }
        };

        let mut match_found: bool = true;

        let supported_architectures: Vec<String> = match SETTINGS
            .read()
            .unwrap()
            .get::<Vec<String>>("supported_architectures") {
            Ok(vs) => vs,
            Err(e) => {
                warn!("{e}: no supported architectures found (or error in config file?) -- letting pod through without patch");
                let mut review = AdmissionReview::default();
                review.response = Some(response);
                return web::Json(review);
            }
        };
        let mut toleration_config: HashMap<String, String> = match SETTINGS
            .read()
            .unwrap()
            .get::<HashMap<String, String>>("tolerations")
        {
            Ok(c) => c,
            Err(_) => {
                warn!("No toleration is specified in configfile, letting pod through without patch.");
                let mut review = AdmissionReview::default();
                review.response = Some(response);
                return web::Json(review);
            }
        };


        for architecture in supported_architectures {
            for container in containers {
                let obj: HashMap<String, Value> = serde_json::from_value(container.clone()).unwrap();
                let image = obj.get("image").unwrap().as_str().unwrap();
                let arches = match validate_manifest(image.to_string()).await {
                    Some(a) => a,
                    None => {
                        warn!("Can't find architecture for image {}", image);
                        match_found = false;
                        vec![]
                    }
                };

                if arches.contains(&architecture) {
                    info!("HIT: {image} has an image of type {architecture}");
                } else {
                    info!("MISS: image {image} doesn't contain an {architecture} image");
                    match_found = false;
                };


                if match_found {
                    let mut arch_toleration = toleration_config.clone();
                    arch_toleration.entry("value".to_string())
                        .and_modify(|val| *val = architecture.clone()).or_insert(architecture.clone());
                    // all of the containers of this pod match supported image list, lets add the toleration
                    debug!("patching pod with architecture {architecture}.");
                    patches.push(json!({
                        "op": "add",
                        "path": "/spec/tolerations/-",
                        "value": arch_toleration
                    }));
                };
            }
        }

        let tolerations = spec.get("tolerations");
        if tolerations.is_none() {
            // no previous tolerations, so we need to patch it in
            if patches.len() > 0 {
                debug!("pod did not have previous tolerations");
                patches.insert(0, (json!({
                    "op": "add",
                    "path": "/spec",
                    "value": {
                        "tolerations": []
                    }
                    }
                )));
            }
        };
    }
    // build review wrapper
    if patches.len() > 0 {
        response.patch = Some(general_purpose::STANDARD.encode(json!(patches).to_string()));
    }
    let mut review = AdmissionReview::default();
    review.response = Some(response);

    // send it
    web::Json(review)
}
