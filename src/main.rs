mod consts;
mod metrics;
mod models;
mod mutation;
mod tests;
mod manifest;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};

use crate::metrics::{register_metrics, APPVER, STATIC_PROM};
use crate::mutation::mutate_handler;
use config::{Config};
use rustls::ServerConfig;
use rustls_pemfile;
use std::env;
use std::env::set_var;
use std::fs;
use std::io::BufReader;
use std::sync::RwLock;

lazy_static! {
    static ref SETTINGS: RwLock<Config> = RwLock::new({
        let cfg_file = match std::env::var("CONFIG_FILE_PATH") {
            Ok(s) => s,
            Err(_e) => { "./tolerable.toml".to_string()}
        };
        let settings = match Config::builder()
            .add_source(config::File::with_name(&cfg_file))
            .add_source(
                config::Environment::with_prefix("TOLERABLE")
                .try_parsing(true)
                .list_separator(",")
                .with_list_parse_key("supported_architectures")
            )
            .build()
            {
                Ok(s) => s,
                Err(e) => {
                    panic!("{}", e);
                }
            };
        settings
    });
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // initialize logging
     match read_setting_string("log_level") {
        Ok(log_level) => set_var("RUST_LOG", log_level),
        Err(_e) => {}
    }

    pretty_env_logger::init();

    // get prometheus metrics ready to rock
    register_metrics();

    // measure our first metric, the appver
    let appdata = APPVER.with_label_values(&[env!("CARGO_PKG_VERSION"), env!("GIT_HASH")]);
    appdata.set(1 as f64);
    debug!("tolerable cargo:{}, githash:{}", env!("CARGO_PKG_VERSION"),env!("GIT_HASH"));

    let ssl_key_path = match read_setting_string("ssl_key_path") {
        Ok(s) => s,
        Err(e) => {panic!("{}",e);}
    };
    let ssl_cert_path = match read_setting_string("ssl_cert_path") {
        Ok(s) => s,
        Err(e) => {panic!("{}",e);}
    };
    let versions = rustls::ALL_VERSIONS.to_vec();
    let suites = rustls::ALL_CIPHER_SUITES.to_vec();
    let certs = load_certs(&ssl_cert_path);
    let privkey = load_private_key(&ssl_key_path);
    let rustls_config = ServerConfig::builder()
        .with_cipher_suites(&suites)
        .with_safe_default_kx_groups()
        .with_protocol_versions(&versions)
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(certs, privkey)
        .unwrap();
    // fire up server and lets go!
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(STATIC_PROM.clone())
            .service(mutate_handler)
            .route("/health", web::get().to(HttpResponse::Ok))
    })
    .bind_rustls(("0.0.0.0", 8443), rustls_config)?
    .run()
    .await?;
    Ok(())
}

fn load_private_key(filename: &str) -> rustls::PrivateKey {
    let keyfile = fs::File::open(filename).expect("cannot open private key file");
    let mut reader = BufReader::new(keyfile);

    loop {
        match rustls_pemfile::read_one(&mut reader).expect("cannot parse private key .pem file") {
            Some(rustls_pemfile::Item::RSAKey(key)) => return rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::PKCS8Key(key)) => return rustls::PrivateKey(key),
            Some(rustls_pemfile::Item::ECKey(key)) => return rustls::PrivateKey(key),
            None => break,
            _ => {}
        }
    }

    panic!(
        "no keys found in {:?} (encrypted keys not supported)",
        filename
    );
}
fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    let certfile = fs::File::open(filename).expect("cannot open certificate file");
    let mut reader = BufReader::new(certfile);
    rustls_pemfile::certs(&mut reader)
        .unwrap()
        .iter()
        .map(|v| rustls::Certificate(v.clone()))
        .collect()
}

fn read_setting_string(key: &str) -> anyhow::Result<String> {
    match SETTINGS.read().unwrap().get::<String>(&key) {
        Ok(s) => Ok(s),
        Err(_e)  => {
            Err(anyhow::anyhow!("unable to read config setting {}",key))
            }
        }
}