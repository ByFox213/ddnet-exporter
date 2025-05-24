#[cfg(test)]
mod tests {
    use crate::util::get_address;

    #[test]
    fn test_ipv4() {
        assert_eq!(
            get_address("udp://127.0.0.1:8303"),
            Some(("127.0.0.1".into(), "8303".into()))
        );
        assert_eq!(
            get_address("tw-0.6+udp://192.168.1.1:8304"),
            Some(("192.168.1.1".into(), "8304".into()))
        );
        assert_eq!(
            get_address("tw-0.7+udp://10.0.0.1:8305"),
            Some(("10.0.0.1".into(), "8305".into()))
        );
    }

    #[test]
    fn test_ipv6_udp() {
        assert_eq!(
            get_address("udp://[::1]:8303"),
            Some(("::1".into(), "8303".into()))
        );
        assert_eq!(
            get_address("tw-0.6+udp://[2001:db8::1]:8304"),
            Some(("2001:db8::1".into(), "8304".into()))
        );
        assert_eq!(
            get_address("tw-0.7+udp://[fe80::1]:8305"),
            Some(("fe80::1".into(), "8305".into()))
        );
    }

    #[test]
    fn test_invalid_address() {
        assert!(get_address("invalid").is_none());
        assert!(get_address("tw-0.7+udp://invalid:port").is_none());
    }
}
