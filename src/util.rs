use crate::register::*;
use ddapi_rs::api::DDApi;
use ddapi_rs::api::ddnet::DDnetApi;
use lazy_static::lazy_static;
use log::{error, trace};
use regex::Regex;
use std::time::Duration;
use tokio::time::sleep;
use crate::models::Config;

lazy_static! {
    static ref ADDRESS_REGEX: Regex = Regex::new(
        r"(?x)
        (?:tw-0\.[67]\+udp://|udp://)  # tw-0.7, tw-0.6 and udp
        (?:\[([0-9a-fA-F:]+)\]|      # IPv6
         ([0-9.]+))                  # IPv4
        :(\d+)                       # Port
        "
    )
    .unwrap();
}

/// - "udp://IP:PORT"
/// - "udp://[IPv6]:PORT"
/// - tw-0.6+udp://IP:PORT
/// - tw-0.6+udp://[IPv6]:PORT
/// - "tw-0.7+udp://IP:PORT"
/// - "tw-0.7+udp://[IPv6]:PORT"
pub fn get_address(address: &str) -> Option<(String, String)> {
    let captures = ADDRESS_REGEX.captures(address)?;

    let ip = captures
        .get(1) // IPv6
        .or_else(|| captures.get(2))? // IPv4
        .as_str()
        .to_string();

    let port = captures.get(3)?.as_str().to_string();

    if port.parse::<u16>().is_err() {
        return None;
    }

    Some((ip, port))
}

pub async fn ddnet(config: Config) {
    let ddapi = DDApi::new();
    loop {
        trace!("update metrics");
        let master = match ddapi.master().await {
            Ok(result) => result,
            Err(err) => {
                error!("ddnet err: {err:?}");
                continue;
            }
        };
        clear_metrics();
        for server in master.servers {
            let online = server.count_client() as f64;
            for address in server.addresses {
                if let Some((ip, port_str)) = get_address(&address) {
                    let address = format!("{}:{}", ip, port_str);
                    let labels = &[
                        &address,
                        &server.info.gametype,
                        &server.info.map.name,
                        &server.info.name,
                        &server.info.passworded.to_string(),
                        &server.info.max_clients.to_string(),
                    ];

                    SERVER_ONLINE.with_label_values(labels).set(online);
                    SERVER_ONLINE_PER_IP
                        .with_label_values(&[&address])
                        .set(online);
                }
            }
        }

        UPDATER_COUNTER.inc();
        trace!("updated metrics, sleep: {}", config.delay);
        sleep(Duration::from_secs(config.delay)).await;
    }
}
