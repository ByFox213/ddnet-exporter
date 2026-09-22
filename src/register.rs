use lazy_static::lazy_static;
use prometheus::{opts, register_counter, register_gauge, register_gauge_vec, Counter, Gauge, GaugeVec};

lazy_static! {
    pub static ref UPDATER_COUNTER: Counter =
        register_counter!(opts!("update_requests_total", "Number of update cycles performed")).unwrap();

    pub static ref SCRAPE_SUCCESS: Gauge =
        register_gauge!(opts!("ddnet_exporter_scrape_success", "1 if last scrape from DDNet master succeeded, 0 otherwise")).unwrap();

    pub static ref LAST_SCRAPE_TIMESTAMP: Gauge =
        register_gauge!(opts!("ddnet_exporter_last_scrape_timestamp_seconds", "Timestamp of last successful DDNet scrape")).unwrap();

    pub static ref SCRAPE_ERRORS_TOTAL: Counter =
        register_counter!(opts!("ddnet_exporter_scrape_errors_total", "Total number of failed DDNet scrape attempts")).unwrap();

    pub static ref SERVER_COUNT: Gauge =
        register_gauge!(opts!("ddnet_exporter_servers_total", "Total number of unique DDNet servers parsed")).unwrap();

    pub static ref SERVER_ONLINE: GaugeVec = register_gauge_vec!(
        opts!("server_online", "DDNet server online status"),
        &[
            "address",
            "gametype",
            "map",
            "name",
            "hasPassword",
            "max_clients",
        ]
    )
    .expect("Failed to create SERVER_ONLINE gauge");

    pub static ref SERVER_ONLINE_PER_IP: GaugeVec = register_gauge_vec!(
        opts!(
            "server_online_per_ip",
            "DDNet server online per IP",
        ),
        &["address"]
    )
    .unwrap();
}

pub fn clear_metrics() {
    SERVER_ONLINE.reset();
    SERVER_ONLINE_PER_IP.reset();
}
