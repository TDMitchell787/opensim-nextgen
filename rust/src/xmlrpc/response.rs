use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use tracing::{debug, info};
use crate::session::LoginSession;
use crate::inventory::{LoginInventoryResponse, LoginInventoryFolder, LoginInventoryOwner};


#[derive(Debug, Clone)]
pub struct LoginResponse {
    pub login: String,
    pub agent_id: String,
    pub real_id: String,
    pub session_id: String,
    pub secure_session_id: String,
    pub first_name: String,
    pub last_name: String,
    pub agent_access: String,
    pub agent_access_max: String,
    pub start_location: String,
    pub circuit_code: u32,
    pub sim_ip: String,
    pub sim_port: u16,
    pub http_port: u16,
    pub region_x: u32,
    pub region_y: u32,
    pub region_size_x: u32,
    pub region_size_y: u32,
    pub look_at: String,
    pub seed_capability: String,
    pub home: String,
    pub message: String,
    pub seconds_since_epoch: i64,
    pub inventory_root: Vec<LoginInventoryFolder>,
    pub inventory_skeleton: Vec<LoginInventoryFolder>,
    pub inventory_lib_root: Vec<LoginInventoryFolder>,
    pub inventory_lib_owner: Vec<LoginInventoryOwner>,
    pub inventory_skel_lib: Vec<LoginInventoryFolder>,
    pub buddy_list: Vec<String>,
    pub max_agent_groups: i32,
    pub stipend_since_login: String,
    pub dst: String,
    pub gendered: String,
    pub ever_logged_in: String,
    pub sun_texture_id: String,
    pub cloud_texture_id: String,
    pub moon_texture_id: String,
    pub allow_first_life: String,
    pub map_server_url: String,
    pub service_urls: HashMap<String, String>,
    pub destination_guide_url: Option<String>,
    pub currency_base_uri: Option<String>,
}

impl LoginResponse {
    pub fn success(session: &LoginSession, base_url: &str, inventory: &LoginInventoryResponse, seed_capability_url: &str) -> Self {
        let now = Utc::now();

        Self {
            login: "true".to_string(),
            agent_id: session.agent_id.to_string(),
            real_id: session.agent_id.to_string(),
            session_id: session.session_id.to_string(),
            secure_session_id: session.secure_session_id.to_string(),
            first_name: session.first_name.clone(),
            last_name: session.last_name.clone(),
            agent_access: "M".to_string(),
            agent_access_max: "M".to_string(),
            start_location: "last".to_string(),
            circuit_code: session.circuit_code,
            sim_ip: session.sim_ip.clone(),
            sim_port: session.sim_port,
            http_port: session.sim_port,
            region_x: session.region_x,
            region_y: session.region_y,
            region_size_x: session.region_size_x,
            region_size_y: session.region_size_y,
            look_at: "[r0.5,r1,r0]".to_string(),
            seed_capability: seed_capability_url.to_string(),
            // CRITICAL: region_handle uses METER coordinates (e.g., 256000), not grid coordinates (1000)
            home: format!(
                "{{'region_handle':[r{},r{}], 'position':[r128,r128,r25], 'look_at':[r0,r1,r0]}}",
                session.region_x, session.region_y  // session.region_x/y should already be meters
            ),
            message: "Welcome to OpenSim Next!".to_string(),
            seconds_since_epoch: now.timestamp(),
            inventory_root: inventory.inventory_root.clone(),
            inventory_skeleton: inventory.inventory_skeleton.clone(),
            inventory_lib_root: inventory.inventory_lib_root.clone(),
            inventory_lib_owner: inventory.inventory_lib_owner.clone(),
            inventory_skel_lib: inventory.inventory_skel_lib.clone(),
            buddy_list: vec![],
            max_agent_groups: 42,
            stipend_since_login: "N".to_string(),
            dst: "Y".to_string(),
            gendered: "Y".to_string(),
            ever_logged_in: "Y".to_string(),
            sun_texture_id: "cce0f112-878f-4586-a2e2-a8f104bba271".to_string(),
            cloud_texture_id: "dc4b9f0b-d008-45c6-96a4-01dd947ac621".to_string(),
            moon_texture_id: "ec4b9f0b-d008-45c6-96a4-01dd947ac621".to_string(),
            allow_first_life: "Y".to_string(),
            map_server_url: format!("{}/t/", base_url.trim_end_matches('/')),
            service_urls: HashMap::new(),
            destination_guide_url: None,
            currency_base_uri: None,
        }
    }

