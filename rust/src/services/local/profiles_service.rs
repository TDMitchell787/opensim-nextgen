use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use tracing::{info, debug};

use crate::services::traits::{
    ProfilesServiceTrait, UserProfileProperties, UserProfilePick,
    UserClassifiedAdd, UserProfileNotes, UserPreferences,
};
use crate::database::DatabaseConnection;

pub struct LocalProfilesService {
    db: Arc<DatabaseConnection>,
}

impl LocalProfilesService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ProfilesServiceTrait for LocalProfilesService {
    async fn get_classifieds(&self, creator_id: Uuid) -> Result<Vec<UserClassifiedAdd>> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let rows = sqlx::query(
            "SELECT classifieduuid, name FROM userclassifieds WHERE creatoruuid = $1"
        )
        .bind(creator_id)
        .fetch_all(pool)
        .await?;

        let mut result = Vec::new();
        for row in &rows {
            use sqlx::Row;
            result.push(UserClassifiedAdd {
                classified_id: row.try_get::<Uuid, _>("classifieduuid").unwrap_or_default(),
                creator_id,
                name: row.try_get::<String, _>("name").unwrap_or_default(),
                ..Default::default()
            });
        }
        Ok(result)
    }

    async fn get_classified(&self, classified_id: Uuid) -> Result<Option<UserClassifiedAdd>> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let row = sqlx::query(
            "SELECT classifieduuid, creatoruuid, creationdate, expirationdate, category, \
             name, description, parceluuid, parentestate, snapshotuuid, simname, \
             posglobal, parcelname, classifiedflags, priceforlisting \
             FROM userclassifieds WHERE classifieduuid = $1"
        )
        .bind(classified_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => {
                use sqlx::Row;
                Ok(Some(UserClassifiedAdd {
                    classified_id: r.try_get("classifieduuid").unwrap_or_default(),
                    creator_id: r.try_get("creatoruuid").unwrap_or_default(),
                    creation_date: r.try_get::<i32, _>("creationdate").unwrap_or(0),
                    expiration_date: r.try_get::<i32, _>("expirationdate").unwrap_or(0),
                    category: r.try_get::<i32, _>("category").unwrap_or(0),
                    name: r.try_get("name").unwrap_or_default(),
                    description: r.try_get("description").unwrap_or_default(),
                    parcel_id: r.try_get("parceluuid").unwrap_or_default(),
                    parent_estate: r.try_get::<i32, _>("parentestate").unwrap_or(0),
                    snapshot_id: r.try_get("snapshotuuid").unwrap_or_default(),
                    sim_name: r.try_get("simname").unwrap_or_default(),
                    global_pos: r.try_get("posglobal").unwrap_or_default(),
                    parcel_name: r.try_get("parcelname").unwrap_or_default(),
                    flags: r.try_get::<i32, _>("classifiedflags").unwrap_or(0),
                    listing_price: r.try_get::<i32, _>("priceforlisting").unwrap_or(0),
                }))
            }
            None => Ok(None),
        }
    }

    async fn update_classified(&self, classified: &UserClassifiedAdd) -> Result<bool> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        info!("[PROFILES] update_classified: {} by {}", classified.classified_id, classified.creator_id);

        sqlx::query(
            "INSERT INTO userclassifieds (classifieduuid, creatoruuid, creationdate, expirationdate, \
             category, name, description, parceluuid, parentestate, snapshotuuid, simname, \
             posglobal, parcelname, classifiedflags, priceforlisting) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15) \
             ON CONFLICT (classifieduuid) DO UPDATE SET \
             category = $5, name = $6, description = $7, parceluuid = $8, parentestate = $9, \
             snapshotuuid = $10, simname = $11, posglobal = $12, parcelname = $13, \
             classifiedflags = $14, priceforlisting = $15"
        )
        .bind(classified.classified_id)
        .bind(classified.creator_id)
        .bind(classified.creation_date)
        .bind(classified.expiration_date)
        .bind(classified.category)
        .bind(&classified.name)
        .bind(&classified.description)
        .bind(classified.parcel_id)
        .bind(classified.parent_estate)
        .bind(classified.snapshot_id)
        .bind(&classified.sim_name)
        .bind(&classified.global_pos)
        .bind(&classified.parcel_name)
        .bind(classified.flags)
        .bind(classified.listing_price)
        .execute(pool)
        .await?;

        Ok(true)
    }

    async fn delete_classified(&self, classified_id: Uuid) -> Result<bool> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query("DELETE FROM userclassifieds WHERE classifieduuid = $1")
            .bind(classified_id)
            .execute(pool)
            .await?;

        Ok(true)
    }

    async fn get_picks(&self, creator_id: Uuid) -> Result<Vec<UserProfilePick>> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let rows = sqlx::query(
            "SELECT pickuuid, name FROM userpicks WHERE creatoruuid = $1"
        )
        .bind(creator_id)
        .fetch_all(pool)
        .await?;

        let mut result = Vec::new();
        for row in &rows {
            use sqlx::Row;
            result.push(UserProfilePick {
                pick_id: row.try_get("pickuuid").unwrap_or_default(),
                creator_id,
                name: row.try_get("name").unwrap_or_default(),
                ..Default::default()
            });
        }
        Ok(result)
    }

    async fn get_pick(&self, pick_id: Uuid) -> Result<Option<UserProfilePick>> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let row = sqlx::query(
            "SELECT pickuuid, creatoruuid, toppick, parceluuid, name, description, \
             snapshotuuid, user_, originalname, simname, posglobal, sortorder, enabled \
             FROM userpicks WHERE pickuuid = $1"
        )
        .bind(pick_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => {
                use sqlx::Row;
                Ok(Some(UserProfilePick {
                    pick_id: r.try_get("pickuuid").unwrap_or_default(),
                    creator_id: r.try_get("creatoruuid").unwrap_or_default(),
                    top_pick: r.try_get::<bool, _>("toppick").unwrap_or(false),
                    parcel_id: r.try_get("parceluuid").unwrap_or_default(),
                    name: r.try_get("name").unwrap_or_default(),
                    description: r.try_get("description").unwrap_or_default(),
                    snapshot_id: r.try_get("snapshotuuid").unwrap_or_default(),
                    user: r.try_get("user_").unwrap_or_default(),
                    original_name: r.try_get("originalname").unwrap_or_default(),
                    sim_name: r.try_get("simname").unwrap_or_default(),
                    global_pos: r.try_get("posglobal").unwrap_or_default(),
                    sort_order: r.try_get::<i32, _>("sortorder").unwrap_or(0),
                    enabled: r.try_get::<bool, _>("enabled").unwrap_or(false),
                }))
            }
            None => Ok(None),
        }
    }

    async fn update_pick(&self, pick: &UserProfilePick) -> Result<bool> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query(
            "INSERT INTO userpicks (pickuuid, creatoruuid, toppick, parceluuid, name, \
             description, snapshotuuid, user_, originalname, simname, posglobal, sortorder, enabled) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) \
             ON CONFLICT (pickuuid) DO UPDATE SET \
             toppick = $3, parceluuid = $4, name = $5, description = $6, snapshotuuid = $7, \
             user_ = $8, originalname = $9, simname = $10, posglobal = $11, sortorder = $12, enabled = $13"
        )
        .bind(pick.pick_id)
        .bind(pick.creator_id)
        .bind(pick.top_pick)
        .bind(pick.parcel_id)
        .bind(&pick.name)
        .bind(&pick.description)
        .bind(pick.snapshot_id)
        .bind(&pick.user)
        .bind(&pick.original_name)
        .bind(&pick.sim_name)
        .bind(&pick.global_pos)
        .bind(pick.sort_order)
        .bind(pick.enabled)
        .execute(pool)
        .await?;

        Ok(true)
    }

    async fn delete_pick(&self, pick_id: Uuid) -> Result<bool> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query("DELETE FROM userpicks WHERE pickuuid = $1")
            .bind(pick_id)
            .execute(pool)
            .await?;

        Ok(true)
    }

    async fn get_notes(&self, user_id: Uuid, target_id: Uuid) -> Result<Option<UserProfileNotes>> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let row = sqlx::query(
            "SELECT useruuid, targetuuid, notes FROM usernotes WHERE useruuid = $1 AND targetuuid = $2"
        )
        .bind(user_id)
        .bind(target_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => {
                use sqlx::Row;
                Ok(Some(UserProfileNotes {
                    user_id: r.try_get("useruuid").unwrap_or_default(),
                    target_id: r.try_get("targetuuid").unwrap_or_default(),
                    notes: r.try_get("notes").unwrap_or_default(),
                }))
            }
            None => Ok(None),
        }
    }

    async fn update_notes(&self, notes: &UserProfileNotes) -> Result<bool> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query(
            "INSERT INTO usernotes (useruuid, targetuuid, notes) VALUES ($1, $2, $3) \
             ON CONFLICT (useruuid, targetuuid) DO UPDATE SET notes = $3"
        )
        .bind(notes.user_id)
        .bind(notes.target_id)
        .bind(&notes.notes)
        .execute(pool)
        .await?;

        Ok(true)
    }

    async fn get_properties(&self, user_id: Uuid) -> Result<Option<UserProfileProperties>> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let row = sqlx::query(
            "SELECT useruuid, profilepartner, profileurl, profileimage, \
             profileabouttext, profilefirstimage, profilefirsttext, \
             profilewanttotext, profilewanttomask, profileskillstext, profileskillsmask, \
             profilelanguages \
             FROM userprofile WHERE useruuid = $1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => {
                use sqlx::Row;
                Ok(Some(UserProfileProperties {
                    user_id: r.try_get("useruuid").unwrap_or_default(),
                    partner_id: r.try_get("profilepartner").unwrap_or_default(),
                    profile_url: r.try_get("profileurl").unwrap_or_default(),
                    image_id: r.try_get("profileimage").unwrap_or_default(),
                    about_text: r.try_get("profileabouttext").unwrap_or_default(),
                    first_life_image_id: r.try_get("profilefirstimage").unwrap_or_default(),
                    first_life_text: r.try_get("profilefirsttext").unwrap_or_default(),
                    want_to_text: r.try_get("profilewanttotext").unwrap_or_default(),
                    want_to_mask: r.try_get::<i32, _>("profilewanttomask").unwrap_or(0),
                    skills_text: r.try_get("profileskillstext").unwrap_or_default(),
                    skills_mask: r.try_get::<i32, _>("profileskillsmask").unwrap_or(0),
                    languages: r.try_get("profilelanguages").unwrap_or_default(),
                }))
            }
            None => Ok(None),
        }
    }

    async fn update_properties(&self, props: &UserProfileProperties) -> Result<bool> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query(
            "INSERT INTO userprofile (useruuid, profilepartner, profileurl, profileimage, \
             profileabouttext, profilefirstimage, profilefirsttext, \
             profilewanttotext, profilewanttomask, profileskillstext, profileskillsmask, \
             profilelanguages) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) \
             ON CONFLICT (useruuid) DO UPDATE SET \
             profilepartner = $2, profileurl = $3, profileimage = $4, profileabouttext = $5, \
             profilefirstimage = $6, profilefirsttext = $7, profilewanttotext = $8, \
             profilewanttomask = $9, profileskillstext = $10, profileskillsmask = $11, \
             profilelanguages = $12"
        )
        .bind(props.user_id)
        .bind(props.partner_id)
        .bind(&props.profile_url)
        .bind(props.image_id)
        .bind(&props.about_text)
        .bind(props.first_life_image_id)
        .bind(&props.first_life_text)
        .bind(&props.want_to_text)
        .bind(props.want_to_mask)
        .bind(&props.skills_text)
        .bind(props.skills_mask)
        .bind(&props.languages)
        .execute(pool)
        .await?;

        Ok(true)
    }

    async fn get_preferences(&self, user_id: Uuid) -> Result<Option<UserPreferences>> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        let row = sqlx::query(
            "SELECT useruuid, imviaemail, visible, email FROM usersettings WHERE useruuid = $1"
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        match row {
            Some(r) => {
                use sqlx::Row;
                Ok(Some(UserPreferences {
                    user_id: r.try_get("useruuid").unwrap_or_default(),
                    im_via_email: r.try_get::<bool, _>("imviaemail").unwrap_or(false),
                    visible: r.try_get::<bool, _>("visible").unwrap_or(true),
                    email: r.try_get("email").unwrap_or_default(),
                }))
            }
            None => Ok(Some(UserPreferences {
                user_id,
                im_via_email: false,
                visible: true,
                email: String::new(),
            })),
        }
    }

    async fn update_preferences(&self, prefs: &UserPreferences) -> Result<bool> {
        let pool = self.db.postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("No PG pool"))?;

        sqlx::query(
            "INSERT INTO usersettings (useruuid, imviaemail, visible, email) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (useruuid) DO UPDATE SET imviaemail = $2, visible = $3, email = $4"
        )
        .bind(prefs.user_id)
        .bind(prefs.im_via_email)
        .bind(prefs.visible)
        .bind(&prefs.email)
        .execute(pool)
        .await?;

        Ok(true)
    }
}
