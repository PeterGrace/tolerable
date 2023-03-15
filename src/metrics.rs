use crate::consts::APP_NAME;
use actix_web_prom::{PrometheusMetrics, PrometheusMetricsBuilder};
use prometheus::GaugeVec;
lazy_static! {
        // setup prometheus
    pub static ref STATIC_PROM: PrometheusMetrics = PrometheusMetricsBuilder::new(APP_NAME)
        .endpoint("/metrics")
        .build()
        .unwrap();
    pub static ref APPVER: GaugeVec = register_gauge_vec!(
        format!("{}_app_info",APP_NAME),
        "static app labels that potentially only change at restart",
        &["crate_version", "git_hash"]
    )
    .unwrap();
}

pub fn register_metrics() {
    STATIC_PROM
        .registry
        .register(Box::new(APPVER.clone()))
        .expect("couldn't register appver metric");
}