    pub fn failure(reason: &str, message: &str) -> Self {
        Self {
            login: "false".to_string(),
            agent_id: "00000000-0000-0000-0000-000000000000".to_string(),
            real_id: "00000000-0000-0000-0000-000000000000".to_string(),
            session_id: "00000000-0000-0000-0000-000000000000".to_string(),
            secure_session_id: "00000000-0000-0000-0000-000000000000".to_string(),
            first_name: "".to_string(),
            last_name: "".to_string(),
            agent_access: "".to_string(),
            agent_access_max: "".to_string(),
            start_location: "".to_string(),
            circuit_code: 0,
            sim_ip: "".to_string(),
            sim_port: 0,
            http_port: 0,
            region_x: 0,
            region_y: 0,
            region_size_x: 0,
            region_size_y: 0,
            look_at: "".to_string(),
            seed_capability: "".to_string(),
            home: "".to_string(),
            message: message.to_string(),
            seconds_since_epoch: Utc::now().timestamp(),
            inventory_root: vec![],
            inventory_skeleton: vec![],
            inventory_lib_root: vec![],
            inventory_lib_owner: vec![],
            inventory_skel_lib: vec![],
            buddy_list: vec![],
            max_agent_groups: 0,
            stipend_since_login: "N".to_string(),
            dst: "".to_string(),
            gendered: "".to_string(),
            ever_logged_in: "".to_string(),
            sun_texture_id: "".to_string(),
            cloud_texture_id: "".to_string(),
            moon_texture_id: "".to_string(),
            allow_first_life: "".to_string(),
            map_server_url: String::new(),
            service_urls: HashMap::new(),
            destination_guide_url: None,
            currency_base_uri: None,
        }
    }
}

pub struct XmlRpcResponseGenerator;

impl XmlRpcResponseGenerator {
    pub fn generate_login_response(response: &LoginResponse) -> Result<String> {
        debug!("Generating XMLRPC login response");
        
        if response.login == "true" {
            Self::generate_success_response(response)
        } else {
            Self::generate_failure_response(response)
        }
    }
    
