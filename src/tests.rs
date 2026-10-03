#[cfg(test)]
mod tests {
    use crate::util::{get_address, normalize_server_info};

    #[test]
    fn test_normalize_flat_info_unchanged() {
        let mut value = serde_json::json!({
            "servers": [
                { "addresses": ["tw-0.6+udp://1.2.3.4:8303"], "info": { "max_clients": 64, "name": "A" } }
            ]
        });
        normalize_server_info(&mut value);
        assert_eq!(value["servers"][0]["info"]["max_clients"], 64);
        assert_eq!(value["servers"][0]["info"]["name"], "A");
    }

    #[test]
    fn test_normalize_map_keyed_info() {
        let mut value = serde_json::json!({
            "servers": [
                {
                    "addresses": ["tw-0.6+udp://1.2.3.4:8306"],
                    "info": {
                        "3d0f5a1c2b7e9d48": { "max_clients": 128, "name": "Kestrel CTF", "map": { "name": "sky_islands" } }
                    }
                }
            ]
        });
        normalize_server_info(&mut value);
        assert_eq!(value["servers"][0]["info"]["max_clients"], 128);
        assert_eq!(value["servers"][0]["info"]["name"], "Kestrel CTF");
        assert!(
            value["servers"][0]["info"]
                .get("3d0f5a1c2b7e9d48")
                .is_none()
        );
    }

    #[test]
    fn test_normalize_mixed_servers() {
        let mut value = serde_json::json!({
            "servers": [
                { "info": { "max_clients": 64, "name": "Flat" } },
                { "info": { "aaaa": { "max_clients": 32, "name": "Nested" } } }
            ]
        });
        normalize_server_info(&mut value);
        let servers = value["servers"].as_array().unwrap();
        assert_eq!(servers[0]["info"]["name"], "Flat");
        assert_eq!(servers[1]["info"]["name"], "Nested");
    }

    #[test]
    fn test_valid_ipv4_address() {
        let result = get_address("udp://192.168.1.1:8080");
        assert_eq!(
            result,
            Some(("192.168.1.1".to_string(), "8080".to_string()))
        );
    }

    #[test]
    fn test_valid_ipv6_address() {
        let result = get_address("udp://[2001:db8::1]:8080");
        assert_eq!(
            result,
            Some(("2001:db8::1".to_string(), "8080".to_string()))
        );
    }

    #[test]
    fn test_valid_tw_06_ipv4() {
        let result = get_address("tw-0.6+udp://192.168.1.1:8080");
        assert_eq!(
            result,
            Some(("192.168.1.1".to_string(), "8080".to_string()))
        );
    }

    #[test]
    fn test_valid_tw_06_ipv6() {
        let result = get_address("tw-0.6+udp://[2001:db8::1]:8080");
        assert_eq!(
            result,
            Some(("2001:db8::1".to_string(), "8080".to_string()))
        );
    }

    #[test]
    fn test_valid_tw_07_ipv4() {
        let result = get_address("tw-0.7+udp://192.168.1.1:8080");
        assert_eq!(
            result,
            Some(("192.168.1.1".to_string(), "8080".to_string()))
        );
    }

    #[test]
    fn test_valid_tw_07_ipv6() {
        let result = get_address("tw-0.7+udp://[2001:db8::1]:8080");
        assert_eq!(
            result,
            Some(("2001:db8::1".to_string(), "8080".to_string()))
        );
    }

    #[test]
    fn test_invalid_port_number() {
        let result = get_address("udp://192.168.1.1:99999");
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_port_format() {
        let result = get_address("udp://192.168.1.1:abc");
        assert_eq!(result, None);
    }

    #[test]
    fn test_missing_port() {
        let result = get_address("udp://192.168.1.1:");
        assert_eq!(result, None);
    }

    #[test]
    fn test_missing_ip() {
        let result = get_address("udp://:8080");
        assert_eq!(result, None);
    }

    #[test]
    fn test_empty_string() {
        let result = get_address("");
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_format() {
        let result = get_address("just some random text");
        assert_eq!(result, None);
    }

    #[test]
    fn test_invalid_protocol() {
        let result = get_address("tcp://192.168.1.1:8080");
        assert_eq!(result, None);
    }

    #[test]
    fn test_valid_min_port() {
        let result = get_address("udp://192.168.1.1:1");
        assert_eq!(result, Some(("192.168.1.1".to_string(), "1".to_string())));
    }

    #[test]
    fn test_valid_max_port() {
        let result = get_address("udp://192.168.1.1:65535");
        assert_eq!(
            result,
            Some(("192.168.1.1".to_string(), "65535".to_string()))
        );
    }

    #[test]
    fn test_complex_ipv6() {
        let result = get_address("udp://[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080");
        assert_eq!(
            result,
            Some((
                "2001:0db8:85a3:0000:0000:8a2e:0370:7334".to_string(),
                "8080".to_string()
            ))
        );
    }

    #[test]
    fn test_localhost_ipv4() {
        let result = get_address("udp://127.0.0.1:8080");
        assert_eq!(result, Some(("127.0.0.1".to_string(), "8080".to_string())));
    }

    #[test]
    fn test_localhost_ipv6() {
        let result = get_address("udp://[::1]:8080");
        assert_eq!(result, Some(("::1".to_string(), "8080".to_string())));
    }

    #[test]
    fn test_health_status_initial_state() {
        use crate::util::HealthStatus;
        let health = HealthStatus::new();
        let (is_healthy, _) = health.check(60);
        assert!(
            !is_healthy,
            "Initial health state must be unhealthy until first successful scrape"
        );
    }

    #[test]
    fn test_health_status_mark_healthy_and_unhealthy() {
        use crate::util::HealthStatus;
        let health = HealthStatus::new();
        health.mark_healthy();
        let (is_healthy, age) = health.check(60);
        assert!(is_healthy);
        assert!(age <= 1);

        health.mark_unhealthy();
        let (is_healthy_now, _) = health.check(60);
        assert!(!is_healthy_now);
    }
}
