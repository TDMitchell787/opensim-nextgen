//! Database initialization system for OpenSim Next
//!
//! Automatically creates and populates all required database tables
//! with default data on first startup, matching OpenSim Master behavior

use anyhow::Result;
use sqlx::{SqliteConnection, SqlitePool};
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::migration_engine::{
    get_asset_store_migrations, get_estate_store_migrations, get_inventory_store_migrations,
    MigrationEngine,
};

/// Database initialization coordinator
pub struct DatabaseInitializer {
    pool: SqlitePool,
}

impl DatabaseInitializer {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Run complete database initialization
    pub async fn initialize(&self) -> Result<()> {
        info!("🔧 Starting OpenSim Next database initialization...");

        let mut conn = self.pool.acquire().await?;

        // Step 1: Run all migrations
        self.run_all_migrations(&mut conn).await?;

        // Step 2: Populate default data
        self.populate_default_data(&mut conn).await?;

        info!("✅ Database initialization completed successfully");
        Ok(())
    }

    /// Execute all database migrations
    async fn run_all_migrations(&self, conn: &mut SqliteConnection) -> Result<()> {
        info!("📊 Running database migrations...");

        // Initialize migrations table
        MigrationEngine::init_migrations_table(conn).await?;

        // Asset Store migrations
        let asset_engine = MigrationEngine::new("AssetStore".to_string());
        asset_engine
            .update_store(conn, get_asset_store_migrations())
            .await?;

        // Inventory Store migrations
        let inventory_engine = MigrationEngine::new("InventoryStore".to_string());
        inventory_engine
            .update_store(conn, get_inventory_store_migrations())
            .await?;

        // Estate Store migrations
        let estate_engine = MigrationEngine::new("EstateStore".to_string());
        estate_engine
            .update_store(conn, get_estate_store_migrations())
            .await?;

        info!("✅ All database migrations completed");
        Ok(())
    }

    /// Populate database with essential default data
    async fn populate_default_data(&self, conn: &mut SqliteConnection) -> Result<()> {
        info!("📦 Populating default database content...");

        // Check if we already have data (avoid re-population)
        let asset_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM assets")
            .fetch_one(&mut *conn)
            .await?;

        if asset_count > 1 {
            info!("Database already contains data, skipping population");
            return Ok(());
        }

        // Populate default assets
        self.populate_default_assets(conn).await?;

        // Populate default inventory structure
        self.populate_default_inventory(conn).await?;

        // Populate default estate
        self.populate_default_estate(conn).await?;

        info!("✅ Default data population completed");
        Ok(())
    }

