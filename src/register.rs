use lazy_static::lazy_static;
use prometheus::{Counter, GaugeVec};
use prometheus::{opts, register_counter, register_gauge_vec};

lazy_static! {
    pub static ref UPDATER_COUNTER: Counter =
        register_counter!(opts!("update_requests_total", "Number of UPDATE")).unwrap();
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
            "The HTTP DDNet server online per IP sizes in bytes.",
        ),
        &["address"]
    )
    .unwrap();
}

pub fn clear_metrics() {
    SERVER_ONLINE.reset();
    SERVER_ONLINE_PER_IP.reset();
}
