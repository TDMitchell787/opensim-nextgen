use anyhow::{anyhow, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error};

#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub first: String,
    pub last: String,
    pub passwd: String,
    pub start: String,
    pub channel: String,
    pub version: String,
    pub platform: String,
    pub user_agent: String,
    pub address_size: String,
    pub cpu_brand: String,
    pub cpu_family: String,
    pub cpu_model: String,
    pub cpu_stepping: String,
    pub cpu_vendor: String,
    pub gpu_brand: String,
    pub gpu_vendor: String,
    pub gpu_version: String,
    pub memory: String,
    pub platform_version: String,
    pub machine_fingerprint: String,
    pub mac: String,
    pub machine_id: String,
    pub disk_serial: String,
    pub read_critical: bool,
    pub last_exec_event: i32,
    pub last_exec_duration: i32,
    pub agree_to_tos: bool,
    pub viewer_digest: String,
    pub options: Vec<String>,
}

impl Default for LoginRequest {
    fn default() -> Self {
        Self {
            first: String::new(),
            last: String::new(),
            passwd: String::new(),
            start: "last".to_string(),
            channel: "OpenSim-Next".to_string(),
            version: "1.0.0".to_string(),
            platform: "Unknown".to_string(),
            user_agent: "OpenSim-Next/1.0".to_string(),
            address_size: "64".to_string(),
            cpu_brand: "Unknown".to_string(),
            cpu_family: "0".to_string(),
            cpu_model: "0".to_string(),
            cpu_stepping: "0".to_string(),
            cpu_vendor: "Unknown".to_string(),
            gpu_brand: "Unknown".to_string(),
            gpu_vendor: "Unknown".to_string(),
            gpu_version: "Unknown".to_string(),
            memory: "0".to_string(),
            platform_version: "Unknown".to_string(),
            machine_fingerprint: "unknown".to_string(),
            mac: "00:00:00:00:00:00".to_string(),
            machine_id: "00000000-0000-0000-0000-000000000000".to_string(),
            disk_serial: "Unknown".to_string(),
            read_critical: true,
            last_exec_event: 0,
            last_exec_duration: 0,
            agree_to_tos: true,
            viewer_digest: "unknown".to_string(),
            options: vec![
                "inventory-root".to_string(),
                "inventory-skeleton".to_string(),
                "inventory-lib-root".to_string(),
                "inventory-lib-owner".to_string(),
                "inventory-skel-lib".to_string(),
                "gestures".to_string(),
                "event_categories".to_string(),
                "event_notifications".to_string(),
                "classified_categories".to_string(),
                "buddy-list".to_string(),
                "ui-config".to_string(),
                "tutorial_setting".to_string(),
                "login-flags".to_string(),
                "global-textures".to_string(),
            ],
        }
    }
}

pub struct XmlRpcParser;

impl XmlRpcParser {
    pub fn parse_login_request(xml_body: &str) -> Result<LoginRequest> {
        debug!("Parsing XMLRPC login request");

        // Verify this is a login_to_simulator request
        if !xml_body.contains("<methodName>login_to_simulator</methodName>") {
            return Err(anyhow!("Not a login_to_simulator request"));
        }

        let mut request = LoginRequest::default();

        // DIAGNOSTIC: Log raw XMLRPC for viewer comparison debugging
        let preview_len = xml_body.len().min(500);
        debug!(
            "🔍 XMLRPC RAW (first {} chars): {}",
            preview_len,
            &xml_body[..preview_len]
        );

        // Simple XML parsing for required fields with diagnostic logging
        let first_result = Self::extract_xml_string_value(xml_body, "first");
        debug!("🔍 EXTRACT 'first': {:?}", first_result);
        request.first = first_result.ok_or_else(|| anyhow!("Missing 'first' field"))?;

        let last_result = Self::extract_xml_string_value(xml_body, "last");
        debug!("🔍 EXTRACT 'last': {:?}", last_result);
        request.last = last_result.ok_or_else(|| anyhow!("Missing 'last' field"))?;

        request.passwd = Self::extract_xml_string_value(xml_body, "passwd")
            .ok_or_else(|| anyhow!("Missing 'passwd' field"))?;

        // Debug log the received password for troubleshooting
        debug!("Received password field: '{}'", request.passwd);

        // Parse optional fields with defaults
        if let Some(start) = Self::extract_xml_string_value(xml_body, "start") {
            request.start = start;
        }

        if let Some(channel) = Self::extract_xml_string_value(xml_body, "channel") {
            request.channel = channel;
        }

        if let Some(version) = Self::extract_xml_string_value(xml_body, "version") {
            request.version = version;
        }

        if let Some(platform) = Self::extract_xml_string_value(xml_body, "platform") {
            request.platform = platform;
        }

        if let Some(user_agent) = Self::extract_xml_string_value(xml_body, "user_agent") {
            request.user_agent = user_agent;
        }

        // Hardware information
        if let Some(address_size) = Self::extract_xml_string_value(xml_body, "address_size") {
            request.address_size = address_size;
        }

        if let Some(cpu_brand) = Self::extract_xml_string_value(xml_body, "cpu_brand") {
            request.cpu_brand = cpu_brand;
        }

        if let Some(memory) = Self::extract_xml_string_value(xml_body, "memory") {
            request.memory = memory;
        }

        // Machine identification
        if let Some(machine_id) = Self::extract_xml_string_value(xml_body, "machine_id") {
            request.machine_id = machine_id;
        }

        if let Some(mac) = Self::extract_xml_string_value(xml_body, "mac") {
            request.mac = mac;
        }

        debug!(
            "Successfully parsed login request for {} {}",
            request.first, request.last
        );
        Ok(request)
    }

    fn extract_xml_string_value(xml_body: &str, field_name: &str) -> Option<String> {
        // Simple XML parsing to find <name>field_name</name> followed by <value><string>VALUE</string></value>
        let name_pattern = format!("<name>{}</name>", field_name);
        if let Some(name_pos) = xml_body.find(&name_pattern) {
            let after_name = &xml_body[name_pos + name_pattern.len()..];
            if let Some(string_start) = after_name.find("<value><string>") {
                let content_start = string_start + "<value><string>".len();
                let content = &after_name[content_start..];
                if let Some(string_end) = content.find("</string></value>") {
                    return Some(content[..string_end].to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_default() {
        let request = LoginRequest::default();
        assert_eq!(request.start, "last");
        assert_eq!(request.channel, "OpenSim-Next");
        assert!(request.agree_to_tos);
        assert!(!request.options.is_empty());
    }

    #[test]
    fn test_parse_simple_login_request() {
        let xml = r#"<?xml version="1.0"?>
<methodCall>
  <methodName>login_to_simulator</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member>
            <name>first</name>
            <value><string>John</string></value>
          </member>
          <member>
            <name>last</name>
            <value><string>Doe</string></value>
          </member>
          <member>
            <name>passwd</name>
            <value><string>$1$5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8</string></value>
          </member>
          <member>
            <name>start</name>
            <value><string>last</string></value>
          </member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>"#;

        let request = XmlRpcParser::parse_login_request(xml).unwrap();
        assert_eq!(request.first, "John");
        assert_eq!(request.last, "Doe");
        assert!(request.passwd.starts_with("$1$"));
        assert_eq!(request.start, "last");
    }
}