    /// Create essential default assets including Ruth avatar wearables
    async fn populate_default_assets(&self, conn: &mut SqliteConnection) -> Result<()> {
        info!("🎨 Creating default assets...");

        let creator = "11111111-1111-0000-0000-000100bba000";

        let default_assets = vec![
            DefaultAsset {
                id: "5646d39e-d3d7-6aff-ed71-30fc87d64a91",
                name: "Default Environment",
                description: "Default environment settings",
                asset_type: 45,
                data: b"default_environment_data".to_vec(),
                creator_id: creator,
            },
            // === RUTH AVATAR BODY PARTS (type 13) ===
            // Shape - from BodyPartsAssetSet/base_shape.dat
            DefaultAsset {
                id: "66c41e39-38f9-f75a-024e-585989bfab73",
                name: "Default Shape",
                description: "Default avatar shape",
                asset_type: 13,
                data: create_ruth_shape_data(),
                creator_id: creator,
            },
            // Skin - from BodyPartsAssetSet/base_skin.dat
            DefaultAsset {
                id: "77c41e39-38f9-f75a-024e-585989bbabbb",
                name: "Default Skin",
                description: "Default avatar skin",
                asset_type: 13,
                data: create_ruth_skin_data(),
                creator_id: creator,
            },
            // Hair - from BodyPartsAssetSet/base_hair.dat
            DefaultAsset {
                id: "d342e6c0-b9d2-11dc-95ff-0800200c9a66",
                name: "Default Hair",
                description: "Default avatar hair",
                asset_type: 13,
                data: create_ruth_hair_data(),
                creator_id: creator,
            },
            // Eyes - from BodyPartsAssetSet/base_eyes.dat
            DefaultAsset {
                id: "4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7",
                name: "Default Eyes",
                description: "Default avatar eyes",
                asset_type: 13,
                data: create_ruth_eyes_data(),
                creator_id: creator,
            },
            // === RUTH AVATAR CLOTHING (type 5) ===
            // Shirt - from ClothingAssetSet/newshirt.dat
            DefaultAsset {
                id: "00000000-38f9-1111-024e-222222111110",
                name: "Default Shirt",
                description: "Default avatar shirt",
                asset_type: 5,
                data: create_ruth_shirt_data(),
                creator_id: creator,
            },
            // Pants - from ClothingAssetSet/newpants.dat
            DefaultAsset {
                id: "00000000-38f9-1111-024e-222222111120",
                name: "Default Pants",
                description: "Default avatar pants",
                asset_type: 5,
                data: create_ruth_pants_data(),
                creator_id: creator,
            },
            // === TEXTURES REFERENCED BY WEARABLES ===
            DefaultAsset {
                id: "5748decc-f629-461c-9a36-a35a221fe21f",
                name: "Default Clothing Texture",
                description: "White clothing texture",
                asset_type: 0,
                data: create_default_j2k_texture(),
                creator_id: creator,
            },
            DefaultAsset {
                id: "7ca39b4c-bd19-4699-aff7-f93fd03d3e7b",
                name: "Default Hair Texture",
                description: "Brown hair texture",
                asset_type: 0,
                data: create_default_j2k_texture(),
                creator_id: creator,
            },
            DefaultAsset {
                id: "6522e74d-1660-4e7f-b601-6f48c1659a77",
                name: "Default Eyes Texture",
                description: "Brown eyes texture",
                asset_type: 0,
                data: create_default_j2k_texture(),
                creator_id: creator,
            },
            DefaultAsset {
                id: "00000000-0000-1111-9999-000000000012",
                name: "Default Skin Head Texture",
                description: "Default skin head",
                asset_type: 0,
                data: create_default_j2k_texture(),
                creator_id: creator,
            },
            DefaultAsset {
                id: "00000000-0000-1111-9999-000000000010",
                name: "Default Skin Upper Texture",
                description: "Default skin upper body",
                asset_type: 0,
                data: create_default_j2k_texture(),
                creator_id: creator,
            },
            DefaultAsset {
                id: "00000000-0000-1111-9999-000000000011",
                name: "Default Skin Lower Texture",
                description: "Default skin lower body",
                asset_type: 0,
                data: create_default_j2k_texture(),
                creator_id: creator,
            },
            // === BAKED TEXTURES (sent in AvatarAppearance) ===
            // Phase 82: Using REAL Ruth textures from assets/ruth/ directory
            // These UUIDs must match what's sent in udp/server.rs:1591-1595
            DefaultAsset {
                id: "5a9f4a74-30f2-821c-b88d-70499d3e7183",
                name: "Default Baked Head",
                description: "Ruth baked head texture (female face.j2c)",
                asset_type: 0,
                data: load_ruth_head_texture(),
                creator_id: creator,
            },
            DefaultAsset {
                id: "ae2de45c-d252-50b8-5c6e-19f39ce79317",
                name: "Default Baked Upper Body",
                description: "Ruth baked upper body texture (female body.j2c)",
                asset_type: 0,
                data: load_ruth_upper_texture(),
                creator_id: creator,
            },
            DefaultAsset {
                id: "24daea5f-0539-cfcf-047f-fbc40b2786ba",
                name: "Default Baked Lower Body",
                description: "Ruth baked lower body texture (female bottom.j2c)",
                asset_type: 0,
                data: load_ruth_lower_texture(),
                creator_id: creator,
            },
            DefaultAsset {
                id: "52cc6bb6-2ee5-e632-d3ad-50197b1dcb8a",
                name: "Default Baked Eyes",
                description: "Ruth baked eyes texture (eyes.j2c)",
                asset_type: 0,
                data: load_ruth_eyes_texture(),
                creator_id: creator,
            },
            DefaultAsset {
                id: "09aac1fb-6bce-0bee-7d44-caac6dbb6c63",
                name: "Default Baked Hair",
                description: "Ruth baked hair texture (open sim hair base.j2c)",
                asset_type: 0,
                data: load_ruth_hair_texture(),
                creator_id: creator,
            },
        ];

        for asset in &default_assets {
            sqlx::query(r#"
                INSERT OR IGNORE INTO assets
                (id, name, description, assetType, local, temporary, data, create_time, access_time, asset_flags, CreatorID)
                VALUES (?, ?, ?, ?, 1, 0, ?, ?, ?, 0, ?)
            "#)
            .bind(asset.id)
            .bind(asset.name)
            .bind(asset.description)
            .bind(asset.asset_type)
            .bind(&asset.data)
            .bind(chrono::Utc::now().timestamp())
            .bind(chrono::Utc::now().timestamp())
            .bind(asset.creator_id)
            .execute(&mut *conn)
            .await?;
        }

        info!(
            "✅ Created {} default assets (6 wearables + 11 textures + 1 env)",
            default_assets.len()
        );
        Ok(())
    }

    /// Create default inventory folder structure for all users
    async fn populate_default_inventory(&self, conn: &mut SqliteConnection) -> Result<()> {
        info!("📁 Creating default inventory structure...");

        // Get all user accounts
        let users: Vec<(String,)> = sqlx::query_as("SELECT PrincipalID FROM UserAccounts")
            .fetch_all(&mut *conn)
            .await?;

        for (user_id,) in &users {
            self.create_user_inventory(&user_id, conn).await?;
        }

        info!("✅ Created default inventory for {} users", users.len());
        Ok(())
    }

    /// Create inventory structure for a specific user
    async fn create_user_inventory(
        &self,
        user_id: &str,
        conn: &mut SqliteConnection,
    ) -> Result<()> {
        debug!("Creating inventory for user: {}", user_id);

        let root_folder_id = Uuid::new_v4().to_string();

        // Create root folder
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO inventoryfolders 
            (folderID, agentID, parentFolderID, folderName, type, version)
            VALUES (?, ?, '00000000-0000-0000-0000-000000000000', 'My Inventory', 8, 1)
        "#,
        )
        .bind(&root_folder_id)
        .bind(user_id)
        .execute(&mut *conn)
        .await?;

        // Create standard folders
        let standard_folders = vec![
            ("Animations", 20, 1),
            ("Body Parts", 13, 1),
            ("Calling Cards", 2, 1),
            ("Clothing", 5, 1),
            ("Current Outfit", 46, 1),
            ("Favorites", 23, 1),
            ("Gestures", 21, 1),
            ("Landmarks", 3, 1),
            ("Lost And Found", 16, 1),
            ("Notecards", 7, 1),
            ("Objects", 6, 1),
            ("Photo Album", 15, 1),
            ("Scripts", 10, 1),
            ("Sounds", 1, 1),
            ("Textures", 0, 1),
            ("Trash", 14, 1),
        ];

        for (folder_name, folder_type, version) in standard_folders {
            let folder_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO inventoryfolders 
                (folderID, agentID, parentFolderID, folderName, type, version)
                VALUES (?, ?, ?, ?, ?, ?)
            "#,
            )
            .bind(&folder_id)
            .bind(user_id)
            .bind(&root_folder_id)
            .bind(folder_name)
            .bind(folder_type)
            .bind(version)
            .execute(&mut *conn)
            .await?;

            if folder_name == "Body Parts" {
                self.create_default_bodyparts(&folder_id, user_id, conn)
                    .await?;
            }
            if folder_name == "Clothing" {
                self.create_default_clothing(&folder_id, user_id, conn)
                    .await?;
            }
            if folder_name == "Current Outfit" {
                self.create_default_cof_links(&folder_id, user_id, conn)
                    .await?;
            }
        }

