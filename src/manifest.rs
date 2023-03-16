use std::fmt;
use std::path::Path;
use anyhow::{bail};
use awc::{Client, ClientRequest, SendClientRequest};
use awc::error::JsonPayloadError;
use config::Config;
use regex::Regex;
use docker_image_reference::Reference;
use crate::read_setting_string;
use cached::proc_macro::cached;
use serde_json::Value;
use crate::consts::*;


lazy_static! {
            static ref DOCKER_RE: Regex = Regex::new(DOCKER_IMAGE_REGEXP).unwrap();
            static ref TOKEN_AUTH_RE: Regex = Regex::new(TOKEN_AUTH_REGEXP).unwrap();
}

#[cached]
pub async fn validate_manifest(image: String) -> Option<Vec<String>> {

    let mut manifest_ref: Reference = match Reference::from_str(&image){
        Ok(m) => m,
        Err(e) => {
            warn!{"unable to match manifest from image string: {}", e};
            return None;
        }
    };
    // first attempt to hit the endpoint
    let mut registryport = "".to_string();
    // we will start with the docker hub registry as its the assumed default.
    let mut registry = "docker.io";
    let mut tag = "";
    if manifest_ref.registry_name().is_some() {
        registry = manifest_ref.registry_name().unwrap();
    }
    let cred = get_credentials_for_registry(registry.to_string()).await;
    if registry == "docker.io" {
        // see consts.rs for commentary
        registry = ACTUAL_DOCKER_REGISTRY;
    }
    if manifest_ref.registry_port().is_some() {
        registryport = format!("{}{}", registry, manifest_ref.registry_port().unwrap())
    } else {
        registryport = format!("{}", registry);
    }
    if manifest_ref.tag().is_some() {
        tag = manifest_ref.tag().unwrap();
    } else {
        tag = "latest";
    }

    // Docker legacy image names can be specified without a repository name ('nginx' vs 'library/nginx') so
    // we will fix these here, as the backend for docker.io at least requires that in the api call.
    let mut image_name: String;
    if !manifest_ref.name().contains('/') {
        image_name = format!("library/{}", manifest_ref.name());
    } else {
        image_name = manifest_ref.name().to_owned();
    }

    let url = format!("https://{registryport}/v2/{}/manifests/{tag}", image_name);
    let token = match get_jwt(url.clone(), cred).await {
        Some(t) => t,
        None => {
            warn!("No jwt returned, assuming no auth necessary..");
            "".to_string()
        }
    };
    let client = Client::new();
    let image_manifest_types = vec![
        "application/vnd.oci.image.index.v1+json",
        "application/vnd.docker.distribution.manifest.list.v2+json"
    ];
    let mut manifest_req :ClientRequest;
            manifest_req = client
                .get(&url)
                .bearer_auth(token)
                .insert_header(("Accept", image_manifest_types.join(",")));
    debug!("MANIFEST REQ: {:#?}", manifest_req);
    let mut manifest_rs = match manifest_req.send().await {
        Ok(r) => r,
        Err(e) => {
            warn!("Unable to contact for manifest request: {e}");
            return None;
        }
    };
        

    // due to the fact that docker.io returns a mime type that actix-web doesn't like, we'll save the body to
    // a variable and attempt to convert it to json there.
    let rs_body = match manifest_rs.body().await {
        Ok(b) => b,
        Err(e) => {
            warn!("Unable to decode payload of manifest response: {e}");
            return None;
        }
    };

    let rs_json: Value = match serde_json::from_slice(rs_body.as_ref()) {
        Ok(m) => m,
        Err(e) => {
            warn!("Error decoding result from manifest request: {e}");
            debug!("{:#?}", manifest_rs);
            debug!("{:#?}", manifest_rs.body().await);
            return None;
        }
    };

    // depending on schemaVersion, we need to work a little differently.
    let schemaVersion: u64 = match rs_json.get("schemaVersion") {
        Some(v) => v.as_u64().unwrap(),
        None => {
            warn!("Response had no schemaVersion, so we can't deduce where a platform value would live.");
            return None;
        }
    };
    return match schemaVersion {
        1 => get_version_1_arches(&rs_json),
        2 => get_version_2_arches(&rs_json),
        _ => {
            warn!("We received a value we did not expect for schemaVersion: {}", schemaVersion);
            None
        }
    }
}

