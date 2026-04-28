use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{any, delete, get, head, post},
    Json, Router,
};

use super::agentprefs_handler::handle_agentprefs;
use super::asset_handler::{
    handle_asset_delete, handle_asset_get, handle_asset_metadata_get, handle_asset_post,
    handle_assets_exist,
};
use super::auth_handler::handle_auth;
use super::authorization_handler::handle_authorization;
use super::avatar_handler::handle_avatar;
use super::bakes_handler::{handle_bakes_get, handle_bakes_post};
use super::estate_handler::{
    handle_estate_get, handle_estate_post, handle_estate_regions, handle_estates_get,
};
use super::freeswitch_handler::handle_freeswitch;
use super::friends_handler::handle_friends;
use super::gatekeeper_handler::{
    handle_agent_simulation, handle_agent_update, handle_foreign_agent,
    handle_foreign_agent_standalone, handle_gatekeeper, handle_gatekeeper_standalone,
    handle_object_simulation,
};
use super::grid_handler::handle_grid;
use super::grid_info_handler::{handle_grid_info_json, handle_grid_info_xml, handle_grid_stats};
use super::griduser_handler::handle_griduser;
use super::helo_handler::{handle_helo_get, handle_helo_head};
use super::hg_inventory_handler::handle_hg_inventory;
use super::hgfriends_handler::handle_hgfriends;
use super::inventory_handler::handle_inventory;
use super::land_handler::handle_land;
use super::map_handler::{handle_map_get, handle_map_post, handle_removemap_post};
use super::mutelist_handler::handle_mutelist;
use super::neighbour_handler::{handle_neighbour, handle_neighbour_trailing};
use super::offlineim_handler::handle_offlineim;
use super::presence_handler::handle_presence;
use super::profiles_handler::handle_profiles;
use super::uas_handler::{
    handle_home_agent, handle_home_agent_standalone, handle_useragent, handle_useragent_standalone,
};
use super::user_account_handler::handle_user_account;
use super::{GatekeeperState, RobustState, UasState};

pub fn create_robust_router(state: RobustState) -> Router {
    Router::new()
        .route("/grid", post(handle_grid))
        .route("/accounts", post(handle_user_account))
        .route("/auth", post(handle_auth))
        .route("/assets", post(handle_asset_post.clone()))
        .route("/assets/", post(handle_asset_post))
        .route(
            "/assets/:id",
            get(handle_asset_get.clone()).delete(handle_asset_delete),
        )
        .route("/assets/:id/data", get(handle_asset_get))
        .route("/assets/:id/metadata", get(handle_asset_metadata_get))
        .route("/inventory", post(handle_inventory.clone()))
        .route("/xinventory", post(handle_inventory))
        .route("/presence", post(handle_presence))
        .route("/avatar", post(handle_avatar))
        .route("/gatekeeper", post(handle_gatekeeper))
        .route(
            "/foreignagent/:agent_id",
            post(handle_foreign_agent.clone()),
        )
        .route("/foreignagent/:agent_id/", post(handle_foreign_agent))
        .route("/useragent", post(handle_useragent))
        .route("/homeagent/:agent_id", post(handle_home_agent.clone()))
        .route("/homeagent/:agent_id/", post(handle_home_agent))
        .route("/helo", get(handle_helo_get))
        .route("/helo", head(handle_helo_head))
        .route("/helo/", get(handle_helo_get))
        .route("/helo/", head(handle_helo_head))
        .route("/hgfriends", post(handle_hgfriends))
        .route("/hg/xinventory", post(handle_hg_inventory))
        .route("/griduser", post(handle_griduser))
        .route("/agentprefs", post(handle_agentprefs))
        .route("/bakes/:id", get(handle_bakes_get).post(handle_bakes_post))
        .route("/mutelist", post(handle_mutelist))
        .route("/estates", get(handle_estates_get))
        .route(
            "/estates/estate",
            get(handle_estate_get).post(handle_estate_post),
        )
        .route("/estates/estate/", post(handle_estate_post))
        .route("/estates/regions", get(handle_estate_regions))
        .route("/map/*path", get(handle_map_get))
        .route("/map", post(handle_map_post))
        .route("/removemap", post(handle_removemap_post))
        .route("/get_assets_exist", post(handle_assets_exist))
        // Phase 138 P3: New services
        .route("/authorization", post(handle_authorization))
        .route("/friends", post(handle_friends))
        .route("/land", post(handle_land))
        .route("/offlineim", post(handle_offlineim))
        .route("/region/:region_id", post(handle_neighbour))
        .route("/region/:region_id/", post(handle_neighbour_trailing))
        .route("/user_profile_rpc", post(handle_profiles))
        .route("/fsapi/freeswitch-config", post(handle_freeswitch.clone()))
        .route("/fsapi/region-config", post(handle_freeswitch))
        // Phase 141: Grid info endpoints (faithful to C# GridInfoHandlers.cs)
        .route("/get_grid_info", get(handle_grid_info_xml))
        .route("/json_grid_info", get(handle_grid_info_json))
        .route("/get_grid_stats", get(handle_grid_stats))
        .route("/ready", get(handle_robust_ready))
        .with_state(state)
}

pub fn create_uas_router(state: UasState) -> Router {
    Router::new()
        .route("/useragent", post(handle_useragent_standalone))
        .route(
            "/homeagent/:agent_id",
            post(handle_home_agent_standalone.clone()),
        )
        .route("/homeagent/:agent_id/", post(handle_home_agent_standalone))
        .with_state(state)
}

pub fn create_gatekeeper_router(state: GatekeeperState) -> Router {
    Router::new()
        .route("/gatekeeper", post(handle_gatekeeper_standalone))
        .route(
            "/foreignagent/:agent_id",
            post(handle_foreign_agent_standalone.clone()),
        )
        .route(
            "/foreignagent/:agent_id/",
            post(handle_foreign_agent_standalone),
        )
        .route("/agent/:agent_id/:region_id", any(handle_agent_simulation))
        .route("/agent/:agent_id/:region_id/", any(handle_agent_simulation))
        .route("/agent/:agent_id", any(handle_agent_update))
        .route("/agent/:agent_id/", any(handle_agent_update))
        .route(
            "/object/:object_id/:region_id",
            any(handle_object_simulation),
        )
        .route(
            "/object/:object_id/:region_id/",
            any(handle_object_simulation),
        )
        .with_state(state)
}

async fn handle_robust_ready(State(_state): State<RobustState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ready": true,
            "service": "robust"
        })),
    )
}