    fn generate_success_response(response: &LoginResponse) -> Result<String> {
        let mut xml = String::new();
        
        xml.push_str("<?xml version=\"1.0\"?>\n");
        xml.push_str("<methodResponse>\n");
        xml.push_str("  <params>\n");
        xml.push_str("    <param>\n");
        xml.push_str("      <value>\n");
        xml.push_str("        <struct>\n");
        
        // Authentication Status
        xml.push_str("          <member>\n");
        xml.push_str("            <name>login</name>\n");
        xml.push_str("            <value><boolean>1</boolean></value>\n");
        xml.push_str("          </member>\n");
        
        // Agent Identification
        Self::add_string_member(&mut xml, "agent_id", &response.agent_id);
        Self::add_string_member(&mut xml, "real_id", &response.real_id);
        Self::add_string_member(&mut xml, "session_id", &response.session_id);
        Self::add_string_member(&mut xml, "secure_session_id", &response.secure_session_id);
        
        // User Information
        Self::add_string_member(&mut xml, "first_name", &response.first_name);
        Self::add_string_member(&mut xml, "last_name", &response.last_name);
        Self::add_string_member(&mut xml, "agent_access", &response.agent_access);
        Self::add_string_member(&mut xml, "agent_access_max", &response.agent_access_max);
        
        // Connection Details
        Self::add_string_member(&mut xml, "start_location", &response.start_location);
        Self::add_u32_member(&mut xml, "circuit_code", response.circuit_code);
        Self::add_string_member(&mut xml, "sim_ip", &response.sim_ip);
        Self::add_int_member(&mut xml, "sim_port", response.sim_port as i32);
        Self::add_int_member(&mut xml, "http_port", response.http_port as i32);
        
        // Region Information
        // CRITICAL: region_x/region_y must be in METERS (not grid coordinates)
        // OpenSim LLLoginResponse.cs line 535: responseData["region_x"] = (Int32)(RegionX);
        // Where RegionX is from GridRegion.RegionLocX which is in meters (IGridService.cs line 234)
        // The viewer uses these to calculate region_handle for getRegionFromHandle() check
        Self::add_int_member(&mut xml, "region_x", response.region_x as i32);
        Self::add_int_member(&mut xml, "region_y", response.region_y as i32);
        Self::add_int_member(&mut xml, "region_size_x", response.region_size_x as i32);
        Self::add_int_member(&mut xml, "region_size_y", response.region_size_y as i32);
        
        // Avatar Initial State
        Self::add_string_member(&mut xml, "look_at", &response.look_at);
        Self::add_string_member(&mut xml, "seed_capability", &response.seed_capability);
        Self::add_string_member(&mut xml, "home", &response.home);
        Self::add_string_member(&mut xml, "message", &response.message);
        Self::add_int_member(&mut xml, "seconds_since_epoch", response.seconds_since_epoch as i32);
        
        // Inventory Structure
        Self::add_inventory_array(&mut xml, "inventory-root", &response.inventory_root);
        Self::add_inventory_array(&mut xml, "inventory-skeleton", &response.inventory_skeleton);
        Self::add_inventory_array(&mut xml, "inventory-lib-root", &response.inventory_lib_root);

        // Buddy List
        Self::add_string_array(&mut xml, "buddy-list", &response.buddy_list);

        // System Configuration
        Self::add_int_member(&mut xml, "max-agent-groups", response.max_agent_groups);
        Self::add_string_member(&mut xml, "stipend_since_login", &response.stipend_since_login);

        // Login Flags Array
        Self::add_login_flags_array(&mut xml, response);

        // Global Textures Array
        Self::add_global_textures_array(&mut xml, response);

        // Empty Arrays
        Self::add_empty_array(&mut xml, "event_categories");
        Self::add_empty_array(&mut xml, "event_notifications");
        Self::add_empty_array(&mut xml, "classified_categories");

        // UI Config Array
        Self::add_ui_config_array(&mut xml, response);

        if !response.map_server_url.is_empty() {
            Self::add_string_member(&mut xml, "map-server-url", &response.map_server_url);
        }

        if let Some(ref url) = response.destination_guide_url {
            Self::add_string_member(&mut xml, "destination_guide_url", url);
        }
        if let Some(ref url) = response.currency_base_uri {
            Self::add_string_member(&mut xml, "currency", url);
        }

        // Library Inventory Arrays
        Self::add_inventory_array(&mut xml, "inventory-skel-lib", &response.inventory_skel_lib);
        Self::add_inventory_owner_array(&mut xml, "inventory-lib-owner", &response.inventory_lib_owner);

        // Gesture and Outfit Arrays
        Self::add_empty_array(&mut xml, "gestures");

        // Phase 68.7: Add initial-outfit with default outfit to set isOutfitChosen() = true
        // This is CRITICAL for viewer to transition from "Connecting to Region" to 3D view
        Self::add_initial_outfit_array(&mut xml);

        for (key, value) in &response.service_urls {
            Self::add_string_member(&mut xml, key, value);
        }

        xml.push_str("        </struct>\n");
        xml.push_str("      </value>\n");
        xml.push_str("    </param>\n");
        xml.push_str("  </params>\n");
        xml.push_str("</methodResponse>\n");

        debug!("XMLRPC response generated ({} bytes)", xml.len());

        Ok(xml)
    }
    