        Ok(())
    }

    /// Create default bodypart items for user
    /// CRITICAL: Item IDs MUST match exactly what's sent in AgentWearablesUpdateMessage::with_default_ruth()
    /// The viewer uses these item IDs to look up wearables in inventory
    async fn create_default_bodyparts(
        &self,
        folder_id: &str,
        user_id: &str,
        conn: &mut SqliteConnection,
    ) -> Result<()> {
        // Body parts (asset_type 13, inv_type 13) - goes in Body Parts folder
        // Item IDs match AgentWearablesUpdateMessage::with_default_ruth() EXACTLY
        let bodyparts = vec![
            // (name, item_id, asset_id, asset_type, inv_type)
            // Shape (wearable type 0)
            (
                "Default Shape",
                "66c41e39-38f9-f75a-024e-585989bfaba9",
                "66c41e39-38f9-f75a-024e-585989bfab73",
                13,
                13,
            ),
            // Skin (wearable type 1)
            (
                "Default Skin",
                "77c41e39-38f9-f75a-024e-585989bfabc9",
                "77c41e39-38f9-f75a-024e-585989bbabbb",
                13,
                13,
            ),
            // Hair (wearable type 2)
            (
                "Default Hair",
                "d342e6c1-b9d2-11dc-95ff-0800200c9a66",
                "d342e6c0-b9d2-11dc-95ff-0800200c9a66",
                13,
                13,
            ),
            // Eyes (wearable type 3)
            (
                "Default Eyes",
                "cdc31054-eed8-4021-994f-4e0c6e861b50",
                "4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7",
                13,
                13,
            ),
        ];

        for (name, item_id, asset_id, asset_type, inv_type) in bodyparts {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO inventoryitems
                (inventoryID, assetID, assetType, inventoryName, inventoryDescription,
                 inventoryNextPermissions, inventoryCurrentPermissions, invType, creatorID,
                 inventoryBasePermissions, inventoryEveryOnePermissions, salePrice, saleType,
                 creationDate, groupID, groupOwned, flags, avatarID, parentFolderID,
                 inventoryGroupPermissions)
                VALUES (?, ?, ?, ?, ?, 647168, 647168, ?, '11111111-1111-0000-0000-000100bba000',
                        647168, 0, 0, 0, ?, '00000000-0000-0000-0000-000000000000', 0, 0, ?, ?, 0)
            "#,
            )
            .bind(item_id)
            .bind(asset_id)
            .bind(asset_type)
            .bind(name)
            .bind(format!("Default {} for avatar", name))
            .bind(inv_type)
            .bind(chrono::Utc::now().timestamp())
            .bind(user_id)
            .bind(folder_id)
            .execute(&mut *conn)
            .await?;
        }

        debug!("Created 4 default body parts in Body Parts folder");
        Ok(())
    }

    /// Create default clothing items for user
    /// CRITICAL: Item IDs MUST match exactly what's sent in AgentWearablesUpdateMessage::with_default_ruth()
    async fn create_default_clothing(
        &self,
        folder_id: &str,
        user_id: &str,
        conn: &mut SqliteConnection,
    ) -> Result<()> {
        // Clothing (asset_type 5, inv_type 5) - goes in Clothing folder
        // Item IDs match AgentWearablesUpdateMessage::with_default_ruth() EXACTLY
        let clothing = vec![
            // (name, item_id, asset_id, asset_type, inv_type)
            // Shirt (wearable type 4)
            (
                "Default Shirt",
                "77c41e39-38f9-f75a-0000-585989bf0000",
                "00000000-38f9-1111-024e-222222111110",
                5,
                5,
            ),
            // Pants (wearable type 5)
            (
                "Default Pants",
                "77c41e39-38f9-f75a-0000-5859892f1111",
                "00000000-38f9-1111-024e-222222111120",
                5,
                5,
            ),
        ];

        for (name, item_id, asset_id, asset_type, inv_type) in clothing {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO inventoryitems
                (inventoryID, assetID, assetType, inventoryName, inventoryDescription,
                 inventoryNextPermissions, inventoryCurrentPermissions, invType, creatorID,
                 inventoryBasePermissions, inventoryEveryOnePermissions, salePrice, saleType,
                 creationDate, groupID, groupOwned, flags, avatarID, parentFolderID,
                 inventoryGroupPermissions)
                VALUES (?, ?, ?, ?, ?, 647168, 647168, ?, '11111111-1111-0000-0000-000100bba000',
                        647168, 0, 0, 0, ?, '00000000-0000-0000-0000-000000000000', 0, 0, ?, ?, 0)
            "#,
            )
            .bind(item_id)
            .bind(asset_id)
            .bind(asset_type)
            .bind(name)
            .bind(format!("Default {} for avatar", name))
            .bind(inv_type)
            .bind(chrono::Utc::now().timestamp())
            .bind(user_id)
            .bind(folder_id)
            .execute(&mut *conn)
            .await?;
        }

        debug!("Created 2 default clothing items in Clothing folder");
        Ok(())
    }

    /// Phase 95.5: Create default COF (Current Outfit Folder) link items
    /// These are inventory links (assetType=24, invType=18) pointing to wearable ITEM UUIDs
    async fn create_default_cof_links(
        &self,
        folder_id: &str,
        user_id: &str,
        conn: &mut SqliteConnection,
    ) -> Result<()> {
        // (wearable_item_id, name, wearable_type_flag)
        let cof_links = vec![
            ("66c41e39-38f9-f75a-024e-585989bfaba9", "Default Shape", 0),
            ("77c41e39-38f9-f75a-024e-585989bfabc9", "Default Skin", 1),
            ("d342e6c1-b9d2-11dc-95ff-0800200c9a66", "Default Hair", 2),
            ("cdc31054-eed8-4021-994f-4e0c6e861b50", "Default Eyes", 3),
            ("77c41e39-38f9-f75a-0000-585989bf0000", "Default Shirt", 4),
            ("77c41e39-38f9-f75a-0000-5859892f1111", "Default Pants", 5),
        ];

        for (wearable_item_id, name, wearable_type) in cof_links {
            let link_id = Uuid::new_v4().to_string();
            sqlx::query(r#"
                INSERT OR IGNORE INTO inventoryitems
                (inventoryID, assetID, assetType, inventoryName, inventoryDescription,
                 inventoryNextPermissions, inventoryCurrentPermissions, invType, creatorID,
                 inventoryBasePermissions, inventoryEveryOnePermissions, salePrice, saleType,
                 creationDate, groupID, groupOwned, flags, avatarID, parentFolderID,
                 inventoryGroupPermissions)
                VALUES (?, ?, 24, ?, ?, 32768, 32768, 18, '11111111-1111-0000-0000-000100bba000',
                        32768, 32768, 0, 0, ?, '00000000-0000-0000-0000-000000000000', 0, ?, ?, ?, 32768)
            "#)
            .bind(&link_id)
            .bind(wearable_item_id)  // assetID = wearable ITEM UUID
            .bind(name)
            .bind(format!("@{}", name))
            .bind(chrono::Utc::now().timestamp())
            .bind(wearable_type)
            .bind(user_id)
            .bind(folder_id)
            .execute(&mut *conn)
            .await?;
        }

        debug!("Created 6 default COF links in Current Outfit folder");
        Ok(())
    }

    /// Create default estate
    async fn populate_default_estate(&self, conn: &mut SqliteConnection) -> Result<()> {
        info!("🏰 Creating default estate...");

        // Create default estate settings
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO estate_settings 
            (EstateID, EstateName, AbuseEmailToEstateOwner, DenyAnonymous, ResetHomeOnTeleport,
             FixedSun, DenyTransacted, BlockDwell, DenyIdentified, AllowVoice, UseGlobalTime,
             PricePerMeter, TaxFree, AllowDirectTeleport, RedirectGridX, RedirectGridY,
             ParentEstateID, SunPosition, EstateSkipScripts, BillableFactor, PublicAccess,
             AbuseEmail, EstateOwner, DenyMinors)
            VALUES 
            (1, 'Default Estate', 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0.0, 0, 1.0, 1,
             'abuse@localhost', '11111111-1111-0000-0000-000100bba000', 0)
        "#,
        )
        .execute(&mut *conn)
        .await?;

        info!("✅ Created default estate");
        Ok(())
    }
}

