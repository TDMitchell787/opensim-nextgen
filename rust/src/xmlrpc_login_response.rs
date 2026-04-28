use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct LoginResponseData {
    pub agent_id: String,
    pub session_id: Uuid,
    pub secure_session_id: Uuid,
    pub circuit_code: u32,
    pub first_name: String,
    pub last_name: String,
    pub inventory_root_id: Uuid,
    pub caps_session_id: String,
}

pub fn generate_complete_xmlrpc_response(data: LoginResponseData) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i32;

    // Generate deterministic root folder ID using the SAME logic as FetchInventoryDescendents2
    let root_folder_key = format!("{}:{}", data.agent_id, "my inventory");
    let inventory_root_id = Uuid::new_v5(&Uuid::NAMESPACE_OID, root_folder_key.as_bytes());

    // Generate all required UUIDs for inventory folders (using agent_id for determinism)
    let inventory_folders = generate_inventory_skeleton(&data.agent_id);
    let library_folders = generate_library_skeleton();

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?><methodResponse><params><param><value><struct><member><name>login</name><value><string>true</string></value></member><member><name>agent_id</name><value><string>{}</string></value></member><member><name>session_id</name><value><string>{}</string></value></member><member><name>secure_session_id</name><value><string>{}</string></value></member><member><name>first_name</name><value><string>{}</string></value></member><member><name>last_name</name><value><string>{}</string></value></member><member><name>circuit_code</name><value><i4>{}</i4></value></member><member><name>seed_capability</name><value><string>http://127.0.0.1:9000/cap/{}</string></value></member><member><name>sim_ip</name><value><string>127.0.0.1</string></value></member><member><name>sim_port</name><value><i4>9000</i4></value></member><member><name>region_x</name><value><i4>256000</i4></value></member><member><name>region_y</name><value><i4>256000</i4></value></member><member><name>region_handle</name><value><array><data><value><i4>1000</i4></value><value><i4>1000</i4></value></data></array></value></member><member><name>home</name><value><struct><member><name>region_handle</name><value><array><data><value><i4>1000</i4></value><value><i4>1000</i4></value></data></array></value></member><member><name>position</name><value><array><data><value><double>128.0</double></value><value><double>128.0</double></value><value><double>25.0</double></value></data></array></value></member><member><name>look_at</name><value><array><data><value><double>1.0</double></value><value><double>0.0</double></value><value><double>0.0</double></value></data></array></value></member></struct></value></member><member><name>inventory-root</name><value><array><data><value><struct><member><name>folder_id</name><value><string>{}</string></value></member></struct></value></data></array></value></member><member><name>inventory-skeleton</name><value><array><data>{}</data></array></value></member><member><name>buddy-list</name><value><array><data /></array></value></member><member><name>look_at</name><value><array><data><value><double>1.0</double></value><value><double>0.0</double></value><value><double>0.0</double></value></data></array></value></member><member><name>agent_access</name><value><string>M</string></value></member><member><name>agent_access_max</name><value><string>A</string></value></member><member><name>start_location</name><value><string>safe</string></value></member><member><name>seconds_since_epoch</name><value><i4>{}</i4></value></member><member><name>message</name><value><string>Welcome to OpenSim!</string></value></member></struct></value></param></params></methodResponse>"#,
        data.agent_id,
        data.session_id,
        data.secure_session_id,
        data.first_name,
        data.last_name,
        data.circuit_code,
        data.caps_session_id,
        inventory_root_id,
        inventory_folders,
        timestamp
    )
}

