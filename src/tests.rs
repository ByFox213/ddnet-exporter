#[cfg(test)]
mod tests {
    use crate::util::get_address;

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
}