/// Default asset structure
#[derive(Debug)]
struct DefaultAsset {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    asset_type: i32,
    data: Vec<u8>,
    creator_id: &'static str,
}

pub fn create_ruth_shape_data() -> Vec<u8> {
    "LLWearable version 22\n\
New Shape\n\
\n\
\tpermissions 0\n\
\t{\n\
\t\tbase_mask\t7fffffff\n\
\t\towner_mask\t7fffffff\n\
\t\tgroup_mask\t00000000\n\
\t\teveryone_mask\t00000000\n\
\t\tnext_owner_mask\t00082000\n\
\t\tcreator_id\t11111111-1111-0000-0000-000100bba000\n\
\t\towner_id\t11111111-1111-0000-0000-000100bba000\n\
\t\tlast_owner_id\t00000000-0000-0000-0000-000000000000\n\
\t\tgroup_id\t00000000-0000-0000-0000-000000000000\n\
\t}\n\
\tsale_info\t0\n\
\t{\n\
\t\tsale_type\tnot\n\
\t\tsale_price\t10\n\
\t}\n\
type 0\n\
parameters 142\n\
1 0\n2 0\n4 0\n5 0\n6 0\n7 0\n8 0\n10 0\n11 0\n12 0\n\
13 0\n14 0\n15 0\n17 0\n18 0\n19 0\n20 0\n21 0\n22 0\n23 0\n\
24 0\n25 0\n26 0\n27 0\n28 0\n29 .12\n30 .12\n32 0\n33 0\n34 0\n\
35 0\n36 -.5\n37 0\n38 0\n40 0\n80 0\n100 0\n104 0\n105 .5\n106 0\n\
151 0\n152 0\n153 0\n155 0\n156 0\n157 0\n185 0\n186 0\n187 0\n188 0\n\
189 0\n193 .5\n194 .67\n195 .33\n196 0\n505 .5\n506 0\n507 0\n515 0\n517 0\n\
518 0\n626 0\n627 0\n629 .5\n630 0\n631 0\n633 0\n634 0\n635 0\n637 0\n\
646 0\n647 0\n648 0\n649 .5\n650 0\n651 0\n652 .5\n653 0\n655 -.08\n656 0\n\
657 0\n658 0\n659 .5\n660 0\n661 0\n662 .5\n663 0\n664 0\n665 0\n675 0\n\
676 0\n677 0\n678 .5\n679 -.08\n680 -.08\n681 -.08\n682 .5\n683 -.15\n684 0\n685 0\n\
686 0\n687 0\n688 0\n689 0\n690 .5\n691 0\n692 0\n693 .6\n694 -.08\n695 0\n\
753 0\n756 0\n758 0\n759 .5\n760 0\n764 0\n765 0\n767 0\n768 0\n769 .5\n\
770 0\n772 0\n773 .5\n794 .17\n795 .25\n796 0\n797 0\n798 0\n799 .5\n841 0\n\
842 0\n843 0\n853 0\n854 0\n855 0\n879 0\n880 0\n1103 0\n1104 0\n1105 0\n\
1200 0\n1201 0\n\
textures 0\n"
        .as_bytes()
        .to_vec()
}