    fn generate_failure_response(response: &LoginResponse) -> Result<String> {
        let mut xml = String::new();
        
        xml.push_str("<?xml version=\"1.0\"?>\n");
        xml.push_str("<methodResponse>\n");
        xml.push_str("  <params>\n");
        xml.push_str("    <param>\n");
        xml.push_str("      <value>\n");
        xml.push_str("        <struct>\n");
        
        xml.push_str("          <member>\n");
        xml.push_str("            <name>login</name>\n");
        xml.push_str("            <value><string>false</string></value>\n");
        xml.push_str("          </member>\n");
        
        xml.push_str("          <member>\n");
        xml.push_str("            <name>reason</name>\n");
        xml.push_str("            <value><string>key</string></value>\n");
        xml.push_str("          </member>\n");
        
        Self::add_string_member(&mut xml, "message", &response.message);
        
        xml.push_str("        </struct>\n");
        xml.push_str("      </value>\n");
        xml.push_str("    </param>\n");
        xml.push_str("  </params>\n");
        xml.push_str("</methodResponse>\n");
        
        Ok(xml)
    }
    
    fn add_string_member(xml: &mut String, name: &str, value: &str) {
        xml.push_str(&format!("          <member>\n"));
        xml.push_str(&format!("            <name>{}</name>\n", name));
        xml.push_str(&format!("            <value><string>{}</string></value>\n", Self::escape_xml(value)));
        xml.push_str("          </member>\n");
    }
    
    fn add_int_member(xml: &mut String, name: &str, value: i32) {
        xml.push_str(&format!("          <member>\n"));
        xml.push_str(&format!("            <name>{}</name>\n", name));
        xml.push_str(&format!("            <value><int>{}</int></value>\n", value));
        xml.push_str("          </member>\n");
    }

    fn add_u32_member(xml: &mut String, name: &str, value: u32) {
        xml.push_str(&format!("          <member>\n"));
        xml.push_str(&format!("            <name>{}</name>\n", name));
        xml.push_str(&format!("            <value><int>{}</int></value>\n", value));
        xml.push_str("          </member>\n");
    }

    fn add_inventory_array(xml: &mut String, name: &str, folders: &[LoginInventoryFolder]) {
        xml.push_str(&format!("          <member>\n"));
        xml.push_str(&format!("            <name>{}</name>\n", name));
        xml.push_str("            <value>\n");
        xml.push_str("              <array>\n");
        xml.push_str("                <data>\n");
        
        for folder in folders {
            xml.push_str("                  <value>\n");
            xml.push_str("                    <struct>\n");
            
            Self::add_string_member_indent(xml, "folder_id", &folder.folder_id, 22);
            Self::add_string_member_indent(xml, "parent_id", &folder.parent_id, 22);
            Self::add_string_member_indent(xml, "name", &folder.name, 22);
            Self::add_string_member_indent(xml, "type_default", &folder.type_default, 22);
            Self::add_string_member_indent(xml, "version", &folder.version, 22);
            
            xml.push_str("                    </struct>\n");
            xml.push_str("                  </value>\n");
        }
        
        xml.push_str("                </data>\n");
        xml.push_str("              </array>\n");
        xml.push_str("            </value>\n");
        xml.push_str("          </member>\n");
    }

    fn add_inventory_owner_array(xml: &mut String, name: &str, owners: &[LoginInventoryOwner]) {
        xml.push_str(&format!("          <member>\n"));
        xml.push_str(&format!("            <name>{}</name>\n", name));
        xml.push_str("            <value>\n");
        xml.push_str("              <array>\n");
        xml.push_str("                <data>\n");

        for owner in owners {
            xml.push_str("                  <value>\n");
            xml.push_str("                    <struct>\n");

            Self::add_string_member_indent(xml, "agent_id", &owner.agent_id, 22);

            xml.push_str("                    </struct>\n");
            xml.push_str("                  </value>\n");
        }

        xml.push_str("                </data>\n");
        xml.push_str("              </array>\n");
        xml.push_str("            </value>\n");
        xml.push_str("          </member>\n");
    }

