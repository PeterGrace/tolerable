pub const APP_NAME: &str = "tolerable";
pub const DOCKER_IMAGE_REGEXP: &str = r#"^(?P<repository>[\w.\-_]+((?::\d+|)(?=/[a-z0-9._-]+/[a-z0-9._-]+))|)\(?:/|)(?P<image>[a-z0-9.\-_]+(?:/[a-z0-9.\-_]+|))(:(?P<tag>[\w.\-_]{1,127})|)$"#;
pub const TOKEN_AUTH_REGEXP: &str = r#"^Bearer realm="(?P<url>.+)",service="(?P<service>.+)",scope="(?P<scope>.+)"$"#;

// amusingly enough, it seems that docker.io is the only registry allowed to have a different actual dns name,
// and everyone just seems to go along with this.
pub const ACTUAL_DOCKER_REGISTRY: &str = r#"registry-1.docker.io"#;