pub fn create_ruth_skin_data() -> Vec<u8> {
    "LLWearable version 22\n\
Sexy - Female Skin\n\
\n\
\tpermissions 0\n\
\t{\n\
\t\tbase_mask\t00000000\n\
\t\towner_mask\t00000000\n\
\t\tgroup_mask\t00000000\n\
\t\teveryone_mask\t00000000\n\
\t\tnext_owner_mask\t00000000\n\
\t\tcreator_id\t11111111-1111-0000-0000-000100bba000\n\
\t\towner_id\t11111111-1111-0000-0000-000100bba000\n\
\t\tlast_owner_id\t11111111-1111-0000-0000-000100bba000\n\
\t\tgroup_id\t00000000-0000-0000-0000-000000000000\n\
\t}\n\
\tsale_info\t0\n\
\t{\n\
\t\tsale_type\tnot\n\
\t\tsale_price\t10\n\
\t}\n\
type 1\n\
parameters 26\n\
108 0\n110 0\n111 0\n116 0\n117 1\n150 0\n162 0\n163 0\n165 0\n700 .01\n\
701 .5\n702 .26\n703 0\n704 0\n705 .5\n706 .6\n707 0\n708 0\n709 0\n710 0\n\
711 .5\n712 0\n713 .7\n714 0\n715 0\n775 0\n\
textures 3\n\
0 00000000-0000-1111-9999-000000000012\n\
5 00000000-0000-1111-9999-000000000010\n\
6 00000000-0000-1111-9999-000000000011\n"
        .as_bytes()
        .to_vec()
}