    fn add_string_array(xml: &mut String, name: &str, values: &[String]) {
        xml.push_str(&format!("          <member>\n"));
        xml.push_str(&format!("            <name>{}</name>\n", name));
        xml.push_str("            <value>\n");
        xml.push_str("              <array>\n");
        xml.push_str("                <data>\n");
        
        for value in values {
            xml.push_str(&format!("                  <value><string>{}</string></value>\n", Self::escape_xml(value)));
        }
        
        xml.push_str("                </data>\n");
        xml.push_str("              </array>\n");
        xml.push_str("            </value>\n");
        xml.push_str("          </member>\n");
    }
    
    fn add_string_member_indent(xml: &mut String, name: &str, value: &str, indent: usize) {
        let spaces = " ".repeat(indent);
        xml.push_str(&format!("{}  <member>\n", spaces));
        xml.push_str(&format!("{}    <name>{}</name>\n", spaces, name));
        xml.push_str(&format!("{}    <value><string>{}</string></value>\n", spaces, Self::escape_xml(value)));
        xml.push_str(&format!("{}  </member>\n", spaces));
    }

    fn add_int_member_indent(xml: &mut String, name: &str, value: i32, indent: usize) {
        let spaces = " ".repeat(indent);
        xml.push_str(&format!("{}  <member>\n", spaces));
        xml.push_str(&format!("{}    <name>{}</name>\n", spaces, name));
        xml.push_str(&format!("{}    <value><i4>{}</i4></value>\n", spaces, value));
        xml.push_str(&format!("{}  </member>\n", spaces));
    }

    fn add_login_flags_array(xml: &mut String, response: &LoginResponse) {
        xml.push_str("          <member>\n");
        xml.push_str("            <name>login-flags</name>\n");
        xml.push_str("            <value>\n");
        xml.push_str("              <array>\n");
        xml.push_str("                <data>\n");
        xml.push_str("                  <value>\n");
        xml.push_str("                    <struct>\n");

        Self::add_string_member_indent(xml, "daylight_savings", &response.dst, 22);
        Self::add_string_member_indent(xml, "stipend_since_login", &response.stipend_since_login, 22);
        Self::add_string_member_indent(xml, "gendered", &response.gendered, 22);
        Self::add_string_member_indent(xml, "ever_logged_in", &response.ever_logged_in, 22);

        xml.push_str("                    </struct>\n");
        xml.push_str("                  </value>\n");
        xml.push_str("                </data>\n");
        xml.push_str("              </array>\n");
        xml.push_str("            </value>\n");
        xml.push_str("          </member>\n");
    }

    fn add_global_textures_array(xml: &mut String, response: &LoginResponse) {
        xml.push_str("          <member>\n");
        xml.push_str("            <name>global-textures</name>\n");
        xml.push_str("            <value>\n");
        xml.push_str("              <array>\n");
        xml.push_str("                <data>\n");
        xml.push_str("                  <value>\n");
        xml.push_str("                    <struct>\n");

        Self::add_string_member_indent(xml, "sun_texture_id", &response.sun_texture_id, 22);
        Self::add_string_member_indent(xml, "cloud_texture_id", &response.cloud_texture_id, 22);
        Self::add_string_member_indent(xml, "moon_texture_id", &response.moon_texture_id, 22);

        xml.push_str("                    </struct>\n");
        xml.push_str("                  </value>\n");
        xml.push_str("                </data>\n");
        xml.push_str("              </array>\n");
        xml.push_str("            </value>\n");
        xml.push_str("          </member>\n");
    }

