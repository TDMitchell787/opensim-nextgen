use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, warn};

use crate::database::DatabaseConnection;
use crate::services::traits::{EstateBan, EstateServiceTrait, EstateSettings};

pub struct LocalEstateService {
    db: Arc<DatabaseConnection>,
}

impl LocalEstateService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn bool_from_int(v: i32) -> bool {
        v != 0
    }

    async fn load_estate_extras(
        &self,
        estate_id: i32,
        settings: &mut EstateSettings,
    ) -> Result<()> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let managers = sqlx::query("SELECT uuid FROM estate_managers WHERE estateid = $1")
            .bind(estate_id)
            .fetch_all(pool)
            .await?;
        for row in &managers {
            use sqlx::Row;
            if let Ok(uuid) = row.try_get::<String, _>("uuid") {
                settings.estate_managers.push(uuid.trim().to_string());
            }
        }

        let users = sqlx::query("SELECT uuid FROM estate_users WHERE estateid = $1")
            .bind(estate_id)
            .fetch_all(pool)
            .await?;
        for row in &users {
            use sqlx::Row;
            if let Ok(uuid) = row.try_get::<String, _>("uuid") {
                settings.estate_users.push(uuid.trim().to_string());
            }
        }

        let groups = sqlx::query("SELECT uuid FROM estate_groups WHERE estateid = $1")
            .bind(estate_id)
            .fetch_all(pool)
            .await?;
        for row in &groups {
            use sqlx::Row;
            if let Ok(uuid) = row.try_get::<String, _>("uuid") {
                settings.estate_groups.push(uuid.trim().to_string());
            }
        }

        let bans = sqlx::query(
            "SELECT banneduuid, bannedip, bannedipmask, bantime FROM estateban WHERE estateid = $1",
        )
        .bind(estate_id)
        .fetch_all(pool)
        .await?;
        for row in &bans {
            use sqlx::Row;
            settings.estate_bans.push(EstateBan {
                banned_user_id: row.try_get::<String, _>("banneduuid").unwrap_or_default(),
                banned_ip: row.try_get::<String, _>("bannedip").unwrap_or_default(),
                banned_ip_mask: row.try_get::<String, _>("bannedipmask").unwrap_or_default(),
                ban_time: row.try_get::<i32, _>("bantime").unwrap_or(0),
            });
        }

        Ok(())
    }

    fn row_to_settings(row: &sqlx::postgres::PgRow) -> EstateSettings {
        use sqlx::Row;
        EstateSettings {
            estate_id: row.try_get::<i32, _>("estateid").unwrap_or(1),
            estate_name: row
                .try_get::<String, _>("estatename")
                .unwrap_or_else(|_| "My Estate".to_string()),
            estate_owner: row
                .try_get::<String, _>("estateowner")
                .unwrap_or_else(|_| "00000000-0000-0000-0000-000000000000".to_string()),
            parent_estate_id: row.try_get::<i32, _>("parentestateid").unwrap_or(0),
            abuse_email_to_estate_owner: Self::bool_from_int(
                row.try_get("abuseemailtoestateowner").unwrap_or(1),
            ),
            deny_anonymous: Self::bool_from_int(row.try_get("denyanonymous").unwrap_or(0)),
            reset_home_on_teleport: Self::bool_from_int(
                row.try_get("resethomeonteleport").unwrap_or(0),
            ),
            fixed_sun: Self::bool_from_int(row.try_get("fixedsun").unwrap_or(0)),
            deny_transacted: Self::bool_from_int(row.try_get("denytransacted").unwrap_or(0)),
            block_dwell: Self::bool_from_int(row.try_get("blockdwell").unwrap_or(0)),
            deny_identified: Self::bool_from_int(row.try_get("denyidentified").unwrap_or(0)),
            allow_voice: Self::bool_from_int(row.try_get("allowvoice").unwrap_or(1)),
            use_global_time: Self::bool_from_int(row.try_get("useglobaltime").unwrap_or(1)),
            price_per_meter: row.try_get("pricepermeter").unwrap_or(1),
            tax_free: Self::bool_from_int(row.try_get("taxfree").unwrap_or(0)),
            allow_direct_teleport: Self::bool_from_int(
                row.try_get("allowdirectteleport").unwrap_or(1),
            ),
            redirect_grid_x: row.try_get("redirectgridx").unwrap_or(0),
            redirect_grid_y: row.try_get("redirectgridy").unwrap_or(0),
            sun_position: row
                .try_get::<f32, _>("sunposition")
                .map(|v| v as f64)
                .unwrap_or(0.0),
            estate_skip_scripts: Self::bool_from_int(row.try_get("estateskipscripts").unwrap_or(0)),
            billable_factor: row
                .try_get::<f32, _>("billablefactor")
                .map(|v| v as f64)
                .unwrap_or(1.0),
            public_access: Self::bool_from_int(row.try_get("publicaccess").unwrap_or(1)),
            abuse_email: row.try_get::<String, _>("abuseemail").unwrap_or_default(),
            deny_minors: Self::bool_from_int(row.try_get("denyminors").unwrap_or(0)),
            allow_landmark: Self::bool_from_int(row.try_get("allowlandmark").unwrap_or(1)),
            allow_parcel_changes: Self::bool_from_int(
                row.try_get("allowparcelchanges").unwrap_or(1),
            ),
            allow_set_home: Self::bool_from_int(row.try_get("allowsethome").unwrap_or(1)),
            allow_environment_override: Self::bool_from_int(
                row.try_get("allowenviromentoverride").unwrap_or(0),
            ),
            estate_managers: Vec::new(),
            estate_users: Vec::new(),
            estate_groups: Vec::new(),
            estate_bans: Vec::new(),
        }
    }
}