pub fn create_ruth_hair_data() -> Vec<u8> {
    "LLWearable version 22\n\
New Hair\n\
\n\
\tpermissions 0\n\
\t{\n\
\t\tbase_mask\t7fffffff\n\
\t\towner_mask\t7fffffff\n\
\t\tgroup_mask\t00000000\n\
\t\teveryone_mask\t00000000\n\
\t\tnext_owner_mask\t00082000\n\
\t\tcreator_id\t11111111-1111-0000-0000-000100bba000\n\
\t\towner_id\t11111111-1111-0000-0000-000100bba000\n\
\t\tlast_owner_id\t00000000-0000-0000-0000-000000000000\n\
\t\tgroup_id\t00000000-0000-0000-0000-000000000000\n\
\t}\n\
\tsale_info\t0\n\
\t{\n\
\t\tsale_type\tnot\n\
\t\tsale_price\t10\n\
\t}\n\
type 2\n\
parameters 90\n\
16 0\n31 .5\n112 0\n113 0\n114 .5\n115 0\n119 .5\n130 .45\n131 .5\n132 .39\n\
133 .25\n134 .5\n135 .55\n136 .5\n137 .5\n140 0\n141 0\n142 0\n143 .12\n144 .1\n\
145 0\n146 0\n147 0\n148 .22\n149 0\n166 0\n167 0\n168 0\n169 0\n171 0\n\
172 .5\n173 0\n174 0\n175 .3\n176 0\n177 0\n178 0\n179 0\n180 .13\n181 .14\n\
182 .7\n183 .05\n184 0\n190 0\n191 0\n192 0\n400 .75\n640 0\n641 0\n642 0\n\
643 0\n644 0\n645 0\n674 -.3\n750 .7\n751 0\n752 .5\n754 0\n755 .05\n757 -1\n\
761 0\n762 0\n763 .55\n771 0\n774 0\n782 0\n783 0\n784 0\n785 0\n786 0\n\
787 0\n788 0\n789 0\n790 0\n870 -.29\n871 0\n872 .25\n1000 .5\n1001 .5\n1002 .7\n\
1003 .7\n1004 0\n1005 0\n1006 0\n1007 0\n1008 0\n1009 0\n1010 0\n1011 0\n1012 .25\n\
textures 1\n\
4 7ca39b4c-bd19-4699-aff7-f93fd03d3e7b\n"
        .as_bytes()
        .to_vec()
}