fn generate_inventory_skeleton(agent_id: &str) -> String {
    // Generate deterministic folder IDs using the SAME logic as FetchInventoryDescendents2
    // Format: Uuid::new_v5(NAMESPACE_OID, "agent_id:folder_name.to_lowercase()")
    fn folder_id(owner_id: &str, name: &str) -> String {
        let folder_key = format!("{}:{}", owner_id, name.to_lowercase());
        Uuid::new_v5(&Uuid::NAMESPACE_OID, folder_key.as_bytes()).to_string()
    }

    // Root folder ID is derived from "my inventory"
    let root_id = folder_id(agent_id, "my inventory");

    // Folders: (name, parent_id, type_default, version)
    // parent_id is "00000000-0000-0000-0000-000000000000" for root, root_id for all others
    let folders = vec![
        (
            "My Inventory",
            "00000000-0000-0000-0000-000000000000",
            8,
            19,
        ),
        ("Animations", root_id.as_str(), 20, 1),
        ("Body Parts", root_id.as_str(), 13, 5),
        ("Calling Cards", root_id.as_str(), 2, 2),
        ("Clothing", root_id.as_str(), 5, 3),
        ("Current Outfit", root_id.as_str(), 46, 7),
        ("Favorites", root_id.as_str(), 23, 1),
        ("Gestures", root_id.as_str(), 21, 1),
        ("Landmarks", root_id.as_str(), 3, 1),
        ("Lost And Found", root_id.as_str(), 16, 1),
        ("Notecards", root_id.as_str(), 7, 1),
        ("Objects", root_id.as_str(), 6, 1),
        ("Photo Album", root_id.as_str(), 15, 1),
        ("Scripts", root_id.as_str(), 10, 1),
        ("Sounds", root_id.as_str(), 1, 1),
        ("Textures", root_id.as_str(), 0, 1),
        ("Trash", root_id.as_str(), 14, 1),
        ("Settings", root_id.as_str(), 56, 1),
        ("Materials", root_id.as_str(), 57, 1),
    ];

    let mut result = String::new();
    for (name, parent_id, type_default, version) in folders.iter() {
        // Use deterministic folder ID based on agent_id:folder_name
        let fid = folder_id(agent_id, name);
        result.push_str(&format!(
            r#"<value><struct><member><name>version</name><value><i4>{}</i4></value></member><member><name>parent_id</name><value><string>{}</string></value></member><member><name>folder_id</name><value><string>{}</string></value></member><member><name>name</name><value><string>{}</string></value></member><member><name>type_default</name><value><i4>{}</i4></value></member></struct></value>"#,
            version, parent_id, fid, name, type_default
        ));
    }
    result
}

fn generate_library_skeleton() -> String {
    let folders = vec![
        (
            "OpenSim Library",
            "00000000-0000-0000-0000-000000000000",
            "00000112-000f-0000-0000-000100bba000",
            8,
            429,
        ),
        (
            "Animations Library",
            "00000112-000f-0000-0000-000100bba000",
            "f0908f10-b9bf-11dc-95ff-0800200c9a66",
            20,
            1,
        ),
        (
            "BodyParts Library",
            "00000112-000f-0000-0000-000100bba000",
            "d499e5e0-b9bf-11dc-95ff-0800200c9a66",
            13,
            1,
        ),
        (
            "Clothing Library",
            "00000112-000f-0000-0000-000100bba000",
            "b75056e0-b9bf-11dc-95ff-0800200c9a66",
            5,
            1,
        ),
        (
            "Gestures Library",
            "00000112-000f-0000-0000-000100bba000",
            "8e1e3a30-b9bf-11dc-95ff-0800200c9a66",
            21,
            1,
        ),
        (
            "Landmarks Library",
            "00000112-000f-0000-0000-000100bba000",
            "6bcd48e0-b9bf-11dc-95ff-0800200c9a66",
            3,
            1,
        ),
        (
            "Notecards Library",
            "00000112-000f-0000-0000-000100bba000",
            "33cbd240-b9bf-11dc-95ff-0800200c9a66",
            7,
            1,
        ),
        (
            "Objects Library",
            "00000112-000f-0000-0000-000100bba000",
            "1576c6b0-b9bf-11dc-95ff-0800200c9a66",
            6,
            1,
        ),
        (
            "Photos Library",
            "00000112-000f-0000-0000-000100bba000",
            "cf7e2db0-b9be-11dc-95ff-0800200c9a66",
            15,
            1,
        ),
        (
            "Scripts Library",
            "00000112-000f-0000-0000-000100bba000",
            "30000112-000f-0000-0000-000100bba002",
            10,
            1,
        ),
        (
            "Sounds Library",
            "00000112-000f-0000-0000-000100bba000",
            "c1284980-b9be-11dc-95ff-0800200c9a66",
            1,
            1,
        ),
        (
            "Texture Library",
            "00000112-000f-0000-0000-000100bba000",
            "00000112-000f-0000-0000-000100bba001",
            0,
            1,
        ),
        (
            "Settings Library",
            "00000112-000f-0000-0000-000100bba000",
            "00000112-000f-0000-0000-000100bba025",
            56,
            4,
        ),
    ];

    let mut result = String::new();
    for (name, parent_id, folder_id, type_default, version) in folders {
        result.push_str(&format!(
            r#"<value><struct><member><name>version</name><value><i4>{}</i4></value></member><member><name>parent_id</name><value><string>{}</string></value></member><member><name>folder_id</name><value><string>{}</string></value></member><member><name>name</name><value><string>{}</string></value></member><member><name>type_default</name><value><i4>{}</i4></value></member></struct></value>"#,
            version, parent_id, folder_id, name, type_default
        ));
    }
    result
}