#[async_trait]
impl EstateServiceTrait for LocalEstateService {
    async fn load_estate_by_region(
        &self,
        region_id: &str,
        create: bool,
    ) -> Result<Option<EstateSettings>> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let row = sqlx::query(
            "SELECT es.* FROM estate_settings es \
             JOIN estate_map em ON es.estateid = em.estateid \
             WHERE em.regionid = $1",
        )
        .bind(region_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => {
                let mut settings = Self::row_to_settings(&r);
                self.load_estate_extras(settings.estate_id, &mut settings)
                    .await?;
                Ok(Some(settings))
            }
            None => {
                if create {
                    let default_row =
                        sqlx::query("SELECT * FROM estate_settings WHERE estateid = 1")
                            .fetch_optional(pool)
                            .await?;

                    if let Some(r) = default_row {
                        sqlx::query(
                            "INSERT INTO estate_map (regionid, estateid) VALUES ($1, 1) \
                             ON CONFLICT (regionid) DO NOTHING",
                        )
                        .bind(region_id)
                        .execute(pool)
                        .await?;

                        let mut settings = Self::row_to_settings(&r);
                        self.load_estate_extras(settings.estate_id, &mut settings)
                            .await?;
                        debug!("[ESTATE] Linked region {} to default estate 1", region_id);
                        Ok(Some(settings))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
        }
    }

    async fn load_estate_by_id(&self, estate_id: i32) -> Result<Option<EstateSettings>> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let row = sqlx::query("SELECT * FROM estate_settings WHERE estateid = $1")
            .bind(estate_id)
            .fetch_optional(pool)
            .await?;

        match row {
            Some(r) => {
                let mut settings = Self::row_to_settings(&r);
                self.load_estate_extras(settings.estate_id, &mut settings)
                    .await?;
                Ok(Some(settings))
            }
            None => Ok(None),
        }
    }

    async fn store_estate_settings(&self, settings: &EstateSettings) -> Result<bool> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let b = |v: bool| -> i32 {
            if v {
                1
            } else {
                0
            }
        };

        sqlx::query(
            "UPDATE estate_settings SET \
             estatename = $2, estateowner = $3, parentestateid = $4, \
             abuseemailtoestateowner = $5, denyanonymous = $6, resethomeonteleport = $7, \
             fixedsun = $8, denytransacted = $9, blockdwell = $10, denyidentified = $11, \
             allowvoice = $12, useglobaltime = $13, pricepermeter = $14, taxfree = $15, \
             allowdirectteleport = $16, redirectgridx = $17, redirectgridy = $18, \
             sunposition = $19, estateskipscripts = $20, billablefactor = $21, \
             publicaccess = $22, abuseemail = $23, denyminors = $24, \
             allowlandmark = $25, allowparcelchanges = $26, allowsethome = $27, \
             allowenviromentoverride = $28 \
             WHERE estateid = $1",
        )
        .bind(settings.estate_id)
        .bind(&settings.estate_name)
        .bind(&settings.estate_owner)
        .bind(settings.parent_estate_id)
        .bind(b(settings.abuse_email_to_estate_owner))
        .bind(b(settings.deny_anonymous))
        .bind(b(settings.reset_home_on_teleport))
        .bind(b(settings.fixed_sun))
        .bind(b(settings.deny_transacted))
        .bind(b(settings.block_dwell))
        .bind(b(settings.deny_identified))
        .bind(b(settings.allow_voice))
        .bind(b(settings.use_global_time))
        .bind(settings.price_per_meter)
        .bind(b(settings.tax_free))
        .bind(b(settings.allow_direct_teleport))
        .bind(settings.redirect_grid_x)
        .bind(settings.redirect_grid_y)
        .bind(settings.sun_position as f32)
        .bind(b(settings.estate_skip_scripts))
        .bind(settings.billable_factor as f32)
        .bind(b(settings.public_access))
        .bind(&settings.abuse_email)
        .bind(b(settings.deny_minors))
        .bind(b(settings.allow_landmark))
        .bind(b(settings.allow_parcel_changes))
        .bind(b(settings.allow_set_home))
        .bind(b(settings.allow_environment_override))
        .execute(pool)
        .await?;

        debug!("[ESTATE] Stored settings for estate {}", settings.estate_id);
        Ok(true)
    }