pub fn create_ruth_eyes_data() -> Vec<u8> {
    "LLWearable version 22\n\
New Eyes\n\
\n\
\tpermissions 0\n\
\t{\n\
\t\tbase_mask\t7fffffff\n\
\t\towner_mask\t7fffffff\n\
\t\tgroup_mask\t00000000\n\
\t\teveryone_mask\t00000000\n\
\t\tnext_owner_mask\t00082000\n\
\t\tcreator_id\t11111111-1111-0000-0000-000100bba000\n\
\t\towner_id\t11111111-1111-0000-0000-000100bba000\n\
\t\tlast_owner_id\t00000000-0000-0000-0000-000000000000\n\
\t\tgroup_id\t00000000-0000-0000-0000-000000000000\n\
\t}\n\
\tsale_info\t0\n\
\t{\n\
\t\tsale_type\tnot\n\
\t\tsale_price\t10\n\
\t}\n\
type 3\n\
parameters 2\n\
98 0\n99 0\n\
textures 1\n\
3 6522e74d-1660-4e7f-b601-6f48c1659a77\n"
        .as_bytes()
        .to_vec()
}

pub fn create_ruth_shirt_data() -> Vec<u8> {
    "LLWearable version 22\n\
New Shirt\n\
\n\
\tpermissions 0\n\
\t{\n\
\t\tbase_mask\t00000000\n\
\t\towner_mask\t00000000\n\
\t\tgroup_mask\t00000000\n\
\t\teveryone_mask\t00000000\n\
\t\tnext_owner_mask\t00000000\n\
\t\tcreator_id\t11111111-1111-0000-0000-000100bba000\n\
\t\towner_id\t11111111-1111-0000-0000-000100bba000\n\
\t\tlast_owner_id\t00000000-0000-0000-0000-000000000000\n\
\t\tgroup_id\t00000000-0000-0000-0000-000000000000\n\
\t}\n\
\tsale_info\t0\n\
\t{\n\
\t\tsale_type\tnot\n\
\t\tsale_price\t10\n\
\t}\n\
type 4\n\
parameters 10\n\
781 .78\n800 .65\n801 .82\n802 .78\n803 .5\n804 .5\n805 .6\n828 0\n840 0\n868 0\n\
textures 1\n\
1 5748decc-f629-461c-9a36-a35a221fe21f\n"
        .as_bytes()
        .to_vec()
}

