use uuid::Uuid;

pub fn format_uui(user_id: Uuid, home_uri: &str, first: &str, last: &str) -> String {
    format!("{};{};{} {}", user_id, home_uri, first, last)
}

pub fn parse_uui(uui: &str) -> Option<(Uuid, String, String, String)> {
    let parts: Vec<&str> = uui.splitn(3, ';').collect();
    if parts.len() < 3 {
        if parts.len() == 1 {
            let uuid = Uuid::parse_str(parts[0]).ok()?;
            return Some((uuid, String::new(), String::new(), String::new()));
        }
        return None;
    }

    let uuid = Uuid::parse_str(parts[0]).ok()?;
    let home_uri = parts[1].to_string();
    let name = parts[2];

    let (first, last) = if let Some(pos) = name.find(' ') {
        (name[..pos].to_string(), name[pos + 1..].to_string())
    } else {
        (name.to_string(), String::new())
    };

    Some((uuid, home_uri, first, last))
}

pub fn format_foreign_name(first: &str, last: &str, home_uri: &str) -> String {
    let host = home_uri
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/');
    format!("{}.{} @{}", first, last, host)
}

pub fn is_foreign_user(last_name: &str) -> bool {
    last_name.contains('@')
}

pub fn extract_home_uri(last_name: &str) -> Option<String> {
    if let Some(pos) = last_name.find('@') {
        let host = last_name[pos + 1..].trim();
        if host.is_empty() {
            return None;
        }
        if host.contains("://") {
            Some(host.to_string())
        } else {
            Some(format!("http://{}", host))
        }
    } else {
        None
    }
}

fn decode_region_name(name: &str) -> String {
    urlencoding::decode(name).unwrap_or(std::borrow::Cow::Borrowed(name)).into_owned()
}

pub fn parse_hg_url(input: &str) -> Option<(String, u16, String)> {
    let trimmed = input.trim();
    let trimmed = if let Some(stripped) = trimmed.strip_prefix("hop://") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("hop:") {
        stripped.trim_start_matches('/')
    } else if let Some(stripped) = trimmed.strip_prefix("hg:") {
        stripped.trim_start_matches('/')
    } else {
        trimmed
    };

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let without_scheme = if trimmed.starts_with("https://") {
            &trimmed[8..]
        } else {
            &trimmed[7..]
        };
        let scheme = if trimmed.starts_with("https://") { "https://" } else { "http://" };

        let (host_port, region) = if let Some(slash_pos) = without_scheme.find('/') {
            let hp = &without_scheme[..slash_pos];
            let r = without_scheme[slash_pos + 1..].to_string();
            (hp, r)
        } else {
            (without_scheme, String::new())
        };

        let (host, port) = if let Some(colon_pos) = host_port.rfind(':') {
            let h = &host_port[..colon_pos];
            let p = host_port[colon_pos + 1..].parse::<u16>().unwrap_or(80);
            (h, p)
        } else {
            (host_port, 80u16)
        };

        let gatekeeper_url = format!("{}{}:{}", scheme, host, port);
        return Some((gatekeeper_url, port, decode_region_name(&region)));
    }

    let trimmed = trimmed.trim_end_matches('/');

    let parts: Vec<&str> = trimmed.splitn(3, ':').collect();
    match parts.len() {
        3 => {
            let host = parts[0];
            let port = parts[1].parse::<u16>().ok()?;
            let region = parts[2].trim_start_matches('/').to_string();
            let gatekeeper_url = format!("http://{}:{}", host, port);
            Some((gatekeeper_url, port, decode_region_name(&region)))
        }
        2 => {
            let host = parts[0];
            let (port_part, slash_region) = if let Some(slash_pos) = parts[1].find('/') {
                (&parts[1][..slash_pos], Some(parts[1][slash_pos + 1..].to_string()))
            } else {
                (parts[1], None)
            };
            if let Ok(port) = port_part.parse::<u16>() {
                let region = slash_region.unwrap_or_default();
                let gatekeeper_url = format!("http://{}:{}", host, port);
                Some((gatekeeper_url, port, decode_region_name(&region)))
            } else {
                let gatekeeper_url = format!("http://{}:80", host);
                Some((gatekeeper_url, 80, decode_region_name(parts[1])))
            }
        }
        1 => {
            if let Some(slash_pos) = parts[0].find('/') {
                let host = &parts[0][..slash_pos];
                let region = parts[0][slash_pos + 1..].trim_end_matches('/').to_string();
                let gatekeeper_url = format!("http://{}:80", host);
                Some((gatekeeper_url, 80, decode_region_name(&region)))
            } else {
                let gatekeeper_url = format!("http://{}:80", parts[0]);
                Some((gatekeeper_url, 80, String::new()))
            }
        }
        _ => None,
    }
}