    fn add_ui_config_array(xml: &mut String, response: &LoginResponse) {
        xml.push_str("          <member>\n");
        xml.push_str("            <name>ui-config</name>\n");
        xml.push_str("            <value>\n");
        xml.push_str("              <array>\n");
        xml.push_str("                <data>\n");
        xml.push_str("                  <value>\n");
        xml.push_str("                    <struct>\n");

        Self::add_string_member_indent(xml, "allow_first_life", &response.allow_first_life, 22);

        xml.push_str("                    </struct>\n");
        xml.push_str("                  </value>\n");
        xml.push_str("                </data>\n");
        xml.push_str("              </array>\n");
        xml.push_str("            </value>\n");
        xml.push_str("          </member>\n");
    }

    fn add_empty_array(xml: &mut String, name: &str) {
        xml.push_str(&format!("          <member>\n"));
        xml.push_str(&format!("            <name>{}</name>\n", name));
        xml.push_str("            <value>\n");
        xml.push_str("              <array>\n");
        xml.push_str("                <data />\n");
        xml.push_str("              </array>\n");
        xml.push_str("            </value>\n");
        xml.push_str("          </member>\n");
    }

    // Phase 68.7: Add initial-outfit with default outfit entry
    // This sets isOutfitChosen() = true in viewer, allowing transition to 3D view
    fn add_initial_outfit_array(xml: &mut String) {
        xml.push_str("          <member>\n");
        xml.push_str("            <name>initial-outfit</name>\n");
        xml.push_str("            <value>\n");
        xml.push_str("              <array>\n");
        xml.push_str("                <data>\n");
        xml.push_str("                  <value>\n");
        xml.push_str("                    <struct>\n");
        xml.push_str("                      <member>\n");
        xml.push_str("                        <name>folder_name</name>\n");
        xml.push_str("                        <value><string>Default Outfit</string></value>\n");
        xml.push_str("                      </member>\n");
        xml.push_str("                      <member>\n");
        xml.push_str("                        <name>gender</name>\n");
        xml.push_str("                        <value><string>female</string></value>\n");
        xml.push_str("                      </member>\n");
        xml.push_str("                    </struct>\n");
        xml.push_str("                  </value>\n");
        xml.push_str("                </data>\n");
        xml.push_str("              </array>\n");
        xml.push_str("            </value>\n");
        xml.push_str("          </member>\n");
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
         .replace('<', "&lt;")
         .replace('>', "&gt;")
         .replace('"', "&quot;")
         .replace('\'', "&apos;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_success_response_generation() {
        let session = LoginSession::new(
            Uuid::new_v4(),
            "John".to_string(),
            "Doe".to_string(),
            "127.0.0.1".to_string(),
            9000,
        );
        
        let inventory = crate::inventory::LoginInventoryResponse::empty();
        let seed_url = "http://127.0.0.1:9000/cap/test";
        let response = LoginResponse::success(&session, "http://127.0.0.1:9000", &inventory, seed_url);
        let xml = XmlRpcResponseGenerator::generate_login_response(&response).unwrap();

        assert!(xml.contains("<boolean>1</boolean>"));
        assert!(xml.contains("John"));
        assert!(xml.contains("Doe"));
        assert!(xml.contains("127.0.0.1"));
        assert!(xml.contains(&session.circuit_code.to_string()));
    }
    
    #[test]
    fn test_failure_response_generation() {
        let response = LoginResponse::failure("key", "Invalid username or password");
        let xml = XmlRpcResponseGenerator::generate_login_response(&response).unwrap();
        
        assert!(xml.contains("<string>false</string>"));
        assert!(xml.contains("Invalid username or password"));
        assert!(xml.contains("<name>reason</name>"));
    }
    
    #[test]
    fn test_xml_escaping() {
        assert_eq!(XmlRpcResponseGenerator::escape_xml("test&data"), "test&amp;data");
        assert_eq!(XmlRpcResponseGenerator::escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(XmlRpcResponseGenerator::escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }
}