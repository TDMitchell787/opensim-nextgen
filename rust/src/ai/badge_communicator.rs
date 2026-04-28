use tracing::info;
use uuid::Uuid;

pub struct BadgeCommunicator {
    pub badge_texture_uuid: Uuid,
}

impl BadgeCommunicator {
    pub fn new() -> Self {
        Self {
            badge_texture_uuid: Uuid::parse_str("c228d1cf-4b5d-4ba8-84f4-899a0796aa97")
                .unwrap_or(Uuid::nil()),
        }
    }

    pub fn with_texture(texture_uuid: Uuid) -> Self {
        Self {
            badge_texture_uuid: texture_uuid,
        }
    }

    pub async fn import_badge_texture(
        pool: &sqlx::PgPool,
        image_path: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let image_data = tokio::fs::read(image_path).await?;

        let asset_id = Uuid::new_v4();
        let now = chrono::Utc::now().timestamp() as i32;

        sqlx::query(
            "INSERT INTO assets (id, name, description, assettype, local, temporary, create_time, access_time, asset_flags, creatorid, data) \
             VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             ON CONFLICT (id) DO NOTHING"
        )
        .bind(asset_id)
        .bind("Galadriel Communicator Badge")
        .bind("Voyager-era communicator badge texture")
        .bind(0i32) // assettype 0 = Texture
        .bind(true)
        .bind(false)
        .bind(now)
        .bind(now)
        .bind(0i32)
        .bind(Uuid::nil().to_string())
        .bind(&image_data)
        .execute(pool)
        .await?;

        info!(
            "[BADGE] Imported badge texture {} from {}",
            asset_id, image_path
        );
        Ok(asset_id)
    }

    pub fn badge_prim_scale() -> [f32; 3] {
        [0.06, 0.06, 0.01]
    }

    pub fn badge_name() -> &'static str {
        "Galadriel Communicator"
    }

    pub fn badge_channel() -> i32 {
        super::galadriel::GALADRIEL_CHANNEL
    }
}

pub fn badge_communicator_script() -> String {
    format!(
        r#"integer GALADRIEL_CH = {};
integer g_active = TRUE;
default {{
    state_entry() {{
        llListen(GALADRIEL_CH, "", llGetOwner(), "");
        llSetText("Listen", <0.3, 1.0, 0.3>, 0.5);
    }}
    touch_start(integer n) {{
        if (llDetectedKey(0) != llGetOwner()) return;
        g_active = !g_active;
        if (g_active) {{
            llSetText("Listen", <0.3, 1.0, 0.3>, 0.5);
            llOwnerSay("Galadriel listening.");
            llRegionSay(GALADRIEL_CH, "/mode listen");
        }} else {{
            llSetText("Quiet", <1.0, 0.3, 0.3>, 0.5);
            llOwnerSay("Galadriel quiet mode.");
            llRegionSay(GALADRIEL_CH, "/mode quiet");
        }}
    }}
    listen(integer chan, string name, key id, string msg) {{
        if (chan == GALADRIEL_CH && id == llGetOwner() && g_active) {{
            llRegionSay(GALADRIEL_CH, msg);
        }}
    }}
}}"#,
        super::galadriel::GALADRIEL_CHANNEL
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_communicator_defaults() {
        let badge = BadgeCommunicator::new();
        assert_eq!(
            badge.badge_texture_uuid.to_string(),
            "c228d1cf-4b5d-4ba8-84f4-899a0796aa97"
        );
    }

    #[test]
    fn test_badge_with_custom_texture() {
        let custom_id = Uuid::new_v4();
        let badge = BadgeCommunicator::with_texture(custom_id);
        assert_eq!(badge.badge_texture_uuid, custom_id);
    }

    #[test]
    fn test_badge_prim_scale() {
        let scale = BadgeCommunicator::badge_prim_scale();
        assert!((scale[0] - 0.06).abs() < 0.001);
        assert!((scale[1] - 0.06).abs() < 0.001);
        assert!((scale[2] - 0.01).abs() < 0.001);
    }

    #[test]
    fn test_badge_name() {
        assert_eq!(BadgeCommunicator::badge_name(), "Galadriel Communicator");
    }

    #[test]
    fn test_badge_channel() {
        assert_eq!(BadgeCommunicator::badge_channel(), -15400);
    }

    #[test]
    fn test_badge_script_contains_channel() {
        let script = badge_communicator_script();
        assert!(script.contains("-15400"));
        assert!(script.contains("touch_start"));
        assert!(script.contains("llOwnerSay"));
        assert!(script.contains("llRegionSay"));
        assert!(script.contains("g_active"));
        assert!(script.contains("/mode listen"));
        assert!(script.contains("/mode quiet"));
        assert!(script.contains("llSetText"));
    }
}
