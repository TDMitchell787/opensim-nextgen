mod xml_roundtrip {
    use opensim_next::services::robust::xml_response::*;

    #[test]
    fn test_success_result_roundtrip() {
        let xml = success_result();
        assert!(xml.contains("<ServerResponse>"));
        assert!(xml.contains("<Result>Success</Result>"));

        let parsed = parse_xml_response(&xml).unwrap();
        assert_eq!(parsed.get("Result").unwrap().as_str(), Some("Success"));
    }

    #[test]
    fn test_failure_result_roundtrip() {
        let xml = failure_result("Region not found");
        let parsed = parse_xml_response(&xml).unwrap();
        assert_eq!(parsed.get("Result").unwrap().as_str(), Some("Failure"));
        assert_eq!(parsed.get("Message").unwrap().as_str(), Some("Region not found"));
    }

    #[test]
    fn test_null_result_roundtrip() {
        let xml = null_result();
        let parsed = parse_xml_response(&xml).unwrap();
        assert_eq!(parsed.get("result").unwrap().as_str(), Some("null"));
    }

    #[test]
    fn test_bool_result_true() {
        let xml = bool_result(true);
        let parsed = parse_xml_response(&xml).unwrap();
        assert_eq!(parsed.get("result").unwrap().as_str(), Some("true"));
    }

    #[test]
    fn test_bool_result_false() {
        let xml = bool_result(false);
        let parsed = parse_xml_response(&xml).unwrap();
        assert_eq!(parsed.get("result").unwrap().as_str(), Some("false"));
    }

    #[test]
    fn test_single_result_roundtrip() {
        let mut fields = std::collections::HashMap::new();
        fields.insert("uuid".to_string(), "00000000-0000-0000-0000-000000000001".to_string());
        fields.insert("regionName".to_string(), "Test Region".to_string());
        fields.insert("locX".to_string(), "1000".to_string());
        fields.insert("locY".to_string(), "1000".to_string());

        let xml = single_result(fields);
        let parsed = parse_xml_response(&xml).unwrap();

        let result = parsed.get("result").unwrap().as_dict().unwrap();
        assert_eq!(result.get("uuid").unwrap().as_str(), Some("00000000-0000-0000-0000-000000000001"));
        assert_eq!(result.get("regionName").unwrap().as_str(), Some("Test Region"));
        assert_eq!(result.get("locX").unwrap().as_str(), Some("1000"));
    }

    #[test]
    fn test_list_result_roundtrip() {
        let mut region1 = std::collections::HashMap::new();
        region1.insert("uuid".to_string(), "id-1".to_string());
        region1.insert("regionName".to_string(), "Region One".to_string());

        let mut region2 = std::collections::HashMap::new();
        region2.insert("uuid".to_string(), "id-2".to_string());
        region2.insert("regionName".to_string(), "Region Two".to_string());

        let xml = list_result("region", vec![region1, region2]);
        let parsed = parse_xml_response(&xml).unwrap();

        let r0 = parsed.get("region0").unwrap().as_dict().unwrap();
        assert_eq!(r0.get("uuid").unwrap().as_str(), Some("id-1"));
        let r1 = parsed.get("region1").unwrap().as_dict().unwrap();
        assert_eq!(r1.get("uuid").unwrap().as_str(), Some("id-2"));
    }

    #[test]
    fn test_empty_list_returns_null() {
        let xml = list_result("region", vec![]);
        let parsed = parse_xml_response(&xml).unwrap();
        assert_eq!(parsed.get("result").unwrap().as_str(), Some("null"));
    }

    #[test]
    fn test_xml_name_encoding() {
        let mut fields = std::collections::HashMap::new();
        fields.insert("Wearable 0:0".to_string(), "some-uuid:some-asset".to_string());
        let xml = single_result(fields);
        assert!(xml.contains("Wearable_x0020_0_x003A_0"));

        let parsed = parse_xml_response(&xml).unwrap();
        let result = parsed.get("result").unwrap().as_dict().unwrap();
        assert_eq!(
            result.get("Wearable 0:0").unwrap().as_str(),
            Some("some-uuid:some-asset")
        );
    }

    #[test]
    fn test_form_body_parsing() {
        let body = "METHOD=register&REGIONID=abc-123&REGIONNAME=Test%20Region&LOCX=1000";
        let result = parse_form_body(body);
        assert_eq!(result.get("METHOD").unwrap(), "register");
        assert_eq!(result.get("REGIONID").unwrap(), "abc-123");
        assert_eq!(result.get("REGIONNAME").unwrap(), "Test Region");
        assert_eq!(result.get("LOCX").unwrap(), "1000");
    }

    #[test]
    fn test_try_parse_xml_to_flat_with_xml() {
        let xml = r#"<?xml version="1.0"?>
<ServerResponse>
  <result>Success</result>
  <token>abc123</token>
</ServerResponse>"#;
        let result = try_parse_xml_to_flat(xml).unwrap();
        assert_eq!(result.get("result").unwrap(), "Success");
        assert_eq!(result.get("token").unwrap(), "abc123");
    }

    #[test]
    fn test_try_parse_xml_to_flat_with_form() {
        let body = "result=Success\ntoken=abc123";
        let result = try_parse_xml_to_flat(body);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_parse_xml_to_flat_nested_dict() {
        let mut fields = std::collections::HashMap::new();
        fields.insert("uuid".to_string(), "test-id".to_string());
        fields.insert("regionName".to_string(), "My Region".to_string());
        let xml = single_result(fields);

        let flat = try_parse_xml_to_flat(&xml).unwrap();
        assert_eq!(flat.get("uuid").unwrap(), "test-id");
        assert_eq!(flat.get("regionName").unwrap(), "My Region");
    }
}

mod service_config {
    use opensim_next::services::traits::{ServiceConfig, ServiceMode};
    use opensim_next::services::config_parser::*;

    #[test]
    fn test_default_config() {
        let config = ServiceConfig::default();
        assert_eq!(config.mode, ServiceMode::Standalone);
        assert!(config.grid_server_uri.is_none());
        assert!(config.asset_server_uri.is_none());
    }

    #[test]
    fn test_build_standalone_config() {
        let config = build_service_config("standalone");
        assert_eq!(config.mode, ServiceMode::Standalone);
    }

    #[test]
    fn test_build_grid_config() {
        let config = build_service_config("grid");
        assert_eq!(config.mode, ServiceMode::Grid);
        assert!(config.grid_server_uri.is_some());
    }

    #[test]
    fn test_build_robust_config() {
        let config = build_service_config("robust");
        assert_eq!(config.mode, ServiceMode::Standalone);
    }
}

mod service_factory {
    use opensim_next::services::traits::{ServiceConfig, ServiceMode};
    use opensim_next::services::factory::ServiceFactory;

    #[test]
    fn test_create_remote_grid_service() {
        let config = ServiceConfig {
            mode: ServiceMode::Grid,
            grid_server_uri: Some("http://localhost:8003".to_string()),
            ..Default::default()
        };
        let result = ServiceFactory::create_grid_service(&config, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_remote_user_account_service() {
        let config = ServiceConfig {
            mode: ServiceMode::Grid,
            user_account_server_uri: Some("http://localhost:8003".to_string()),
            ..Default::default()
        };
        let result = ServiceFactory::create_user_account_service(&config, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_remote_asset_service() {
        let config = ServiceConfig {
            mode: ServiceMode::Grid,
            asset_server_uri: Some("http://localhost:8003".to_string()),
            ..Default::default()
        };
        let result = ServiceFactory::create_asset_service(&config, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_remote_auth_service() {
        let config = ServiceConfig {
            mode: ServiceMode::Grid,
            authentication_server_uri: Some("http://localhost:8003".to_string()),
            ..Default::default()
        };
        let result = ServiceFactory::create_authentication_service(&config, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_remote_inventory_service() {
        let config = ServiceConfig {
            mode: ServiceMode::Grid,
            inventory_server_uri: Some("http://localhost:8003".to_string()),
            ..Default::default()
        };
        let result = ServiceFactory::create_inventory_service(&config, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_remote_presence_service() {
        let config = ServiceConfig {
            mode: ServiceMode::Grid,
            presence_server_uri: Some("http://localhost:8003".to_string()),
            ..Default::default()
        };
        let result = ServiceFactory::create_presence_service(&config, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_remote_avatar_service() {
        let config = ServiceConfig {
            mode: ServiceMode::Grid,
            avatar_server_uri: Some("http://localhost:8003".to_string()),
            ..Default::default()
        };
        let result = ServiceFactory::create_avatar_service(&config, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_standalone_requires_db() {
        let config = ServiceConfig {
            mode: ServiceMode::Standalone,
            ..Default::default()
        };
        let result = ServiceFactory::create_grid_service(&config, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_grid_uses_fallback_uri() {
        let config = ServiceConfig {
            mode: ServiceMode::Grid,
            grid_server_uri: Some("http://robust.example.com:8003".to_string()),
            ..Default::default()
        };
        let result = ServiceFactory::create_user_account_service(&config, None);
        assert!(result.is_ok());
    }
}

mod robust_state {
    use opensim_next::services::robust::RobustState;

    #[test]
    fn test_robust_state_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RobustState>();
    }
}