    async fn link_region(&self, region_id: &str, estate_id: i32) -> Result<bool> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query(
            "INSERT INTO estate_map (regionid, estateid) VALUES ($1, $2) \
             ON CONFLICT (regionid) DO UPDATE SET estateid = $2",
        )
        .bind(region_id)
        .bind(estate_id)
        .execute(pool)
        .await?;

        debug!(
            "[ESTATE] Linked region {} to estate {}",
            region_id, estate_id
        );
        Ok(true)
    }

    async fn get_regions(&self, estate_id: i32) -> Result<Vec<String>> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let rows = sqlx::query("SELECT regionid FROM estate_map WHERE estateid = $1")
            .bind(estate_id)
            .fetch_all(pool)
            .await?;

        let mut result = Vec::new();
        for row in &rows {
            use sqlx::Row;
            if let Ok(rid) = row.try_get::<String, _>("regionid") {
                result.push(rid.trim().to_string());
            }
        }

        Ok(result)
    }

    async fn get_estates_by_name(&self, name: &str) -> Result<Vec<i32>> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let rows = sqlx::query("SELECT estateid FROM estate_settings WHERE estatename = $1")
            .bind(name)
            .fetch_all(pool)
            .await?;

        let mut result = Vec::new();
        for row in &rows {
            use sqlx::Row;
            if let Ok(eid) = row.try_get::<i32, _>("estateid") {
                result.push(eid);
            }
        }
        Ok(result)
    }

    async fn get_estates_by_owner(&self, owner_id: &str) -> Result<Vec<i32>> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let rows = sqlx::query("SELECT estateid FROM estate_settings WHERE estateowner = $1")
            .bind(owner_id)
            .fetch_all(pool)
            .await?;

        let mut result = Vec::new();
        for row in &rows {
            use sqlx::Row;
            if let Ok(eid) = row.try_get::<i32, _>("estateid") {
                result.push(eid);
            }
        }
        Ok(result)
    }

    async fn get_estates_all(&self) -> Result<Vec<i32>> {
        let pool = self
            .db
            .postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let rows = sqlx::query("SELECT estateid FROM estate_settings ORDER BY estateid")
            .fetch_all(pool)
            .await?;

        let mut result = Vec::new();
        for row in &rows {
            use sqlx::Row;
            if let Ok(eid) = row.try_get::<i32, _>("estateid") {
                result.push(eid);
            }
        }
        Ok(result)
    }
}