pub fn get_version_1_arches(json: &Value) -> Option<Vec<String>> {
    let mut arches:Vec<String> = vec![];
    match json.get("architecture") {
        Some(s) => arches.push(s.as_str().unwrap().to_string()),
        None => {
            warn!("Schema version 1, but no architecture specified");
            return None
        }
    };
    Some(arches)
}

pub fn get_version_2_arches(json: &Value) -> Option<Vec<String>> {
    let manifests = match json.get("manifests") {
        Some(m) => m.as_array().unwrap(),
        None => {
            warn!("no manifests found: {:#?}", json);
            return None;
        }
    };
    let mut arches = Vec::new();
    for mani in manifests {
        let platform = match mani.get("platform") {
            Some(p) => p,
            None => {
                warn!("no platforms detected: {:#?}", mani);
                return None;
            }
        };
        let arch = match platform.get("architecture") {
            Some(a) => a.as_str().unwrap(),
            None => {
                warn!("platform exists but no architecture found: {:#?}", platform);
                return None;
            }
        };
        if !arches.contains(&arch.to_string()) {
            arches.push(arch.to_string());
        }
    }
    debug!("v2 architectures discovered {:#?}", arches);
    Some(arches)
}


pub async fn get_jwt(url: String, credentials: Option<RegistryCredential>) -> Option<String> {
    let client = Client::new();

    let mut rs = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            warn!("Error on initial request for token data");
            return None;
        }
    };

    let auth = match rs.headers().get("www-authenticate") {
        Some(val) => val.to_str().unwrap(),
        None => {
            info!("we tried to query for a jwt but did not get www-authenticate header.");
            return None;
        }
    };
    let cap = TOKEN_AUTH_RE.captures(auth).unwrap();
    let authurl = format!("{}?service={}&scope={}",
                          cap.name("url").unwrap().as_str(),
                          cap.name("service").unwrap().as_str(),
                          cap.name("scope").unwrap().as_str());
    let mut auth_req = client.get(authurl.clone())
        .basic_auth(credentials.clone().unwrap().user, credentials.clone().unwrap().secret);
    debug!("AUTH REQ: {:#?}", auth_req);
    let mut auth_rs = match auth_req
        .send()
        .await {
        Ok(a) => a,
        Err(e) => {
            warn!("Couldn't get token from {}: {}", authurl, e);
            return None;
        }
    };

    let body = match auth_rs.json::<serde_json::Value>().await {
        Ok(a) => a,
        Err(e) => {
            warn!("Didn't deserialize body to json in response: {}", e);
            return None
        }
    };
    match body.get("token") {
        Some(a) => Some(a.as_str().unwrap().parse().unwrap()),
        None => {
            warn!("Couldn't find token in response.");
            None
        }
    }
}

pub async fn get_credentials_for_registry(registry: String) -> Option<RegistryCredential> {
    let cred_path = match read_setting_string("registry_credential_path") {
        Ok(path) => path,
        Err(e) => {
            warn!{"cred path not found in config (registry_credential_path)"}
            return None;
        }
    };
    let cred_file = format!("{}/{}.toml",cred_path,registry);
    if !Path::new(&cred_file).exists() {
        warn!("credential file does not exist: {}", cred_file);
        return None;
    };
    let cred: config::Config = match Config::builder()
        .add_source(config::File::with_name(&cred_file))
        .build(){
        Ok(config) => config,
        Err(e) => {
            warn!("path '{}' exists but config couldn't initialize", cred_file);
            return None;
        }
    };

    let user: String = match cred.get::<String>("user") {
        Ok(user) => user,
        Err(e) => {
            warn!("In '{}', config key 'user' does not exist: {}", cred_file, e);
            return None;
        }
    };
    let secret: String = match cred.get::<String>("secret") {
        Ok(secret) => secret,
        Err(e) => {
            warn!("In '{}', config key 'secret' does not exist: {}", cred_file, e);
            return None;
        }
    };
    Some(RegistryCredential {user, secret})
}

#[derive(Clone, Debug)]
pub struct RegistryCredential {
    user: String,
    secret: String
}
