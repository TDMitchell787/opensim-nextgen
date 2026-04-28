use axum::{
    extract::ws::WebSocketUpgrade,
    response::Response,
    routing::{get, post},
    Router,
};
use tracing::info;

use super::{event_queue::handle_event_queue_get, handlers::*, CapsHandlerState};

pub fn create_caps_router() -> Router<CapsHandlerState> {
    Router::new()
        // WebSocket support
        .route("/ws", get(ws_handler))
        // Seed capability - returns all capabilities for a session
        .route("/cap/:session_id", get(handle_seed_capability))
        .route("/cap/:session_id", post(handle_seed_capability_post))
        // EventQueueGet - OpenSim-compatible /CE/{uuid} format for viewer communication (long-polling)
        // Viewers use POST for EventQueue polling
        .route("/CE/:eqg_uuid", post(handle_event_queue_get))
        // Inventory capabilities
        .route(
            "/cap/:session_id/FetchInventory2",
            post(handle_fetch_inventory2),
        )
        .route(
            "/cap/:session_id/FetchInventoryDescendents2",
            post(handle_fetch_inventory_descendents2),
        )
        // Library capabilities - OpenSim Library folder access
        .route(
            "/cap/:session_id/FetchLibDescendents2",
            post(handle_fetch_lib_descendents2),
        )
        .route("/cap/:session_id/FetchLib2", post(handle_fetch_lib2))
        // Avatar capabilities
        .route(
            "/cap/:session_id/UpdateAvatarAppearance",
            post(handle_update_avatar_appearance),
        )
        // Asset capabilities
        .route("/cap/:session_id/GetTexture", get(handle_get_texture))
        // Statistics and information
        .route("/cap/:session_id/ViewerStats", post(handle_viewer_stats))
        .route(
            "/cap/:session_id/UpdateAgentInformation",
            post(handle_update_agent_information),
        )
        // Specific capability handlers
        .route(
            "/cap/:session_id/UpdateAgentLanguage",
            post(handle_update_agent_language),
        )
        .route(
            "/cap/:session_id/AgentPreferences",
            get(handle_agent_preferences_get),
        )
        .route(
            "/cap/:session_id/AgentPreferences",
            post(handle_agent_preferences_post),
        )
        .route(
            "/cap/:session_id/HomeLocation",
            get(handle_home_location_get),
        )
        .route(
            "/cap/:session_id/HomeLocation",
            post(handle_home_location_post),
        )
        .route(
            "/cap/:session_id/GetDisplayNames",
            get(handle_get_display_names),
        )
        .route(
            "/cap/:session_id/GetDisplayNames/",
            get(handle_get_display_names),
        )
        .route(
            "/cap/:session_id/SetDisplayName",
            post(handle_set_display_name),
        )
        .route(
            "/cap/:session_id/CreateInventoryCategory",
            post(handle_create_inventory_category),
        )
        .route(
            "/cap/:session_id/NewFileAgentInventory",
            post(handle_new_file_agent_inventory),
        )
        .route(
            "/cap/:session_id/NewFileAgentInventoryVariablePrice",
            post(handle_new_file_agent_inventory),
        )
        .route(
            "/cap/:session_id/UpdateNotecardAgentInventory",
            post(handle_update_notecard_agent_inventory),
        )
        .route(
            "/cap/:session_id/UpdateScriptAgentInventory",
            post(handle_update_script_agent_inventory),
        )
        .route(
            "/cap/:session_id/UpdateScriptTask",
            post(handle_update_script_task_inventory),
        )
        .route(
            "/cap/:session_id/UpdateNotecardTaskInventory",
            post(handle_update_notecard_task_inventory),
        )
        .route(
            "/cap/:session_id/ScriptTaskUpload/:uploader_id",
            post(handle_script_task_upload),
        )
        .route(
            "/cap/:session_id/ParcelPropertiesUpdate",
            post(handle_parcel_properties_update),
        )
        .route("/cap/:session_id/MapLayer", get(handle_map_layer))
        .route(
            "/cap/:session_id/SimulatorFeatures",
            get(handle_simulator_features),
        )
        .route(
            "/cap/:session_id/SimulatorFeatures",
            post(handle_simulator_features_post),
        )
        // Environment capabilities - CRITICAL for login completion
        .route(
            "/cap/:session_id/EnvironmentSettings",
            get(handle_environment_settings_get),
        )
        .route(
            "/cap/:session_id/EnvironmentSettings",
            post(handle_environment_settings_post),
        )
        .route(
            "/cap/:session_id/ExtEnvironment",
            get(handle_ext_environment_get),
        )
        .route(
            "/cap/:session_id/ExtEnvironment",
            post(handle_ext_environment_post),
        )
        // NewFileAgentInventory upload handler (two-stage)
        .route(
            "/cap/:session_id/NewFileAgentInventoryUpload/:uploader_id",
            post(handle_new_file_agent_inventory_upload),
        )
        // Notecard upload handler (two-stage)
        .route(
            "/cap/:session_id/NotecardUpload/:uploader_id",
            post(handle_notecard_upload),
        )
        // Script upload handler (two-stage)
        .route(
            "/cap/:session_id/ScriptUpload/:uploader_id",
            post(handle_script_upload),
        )
        // Baked texture upload - CRITICAL for avatar appearance
        .route(
            "/cap/:session_id/UploadBakedTexture",
            post(handle_upload_baked_texture),
        )
        .route(
            "/cap/:session_id/BakedTextureUpload/:uploader_id",
            post(handle_baked_texture_data),
        )
        // Mesh and asset capabilities (with trailing-slash variants for Firestorm compatibility)
        .route("/cap/:session_id/GetTexture/", get(handle_get_texture))
        .route("/cap/:session_id/GetMesh", get(handle_get_mesh))
        .route("/cap/:session_id/GetMesh/", get(handle_get_mesh))
        .route("/cap/:session_id/GetMesh2", get(handle_get_mesh))
        .route("/cap/:session_id/GetMesh2/", get(handle_get_mesh))
        .route("/cap/:session_id/ViewerAsset", get(handle_viewer_asset))
        .route("/cap/:session_id/ViewerAsset/", get(handle_viewer_asset))
        .route(
            "/cap/:session_id/MeshUploadFlag",
            get(handle_mesh_upload_flag),
        )
        // Object cost and physics data capabilities (Phase 120)
        .route(
            "/cap/:session_id/GetObjectCost",
            post(handle_get_object_cost),
        )
        .route(
            "/cap/:session_id/GetObjectPhysicsData",
            post(handle_get_object_physics_data),
        )
        .route(
            "/cap/:session_id/ResourceCostSelected",
            post(handle_resource_cost_selected),
        )
        // Voice capabilities
        .route(
            "/cap/:session_id/ProvisionVoiceAccountRequest",
            post(handle_provision_voice_account),
        )
        .route(
            "/cap/:session_id/ParcelVoiceInfoRequest",
            post(handle_parcel_voice_info),
        )
        // Profile capabilities
        .route("/cap/:session_id/AgentProfile", get(handle_agent_profile))
        .route(
            "/cap/:session_id/AgentProfile",
            post(handle_agent_profile_update),
        )
        // Materials capabilities (PBR)
        .route(
            "/cap/:session_id/RenderMaterials",
            get(handle_render_materials_get),
        )
        .route(
            "/cap/:session_id/RenderMaterials",
            post(handle_render_materials_post),
        )
        .route(
            "/cap/:session_id/ModifyMaterialParams",
            post(handle_modify_material_params),
        )
        // Object media
        .route("/cap/:session_id/ObjectMedia", get(handle_object_media_get))
        .route(
            "/cap/:session_id/ObjectMedia",
            post(handle_object_media_post),
        )
        .route(
            "/cap/:session_id/ObjectMediaNavigate",
            post(handle_object_media_navigate),
        )
        // Search and info
        .route(
            "/cap/:session_id/SearchStatRequest",
            get(handle_search_stat_request),
        )
        .route("/cap/:session_id/LandResources", get(handle_land_resources))
        .route(
            "/cap/:session_id/ScriptResourceSummary",
            post(handle_script_resource_summary),
        )
        .route(
            "/cap/:session_id/ScriptResourceDetails",
            post(handle_script_resource_details),
        )
        .route(
            "/cap/:session_id/AvatarPickerSearch",
            get(handle_avatar_picker_search),
        )
        .route(
            "/cap/:session_id/DispatchRegionInfo",
            post(handle_dispatch_region_info),
        )
        .route(
            "/cap/:session_id/ProductInfoRequest",
            get(handle_product_info_request),
        )
        .route(
            "/cap/:session_id/ServerReleaseNotes",
            get(handle_server_release_notes),
        )
        // Inventory operations
        .route(
            "/cap/:session_id/CopyInventoryFromNotecard",
            post(handle_copy_inventory_from_notecard),
        )
        .route(
            "/cap/:session_id/UpdateGestureAgentInventory",
            post(handle_update_gesture_agent_inventory),
        )
        .route(
            "/cap/:session_id/UpdateGestureTaskInventory",
            post(handle_update_gesture_task_inventory),
        )
        // Script syntax
        .route("/cap/:session_id/LSLSyntax", get(handle_lsl_syntax))
        // FreeSWITCH callback routes
        .route(
            "/fsapi/viv_get_prelogin.php",
            post(handle_freeswitch_prelogin),
        )
        .route("/fsapi/viv_signin.php", post(handle_freeswitch_signin))
        .route("/fsapi/viv_buddy.php", post(handle_freeswitch_buddy))
        .route("/fsapi/viv_watcher.php", post(handle_freeswitch_watcher))
        // Region handshake endpoint
        .route("/region_handshake", get(region_handshake))
}

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(|_socket| async {
        info!("WebSocket client connected to CAPS server");
        // Basic WebSocket connection without full features for now
    })
}

async fn region_handshake() -> axum::response::Json<serde_json::Value> {
    use serde_json::json;

    info!("Region handshake request");
    axum::response::Json(json!({
        "region_name": "Default Region",
        "region_handle": (256000_u64 << 32) | 256000_u64,
        "sim_access": "M",
        "agent_movement_complete": true
    }))
}
