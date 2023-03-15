use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use strum_macros::EnumString;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct AdmissionReview {
    /// request is the incoming AdmissionRequest object
    #[serde(rename = "request", skip_serializing_if = "Option::is_none")]
    pub request: Option<AdmissionRequest>,
    /// response is the outgoing, mutated object.
    #[serde(rename = "response", skip_serializing_if = "Option::is_none")]
    pub response: Option<AdmissionResponse>,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct AdmissionRequest {
    /// uid is an identifier for the individual request/response
    pub uid: String,
    /// kind is the kind of the object
    pub kind: GroupVersionKind,
    /// resource is the fully-qualified resource being requested
    pub resource: GroupVersionResource,
    #[serde(rename = "subResource", skip_serializing_if = "Option::is_none")]
    pub sub_resource: Option<String>,
    /// request_kind is the fully-qualified type of the original API request
    #[serde(rename = "requestKind", skip_serializing_if = "Option::is_none")]
    pub request_kind: Option<GroupVersionKind>,
    /// request_sub_resource is the name of the subresource of the original API request
    #[serde(rename = "requestSubResource", skip_serializing_if = "Option::is_none")]
    pub request_sub_resource: Option<String>,
    /// name is the name of the object as presented in the request.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// namespace is the namespace associated with the request, if any.
    #[serde(rename = "namespace", skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// operation is the operation being performed, which may be different than requested.
    /// For example, a PATCH might wind up resulting in a CREATE or UPDATE operation.
    pub operation: Operation,
    /// user_info is information about the requesting user
    #[serde(rename = "userInfo")]
    pub user_info: Map<String, Value>,
    /// object is the object from the incoming request.
    #[serde(rename = "object", skip_serializing_if = "Option::is_none")]
    pub object: Option<Map<String, Value>>,
    /// old_object is the existing object, only populated during DELETE or UPDATE requests.
    #[serde(rename = "oldObject", skip_serializing_if = "Option::is_none")]
    pub old_object: Option<Map<String, Value>>,
    /// dry_run indicates whether the modifications will be persisted for the object.  default false.
    #[serde(rename = "dryRun", skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    /// options is the operation options structure of the operation being performed.
    #[serde(rename = "options", skip_serializing_if = "Option::is_none")]
    pub options: Option<Map<String, Value>>,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct AdmissionResponse {
    /// uid is an identifier for the individual response
    pub uid: String,
    /// allowed indicates whether or not the admission request was permitted
    pub allowed: bool,
    /// status contains extra details into why an admission request was denied.
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<StatusResult>>,
    /// patch is the jsonpatch (RFC6902) for the object
    #[serde(rename = "patch", skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,
    /// patch_type is the type of patch.  Currently, only JSONPatch can be used.
    #[serde(rename = "patchType", skip_serializing_if = "Option::is_none")]
    pub patch_type: Option<PatchType>,
    /// audit_annotations is a structured key-value map set by the remote admission controller
    #[serde(rename = "auditAnnotations", skip_serializing_if = "Option::is_none")]
    pub audit_annotations: Option<Map<String, Value>>,
    /// warnings is a list of warning messages to return to the api client
    #[serde(rename = "warnings", skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct GroupVersionKind {
    /// group is the api group of the kind
    pub group: String,
    /// version is the version value assigned to this kind
    pub version: String,
    /// kind is the kubernetes object-kind
    pub kind: String,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct GroupVersionResource {
    /// group is the api group of the resource
    pub group: String,
    /// version is the version value assigned to this resource
    pub version: String,
    /// resource is the name of the resource
    pub resource: String,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, EnumString)]
pub enum Operation {
    CREATE,
    UPDATE,
    DELETE,
    CONNECT,
}

impl Default for Operation {
    fn default() -> Self {
        Operation::UPDATE
    }
}

#[derive(Debug, PartialEq, Clone, EnumString, Serialize, Deserialize)]
pub enum PatchType {
    #[serde(rename = "JSONPatch")]
    JSONPatch,
}
impl Default for PatchType {
    fn default() -> Self {
        PatchType::JSONPatch
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct StatusResult {
    /// status is a string that describes the status
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// message is a human-readable description of the operation failed
    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// reason is a machine-readable description of the failure scenario
    #[serde(rename = "reason", skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// details is extended data associated with the reason
    #[serde(rename = "details", skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// code is the suggested http return code
    #[serde(rename = "code", skip_serializing_if = "Option::is_none")]
    pub code: Option<u16>,
}
