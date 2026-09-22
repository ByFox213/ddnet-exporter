use crate::models::Config;
use crate::register::*;
use ddapi_rs::api::DDApi;
use ddapi_rs::api::ddnet::DDnetApi;
use lazy_static::lazy_static;
use log::{debug, error, trace};
use regex::Regex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

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

#[derive(Clone)]
pub struct HealthStatus {
    is_healthy: Arc<AtomicBool>,
    last_update_ts: Arc<AtomicU64>,
}

impl HealthStatus {
    pub fn new() -> Self {
        Self {
            is_healthy: Arc::new(AtomicBool::new(false)),
            last_update_ts: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn mark_healthy(&self) {
        self.is_healthy.store(true, Ordering::Release);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_update_ts.store(now, Ordering::Release);
    }

    pub fn mark_unhealthy(&self) {
        self.is_healthy.store(false, Ordering::Release);
    }

    pub fn check(&self, max_age_secs: u64) -> (bool, u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let last = self.last_update_ts.load(Ordering::Acquire);
        let age = now.saturating_sub(last);
        let healthy = self.is_healthy.load(Ordering::Acquire) && (last > 0) && (age <= max_age_secs);
        (healthy, age)
    }
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

pub async fn ddnet(config: Config, health: HealthStatus) {
    let ddapi = DDApi::new();
    loop {
        trace!("Updating DDNet metrics from master server");
        let start = std::time::Instant::now();

        let master_result = tokio::time::timeout(
            Duration::from_secs(config.timeout),
            ddapi.master(),
        )
        .await;

        let master = match master_result {
            Ok(Ok(result)) => result,
            Ok(Err(err)) => {
                error!("DDNet master API request failed: {err:?}");
                SCRAPE_ERRORS_TOTAL.inc();
                SCRAPE_SUCCESS.set(0.0);
                health.mark_unhealthy();
                sleep(Duration::from_secs(config.error_delay)).await;
                continue;
            }
            Err(_) => {
                error!("DDNet master API request timed out after {}s", config.timeout);
                SCRAPE_ERRORS_TOTAL.inc();
                SCRAPE_SUCCESS.set(0.0);
                health.mark_unhealthy();
                sleep(Duration::from_secs(config.error_delay)).await;
                continue;
            }
        };

        let server_count = master.servers.len();
        clear_metrics();

        let mut ip_online: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

        for server in master.servers {
            let online = server.count_client() as f64;
            let passworded_str = server.info.passworded.to_string();
            let max_clients_str = server.info.max_clients.to_string();

            let mut server_ips = std::collections::HashSet::new();

            for address in server.addresses {
                if let Some((ip, port_str)) = get_address(&address) {
                    let addr_str = format!("{ip}:{port_str}");
                    let labels = [
                        addr_str.as_str(),
                        server.info.gametype.as_str(),
                        server.info.map.name.as_str(),
                        server.info.name.as_str(),
                        passworded_str.as_str(),
                        max_clients_str.as_str(),
                    ];

                    SERVER_ONLINE.with_label_values(&labels).set(online);
                    server_ips.insert(ip);
                }
            }

            for ip in server_ips {
                *ip_online.entry(ip).or_insert(0.0) += online;
            }
        }

        for (ip, total_online) in ip_online {
            SERVER_ONLINE_PER_IP
                .with_label_values(&[ip.as_str()])
                .set(total_online);
        }

        let now_ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        UPDATER_COUNTER.inc();
        SCRAPE_SUCCESS.set(1.0);
        LAST_SCRAPE_TIMESTAMP.set(now_ts);
        SERVER_COUNT.set(server_count as f64);
        health.mark_healthy();

        let elapsed = start.elapsed();
        debug!(
            "Updated metrics for {} servers in {:.2}s, sleeping for {}s",
            server_count,
            elapsed.as_secs_f64(),
            config.delay
        );

        sleep(Duration::from_secs(config.delay)).await;
    }
}