pub fn create_ruth_pants_data() -> Vec<u8> {
    "LLWearable version 22\n\
New Pants\n\
\n\
\tpermissions 0\n\
\t{\n\
\t\tbase_mask\t00000000\n\
\t\towner_mask\t00000000\n\
\t\tgroup_mask\t00000000\n\
\t\teveryone_mask\t00000000\n\
\t\tnext_owner_mask\t00000000\n\
\t\tcreator_id\t11111111-1111-0000-0000-000100bba000\n\
\t\towner_id\t11111111-1111-0000-0000-000100bba000\n\
\t\tlast_owner_id\t00000000-0000-0000-0000-000000000000\n\
\t\tgroup_id\t00000000-0000-0000-0000-000000000000\n\
\t}\n\
\tsale_info\t0\n\
\t{\n\
\t\tsale_type\tnot\n\
\t\tsale_price\t10\n\
\t}\n\
type 5\n\
parameters 9\n\
625 0\n638 0\n806 .8\n807 .2\n808 .2\n814 1\n815 .8\n816 0\n869 0\n\
textures 1\n\
2 5748decc-f629-461c-9a36-a35a221fe21f\n"
        .as_bytes()
        .to_vec()
}

pub fn create_default_j2k_texture() -> Vec<u8> {
    vec![
        0xFF, 0x4F, // SOC - Start of codestream
        // SIZ marker segment
        0xFF, 0x51, 0x00, 0x29, // marker + length (41)
        0x00, 0x00, // Rsiz
        0x00, 0x00, 0x00, 0x04, // Xsiz = 4
        0x00, 0x00, 0x00, 0x04, // Ysiz = 4
        0x00, 0x00, 0x00, 0x00, // XOsiz
        0x00, 0x00, 0x00, 0x00, // YOsiz
        0x00, 0x00, 0x00, 0x04, // XTsiz
        0x00, 0x00, 0x00, 0x04, // YTsiz
        0x00, 0x00, 0x00, 0x00, // XTOsiz
        0x00, 0x00, 0x00, 0x00, // YTOsiz
        0x00, 0x03, // Csiz = 3 (RGB)
        0x07, 0x01, 0x01, // Component 0: depth=8, XRsiz=1, YRsiz=1
        0x07, 0x01, 0x01, // Component 1
        0x07, 0x01, 0x01, // Component 2
        // COD marker segment
        0xFF, 0x52, 0x00, 0x0C, // marker + length (12)
        0x00, // Scod
        0x00, // SGcod: progression LRCP
        0x00, 0x01, // layers = 1
        0x01, // MCT
        0x00, // decomposition levels = 0
        0x02, // xcb = 4
        0x02, // ycb = 4
        0x00, // code-block style
        // SOT marker (Start of tile)
        0xFF, 0x90, 0x00, 0x0A, // marker + length (10)
        0x00, 0x00, // Isot = 0
        0x00, 0x00, 0x00, 0x00, // Psot = 0 (rest of stream)
        0x00, // TPsot
        0x01, // TNsot
        // SOD marker (Start of data)
        0xFF, 0x93, // Minimal packet: empty header + empty body (all white)
        0x00, // EOC - End of codestream
        0xFF, 0xD9,
    ]
}

pub fn load_ruth_head_texture() -> Vec<u8> {
    load_ruth_texture_file("female face.j2c")
}

pub fn load_ruth_upper_texture() -> Vec<u8> {
    load_ruth_texture_file("female body.j2c")
}

pub fn load_ruth_lower_texture() -> Vec<u8> {
    load_ruth_texture_file("female bottom.j2c")
}

pub fn load_ruth_eyes_texture() -> Vec<u8> {
    load_ruth_texture_file("eyes.j2c")
}

pub fn load_ruth_hair_texture() -> Vec<u8> {
    load_ruth_texture_file("open sim hair base.j2c")
}

fn load_ruth_texture_file(filename: &str) -> Vec<u8> {
    let paths_to_try = [
        format!("assets/ruth/{}", filename),
        format!("opensim-next/assets/ruth/{}", filename),
        format!("../assets/ruth/{}", filename),
    ];

    for path in &paths_to_try {
        if let Ok(data) = std::fs::read(path) {
            info!("Loaded Ruth texture {} ({} bytes)", filename, data.len());
            return data;
        }
    }

    warn!(
        "Could not load Ruth texture {}, using placeholder",
        filename
    );
    create_default_j2k_texture()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_database_initialization() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let initializer = DatabaseInitializer::new(pool);

        // This should complete without errors
        initializer.initialize().await.unwrap();
    }
}