pub fn is_hg_destination(destination: &str) -> bool {
    let trimmed = destination.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return true;
    }
    if trimmed.starts_with("hop://") || trimmed.starts_with("hop:") {
        return true;
    }
    let check = if let Some(stripped) = trimmed.strip_prefix("hg:") {
        stripped.trim_start_matches('/')
    } else {
        trimmed
    };
    if check.contains(':') && !check.starts_with("secondlife://") {
        let first_part = check.split(':').next().unwrap_or("");
        return first_part.contains('.');
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uui() {
        let uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let uui = format_uui(uuid, "http://grid.example.com:8002", "Test", "User");
        assert_eq!(uui, "00000000-0000-0000-0000-000000000001;http://grid.example.com:8002;Test User");
    }

    #[test]
    fn test_parse_uui() {
        let uui = "00000000-0000-0000-0000-000000000001;http://grid.example.com:8002;Test User";
        let (uuid, home, first, last) = parse_uui(uui).unwrap();
        assert_eq!(uuid.to_string(), "00000000-0000-0000-0000-000000000001");
        assert_eq!(home, "http://grid.example.com:8002");
        assert_eq!(first, "Test");
        assert_eq!(last, "User");
    }

    #[test]
    fn test_parse_uui_uuid_only() {
        let uui = "00000000-0000-0000-0000-000000000001";
        let (uuid, home, first, last) = parse_uui(uui).unwrap();
        assert_eq!(uuid.to_string(), "00000000-0000-0000-0000-000000000001");
        assert!(home.is_empty());
        assert!(first.is_empty());
        assert!(last.is_empty());
    }

    #[test]
    fn test_format_foreign_name() {
        let name = format_foreign_name("Test", "User", "http://grid.example.com:8002");
        assert_eq!(name, "Test.User @grid.example.com:8002");
    }

    #[test]
    fn test_is_foreign_user() {
        assert!(is_foreign_user("User @grid.example.com:8002"));
        assert!(!is_foreign_user("User"));
    }

    #[test]
    fn test_extract_home_uri() {
        let uri = extract_home_uri("User @grid.example.com:8002").unwrap();
        assert_eq!(uri, "http://grid.example.com:8002");

        let uri2 = extract_home_uri("User @http://grid.example.com:8002").unwrap();
        assert_eq!(uri2, "http://grid.example.com:8002");
    }

    #[test]
    fn test_parse_hg_url() {
        let (gk, port, region) = parse_hg_url("hg.osgrid.org:80:Welcome").unwrap();
        assert_eq!(gk, "http://hg.osgrid.org:80");
        assert_eq!(port, 80);
        assert_eq!(region, "Welcome");

        let (gk2, port2, region2) = parse_hg_url("http://grid.example.com:8002/MyRegion").unwrap();
        assert_eq!(gk2, "http://grid.example.com:8002");
        assert_eq!(port2, 8002);
        assert_eq!(region2, "MyRegion");
    }

    #[test]
    fn test_is_hg_destination() {
        assert!(is_hg_destination("hg.osgrid.org:80:Welcome"));
        assert!(is_hg_destination("http://grid.example.com:8002/Welcome"));
        assert!(is_hg_destination("hg:alternatemetaverse.com:8002/"));
        assert!(is_hg_destination("hg:alternatemetaverse.com:8002"));
        assert!(!is_hg_destination("Welcome Region"));
        assert!(!is_hg_destination("secondlife://region/128/128/21"));
    }

    #[test]
    fn test_parse_hg_url_with_hg_prefix() {
        let (gk, port, region) = parse_hg_url("hg:alternatemetaverse.com:8002/").unwrap();
        assert_eq!(gk, "http://alternatemetaverse.com:8002");
        assert_eq!(port, 8002);
        assert_eq!(region, "");

        let (gk2, port2, region2) = parse_hg_url("hg:grid.example.com:8002:Welcome").unwrap();
        assert_eq!(gk2, "http://grid.example.com:8002");
        assert_eq!(port2, 8002);
        assert_eq!(region2, "Welcome");
    }

    #[test]
    fn test_hop_protocol() {
        assert!(is_hg_destination("hop://hg.osgrid.org:80/Welcome"));
        assert!(is_hg_destination("hop://grid.example.com:8002/MyRegion"));
        assert!(is_hg_destination("hop://grid.example.com:8002"));
        assert!(is_hg_destination("hop:grid.example.com:8002"));

        let (gk, port, region) = parse_hg_url("hop://hg.osgrid.org:80/Welcome").unwrap();
        assert_eq!(gk, "http://hg.osgrid.org:80");
        assert_eq!(port, 80);
        assert_eq!(region, "Welcome");

        let (gk2, port2, region2) = parse_hg_url("hop://grid.example.com:8002/MyRegion").unwrap();
        assert_eq!(gk2, "http://grid.example.com:8002");
        assert_eq!(port2, 8002);
        assert_eq!(region2, "MyRegion");

        let (gk3, port3, region3) = parse_hg_url("hop://grid.example.com:8002").unwrap();
        assert_eq!(gk3, "http://grid.example.com:8002");
        assert_eq!(port3, 8002);
        assert_eq!(region3, "");

        let (gk4, port4, region4) = parse_hg_url("hop://hg.osgrid.org/Welcome").unwrap();
        assert_eq!(gk4, "http://hg.osgrid.org:80");
        assert_eq!(port4, 80);
        assert_eq!(region4, "Welcome");
    }

    #[test]
    fn test_percent_encoded_region_names() {
        let (gk, port, region) = parse_hg_url("grid.wolfterritories.org:8002:Lunaria%20Emporium%20-%20Main%20Store").unwrap();
        assert_eq!(gk, "http://grid.wolfterritories.org:8002");
        assert_eq!(port, 8002);
        assert_eq!(region, "Lunaria Emporium - Main Store");

        let (gk2, port2, region2) = parse_hg_url("http://grid.example.com:8002/My%20Region%20Name").unwrap();
        assert_eq!(gk2, "http://grid.example.com:8002");
        assert_eq!(port2, 8002);
        assert_eq!(region2, "My Region Name");

        let (gk3, port3, region3) = parse_hg_url("hop://hg.osgrid.org:80/Welcome%20Area").unwrap();
        assert_eq!(gk3, "http://hg.osgrid.org:80");
        assert_eq!(port3, 80);
        assert_eq!(region3, "Welcome Area");
    }
}
