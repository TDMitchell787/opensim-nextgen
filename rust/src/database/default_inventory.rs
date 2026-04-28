use anyhow::Result;
use sqlx::PgPool;
use tracing::{debug, info, warn};
use uuid::Uuid;

const ASSET_SHAPE: &str = "66c41e39-38f9-f75a-024e-585989bfab73";
const ASSET_SKIN: &str = "77c41e39-38f9-f75a-024e-585989bbabbb";
const ASSET_HAIR: &str = "d342e6c0-b9d2-11dc-95ff-0800200c9a66";
const ASSET_EYES: &str = "4bb6fa4d-1cd2-498a-a84c-95c1a0e745a7";
const ASSET_SHIRT: &str = "00000000-38f9-1111-024e-222222111110";
const ASSET_PANTS: &str = "00000000-38f9-1111-024e-222222111120";

const PERM_ALL: i32 = 0x7FFFFFFF;
const PERM_COPY: i32 = 0x00008000;

const ASSET_TYPE_BODYPART: i32 = 13;
const ASSET_TYPE_CLOTHING: i32 = 5;
const ASSET_TYPE_LINK: i32 = 24;
const INV_TYPE_WEARABLE: i32 = 18;

const DEFAULT_VISUAL_PARAMS: &str = "33,61,85,23,58,127,63,85,63,42,0,85,63,36,85,95,153,63,34,0,63,109,88,132,63,136,81,85,103,136,127,0,150,150,150,127,0,0,0,0,0,127,0,0,255,127,114,127,99,63,127,140,127,127,0,0,0,191,0,104,0,0,0,0,0,0,0,0,0,145,216,133,0,127,0,127,170,0,0,127,127,109,85,127,127,63,85,42,150,150,150,150,150,150,150,25,150,150,150,0,127,0,0,144,85,127,132,127,85,0,127,127,127,127,127,127,59,127,85,127,127,106,47,79,127,127,204,2,141,66,0,0,127,127,0,0,0,0,127,0,159,0,0,178,127,36,85,131,127,127,127,153,95,0,140,75,27,127,127,0,150,150,198,0,0,63,30,127,165,209,198,127,127,153,204,51,51,255,255,255,204,0,255,150,150,150,150,150,150,150,150,150,150,0,150,150,150,150,150,0,127,127,150,150,150,150,150,150,150,150,0,0,150,51,132,150,150,150";

pub async fn create_default_user_inventory(pool: &PgPool, user_id: Uuid) -> Result<()> {
    info!(
        "Creating default inventory and appearance for user {}",
        user_id
    );

    let created_timestamp = chrono::Utc::now().timestamp() as i32;
    let user_str = user_id.to_string();

    let root_folder_id = Uuid::new_v4();
    let body_parts_folder_id = Uuid::new_v4();
    let clothing_folder_id = Uuid::new_v4();
    let cof_folder_id = Uuid::new_v4();

    let folders: Vec<(Uuid, Uuid, &str, i32)> = vec![
        (root_folder_id, Uuid::nil(), "My Inventory", 8),
        (Uuid::new_v4(), root_folder_id, "Textures", 0),
        (Uuid::new_v4(), root_folder_id, "Sounds", 1),
        (Uuid::new_v4(), root_folder_id, "Calling Cards", 2),
        (Uuid::new_v4(), root_folder_id, "Landmarks", 3),
        (clothing_folder_id, root_folder_id, "Clothing", 5),
        (Uuid::new_v4(), root_folder_id, "Objects", 6),
        (Uuid::new_v4(), root_folder_id, "Notecards", 7),
        (Uuid::new_v4(), root_folder_id, "Scripts", 10),
        (body_parts_folder_id, root_folder_id, "Body Parts", 13),
        (Uuid::new_v4(), root_folder_id, "Trash", 14),
        (Uuid::new_v4(), root_folder_id, "Photo Album", 15),
        (Uuid::new_v4(), root_folder_id, "Lost And Found", 16),
        (Uuid::new_v4(), root_folder_id, "Animations", 20),
        (Uuid::new_v4(), root_folder_id, "Gestures", 21),
        (Uuid::new_v4(), root_folder_id, "Favorites", 23),
        (cof_folder_id, root_folder_id, "Current Outfit", 46),
        (Uuid::new_v4(), root_folder_id, "My Outfits", 48),
        (Uuid::new_v4(), root_folder_id, "Settings", 56),
        (Uuid::new_v4(), root_folder_id, "Materials", 57),
        (Uuid::new_v4(), root_folder_id, "My Suitcase", 100),
    ];

    let folder_query = r#"
        INSERT INTO inventoryfolders (folderid, agentid, parentfolderid, foldername, type, version)
        VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, 1)
        ON CONFLICT (folderid) DO NOTHING
    "#;

    let mut folder_count = 0;
    for (folder_id, parent_id, name, folder_type) in &folders {
        match sqlx::query(folder_query)
            .bind(folder_id.to_string())
            .bind(&user_str)
            .bind(parent_id.to_string())
            .bind(*name)
            .bind(*folder_type)
            .execute(pool)
            .await
        {
            Ok(_) => folder_count += 1,
            Err(e) => warn!("Failed to create folder '{}': {}", name, e),
        }
    }

    let item_shape_id = Uuid::new_v4();
    let item_skin_id = Uuid::new_v4();
    let item_hair_id = Uuid::new_v4();
    let item_eyes_id = Uuid::new_v4();
    let item_shirt_id = Uuid::new_v4();
    let item_pants_id = Uuid::new_v4();

    let wearables: Vec<(Uuid, &str, Uuid, &str, i32, i32, i32)> = vec![
        (
            item_shape_id,
            ASSET_SHAPE,
            body_parts_folder_id,
            "Default Shape",
            ASSET_TYPE_BODYPART,
            INV_TYPE_WEARABLE,
            0,
        ),
        (
            item_skin_id,
            ASSET_SKIN,
            body_parts_folder_id,
            "Default Skin",
            ASSET_TYPE_BODYPART,
            INV_TYPE_WEARABLE,
            1,
        ),
        (
            item_hair_id,
            ASSET_HAIR,
            body_parts_folder_id,
            "Default Hair",
            ASSET_TYPE_BODYPART,
            INV_TYPE_WEARABLE,
            2,
        ),
        (
            item_eyes_id,
            ASSET_EYES,
            body_parts_folder_id,
            "Default Eyes",
            ASSET_TYPE_BODYPART,
            INV_TYPE_WEARABLE,
            3,
        ),
        (
            item_shirt_id,
            ASSET_SHIRT,
            clothing_folder_id,
            "Default Shirt",
            ASSET_TYPE_CLOTHING,
            INV_TYPE_WEARABLE,
            4,
        ),
        (
            item_pants_id,
            ASSET_PANTS,
            clothing_folder_id,
            "Default Pants",
            ASSET_TYPE_CLOTHING,
            INV_TYPE_WEARABLE,
            5,
        ),
    ];

    let item_query = r#"
        INSERT INTO inventoryitems
        (inventoryid, assetid, assettype, parentfolderid, avatarid, inventoryname, inventorydescription,
         inventorynextpermissions, inventorycurrentpermissions, invtype, creatorid,
         inventorybasepermissions, inventoryeveryonepermissions, inventorygrouppermissions,
         saleprice, saletype, creationdate, groupid, groupowned, flags)
        VALUES ($1::uuid, $2::uuid, $3, $4::uuid, $5::uuid, $6, '',
                $7, $7, $8, $9,
                $7, 0, 0,
                0, 0, $10, '00000000-0000-0000-0000-000000000000'::uuid, 0, $11)
        ON CONFLICT (inventoryid) DO NOTHING
    "#;

    let mut item_count = 0;
    for (item_id, asset_uuid, folder_id, name, asset_type, inv_type, wearable_type) in &wearables {
        match sqlx::query(item_query)
            .bind(item_id.to_string())
            .bind(*asset_uuid)
            .bind(*asset_type)
            .bind(folder_id.to_string())
            .bind(&user_str)
            .bind(*name)
            .bind(PERM_ALL)
            .bind(*inv_type)
            .bind(&user_str)
            .bind(created_timestamp)
            .bind(*wearable_type)
            .execute(pool)
            .await
        {
            Ok(_) => item_count += 1,
            Err(e) => warn!("Failed to create wearable '{}': {}", name, e),
        }
    }

    let link_query = r#"
        INSERT INTO inventoryitems
        (inventoryid, assetid, assettype, parentfolderid, avatarid, inventoryname, inventorydescription,
         inventorynextpermissions, inventorycurrentpermissions, invtype, creatorid,
         inventorybasepermissions, inventoryeveryonepermissions, inventorygrouppermissions,
         saleprice, saletype, creationdate, groupid, groupowned, flags)
        VALUES ($1::uuid, $2::uuid, 24, $3::uuid, $4::uuid, $5, '',
                $6, $6, 18, $7,
                $6, 0, 0,
                0, 0, $8, '00000000-0000-0000-0000-000000000000'::uuid, 0, $9)
        ON CONFLICT (inventoryid) DO NOTHING
    "#;

    let cof_links: Vec<(Uuid, &str, i32)> = vec![
        (item_shape_id, "Default Shape", 0),
        (item_skin_id, "Default Skin", 1),
        (item_hair_id, "Default Hair", 2),
        (item_eyes_id, "Default Eyes", 3),
        (item_shirt_id, "Default Shirt", 4),
        (item_pants_id, "Default Pants", 5),
    ];

    let mut link_count = 0;
    for (target_item_id, name, wearable_type) in &cof_links {
        match sqlx::query(link_query)
            .bind(Uuid::new_v4().to_string())
            .bind(target_item_id.to_string())
            .bind(cof_folder_id.to_string())
            .bind(&user_str)
            .bind(*name)
            .bind(PERM_COPY)
            .bind(&user_str)
            .bind(created_timestamp)
            .bind(*wearable_type)
            .execute(pool)
            .await
        {
            Ok(_) => link_count += 1,
            Err(e) => warn!("Failed to create COF link '{}': {}", name, e),
        }
    }

    let vp_hex: String = DEFAULT_VISUAL_PARAMS
        .split(',')
        .map(|v| format!("{:02x}", v.trim().parse::<u8>().unwrap_or(127)))
        .collect();

    let avatar_entries: Vec<(&str, String)> = vec![
        ("AvatarType", "1".to_string()),
        ("Serial", "0".to_string()),
        ("AvatarHeight", "1.771488".to_string()),
        ("VisualParams", vp_hex),
        ("BodyItem", item_shape_id.to_string()),
        ("BodyAsset", ASSET_SHAPE.to_string()),
        ("SkinItem", item_skin_id.to_string()),
        ("SkinAsset", ASSET_SKIN.to_string()),
        ("HairItem", item_hair_id.to_string()),
        ("HairAsset", ASSET_HAIR.to_string()),
        ("EyesItem", item_eyes_id.to_string()),
        ("EyesAsset", ASSET_EYES.to_string()),
        ("Wearable 0", format!("{}:{}", item_shape_id, ASSET_SHAPE)),
        ("Wearable 0:0", format!("{}:{}", item_shape_id, ASSET_SHAPE)),
        ("Wearable 1", format!("{}:{}", item_skin_id, ASSET_SKIN)),
        ("Wearable 1:0", format!("{}:{}", item_skin_id, ASSET_SKIN)),
        ("Wearable 2", format!("{}:{}", item_hair_id, ASSET_HAIR)),
        ("Wearable 2:0", format!("{}:{}", item_hair_id, ASSET_HAIR)),
        ("Wearable 3", format!("{}:{}", item_eyes_id, ASSET_EYES)),
        ("Wearable 3:0", format!("{}:{}", item_eyes_id, ASSET_EYES)),
        ("Wearable 4", format!("{}:{}", item_shirt_id, ASSET_SHIRT)),
        ("Wearable 4:0", format!("{}:{}", item_shirt_id, ASSET_SHIRT)),
        ("Wearable 5", format!("{}:{}", item_pants_id, ASSET_PANTS)),
        ("Wearable 5:0", format!("{}:{}", item_pants_id, ASSET_PANTS)),
    ];

    let avatar_query = r#"
        INSERT INTO avatars (principalid, name, value)
        VALUES ($1::uuid, $2, $3)
        ON CONFLICT DO NOTHING
    "#;

    let mut appearance_count = 0;
    for (name, value) in &avatar_entries {
        match sqlx::query(avatar_query)
            .bind(&user_str)
            .bind(*name)
            .bind(value)
            .execute(pool)
            .await
        {
            Ok(_) => appearance_count += 1,
            Err(e) => warn!("Failed to create avatar entry '{}': {}", name, e),
        }
    }

    info!("Default inventory created for {}: {} folders, {} items, {} COF links, {} appearance entries",
        user_id, folder_count, item_count, link_count, appearance_count);

    Ok(())
}

pub async fn ensure_user_has_inventory(pool: &PgPool, user_id: Uuid) -> Result<bool> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT COUNT(*) FROM inventoryfolders WHERE agentid = $1::uuid")
            .bind(user_id.to_string())
            .fetch_optional(pool)
            .await?;

    let folder_count = row.map(|(c,)| c).unwrap_or(0);

    if folder_count == 0 {
        info!(
            "User {} has no inventory folders — creating defaults",
            user_id
        );
        create_default_user_inventory(pool, user_id).await?;
        return Ok(true);
    }

    let mut changed = false;

    changed |= ensure_missing_system_folders(pool, user_id)
        .await
        .unwrap_or(false);

    let bp_row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM inventoryitems WHERE avatarid = $1::uuid AND assettype = 13",
    )
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await?;

    let bp_count = bp_row.map(|(c,)| c).unwrap_or(0);

    if bp_count == 0 {
        info!("User {} has no body parts — creating defaults", user_id);
        create_default_bodyparts_only(pool, user_id).await?;
        changed = true;
    }

    let suitcase_row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM inventoryfolders WHERE agentid = $1::uuid AND type = 100",
    )
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await?;

    if suitcase_row.map(|(c,)| c).unwrap_or(0) == 0 {
        let root_row: Option<(String,)> = sqlx::query_as(
            "SELECT folderid::text FROM inventoryfolders WHERE agentid = $1::uuid AND type = 8 LIMIT 1"
        )
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await?;

        if let Some((root_id,)) = root_row {
            info!("User {} has no suitcase folder — creating one", user_id);
            let _ = sqlx::query(
                "INSERT INTO inventoryfolders (folderid, agentid, parentfolderid, foldername, type, version) VALUES ($1::uuid, $2::uuid, $3::uuid, 'My Suitcase', 100, 1) ON CONFLICT (folderid) DO NOTHING"
            )
            .bind(Uuid::new_v4().to_string())
            .bind(user_id.to_string())
            .bind(&root_id)
            .execute(pool)
            .await;
            changed = true;
        }
    }

    Ok(changed)
}

async fn ensure_missing_system_folders(pool: &PgPool, user_id: Uuid) -> Result<bool> {
    let user_str = user_id.to_string();

    let root_row: Option<(String,)> = sqlx::query_as(
        "SELECT folderid::text FROM inventoryfolders WHERE agentid = $1::uuid AND type = 8 LIMIT 1",
    )
    .bind(&user_str)
    .fetch_optional(pool)
    .await?;

    let root_id = match root_row {
        Some((id,)) => id,
        None => return Ok(false),
    };

    use sqlx::Row;
    let existing_types: Vec<i32> =
        sqlx::query("SELECT DISTINCT type FROM inventoryfolders WHERE agentid = $1::uuid")
            .bind(&user_str)
            .fetch_all(pool)
            .await?
            .iter()
            .filter_map(|row| row.try_get::<i32, _>("type").ok())
            .collect();

    let required_folders: Vec<(&str, i32)> = vec![
        ("Textures", 0),
        ("Sounds", 1),
        ("Calling Cards", 2),
        ("Landmarks", 3),
        ("Clothing", 5),
        ("Objects", 6),
        ("Notecards", 7),
        ("Scripts", 10),
        ("Body Parts", 13),
        ("Trash", 14),
        ("Photo Album", 15),
        ("Lost And Found", 16),
        ("Animations", 20),
        ("Gestures", 21),
        ("Favorites", 23),
        ("Current Outfit", 46),
        ("My Outfits", 48),
        ("Received Items", 49),
        ("Settings", 56),
        ("Materials", 57),
    ];

    let mut created = 0;
    for (name, folder_type) in &required_folders {
        if existing_types.contains(folder_type) {
            continue;
        }
        let folder_id = Uuid::new_v4();
        match sqlx::query(
            "INSERT INTO inventoryfolders (folderid, agentid, parentfolderid, foldername, type, version) VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, 1) ON CONFLICT (folderid) DO NOTHING"
        )
        .bind(folder_id.to_string())
        .bind(&user_str)
        .bind(&root_id)
        .bind(*name)
        .bind(*folder_type)
        .execute(pool)
        .await
        {
            Ok(_) => {
                info!("Created missing system folder '{}' (type {}) for user {}", name, folder_type, user_id);
                created += 1;
            }
            Err(e) => warn!("Failed to create system folder '{}': {}", name, e),
        }
    }

    if created > 0 {
        info!(
            "Created {} missing system folders for user {}",
            created, user_id
        );
    }

    Ok(created > 0)
}

async fn create_default_bodyparts_only(pool: &PgPool, user_id: Uuid) -> Result<()> {
    let user_str = user_id.to_string();
    let created_timestamp = chrono::Utc::now().timestamp() as i32;

    let bp_folder: Option<(String,)> = sqlx::query_as(
        "SELECT folderid::text FROM inventoryfolders WHERE agentid = $1::uuid AND type = 13 LIMIT 1"
    )
    .bind(&user_str)
    .fetch_optional(pool)
    .await?;

    let body_parts_folder_id = match bp_folder {
        Some((id,)) => id.parse::<Uuid>().unwrap_or_else(|_| Uuid::new_v4()),
        None => return Ok(()),
    };

    let clothing_folder: Option<(String,)> = sqlx::query_as(
        "SELECT folderid::text FROM inventoryfolders WHERE agentid = $1::uuid AND type = 5 LIMIT 1",
    )
    .bind(&user_str)
    .fetch_optional(pool)
    .await?;

    let clothing_folder_id = match clothing_folder {
        Some((id,)) => id.parse::<Uuid>().unwrap_or_else(|_| Uuid::new_v4()),
        None => body_parts_folder_id,
    };

    let cof_folder: Option<(String,)> = sqlx::query_as(
        "SELECT folderid::text FROM inventoryfolders WHERE agentid = $1::uuid AND type = 46 LIMIT 1"
    )
    .bind(&user_str)
    .fetch_optional(pool)
    .await?;

    let cof_folder_id = match cof_folder {
        Some((id,)) => id.parse::<Uuid>().unwrap_or_else(|_| Uuid::new_v4()),
        None => return Ok(()),
    };

    let item_shape_id = Uuid::new_v4();
    let item_skin_id = Uuid::new_v4();
    let item_hair_id = Uuid::new_v4();
    let item_eyes_id = Uuid::new_v4();
    let item_shirt_id = Uuid::new_v4();
    let item_pants_id = Uuid::new_v4();

    let item_query = r#"
        INSERT INTO inventoryitems
        (inventoryid, assetid, assettype, parentfolderid, avatarid, inventoryname, inventorydescription,
         inventorynextpermissions, inventorycurrentpermissions, invtype, creatorid,
         inventorybasepermissions, inventoryeveryonepermissions, inventorygrouppermissions,
         saleprice, saletype, creationdate, groupid, groupowned, flags)
        VALUES ($1::uuid, $2::uuid, $3, $4::uuid, $5::uuid, $6, '',
                $7, $7, 18, $8,
                $7, 0, 0,
                0, 0, $9, '00000000-0000-0000-0000-000000000000'::uuid, 0, $10)
        ON CONFLICT (inventoryid) DO NOTHING
    "#;

    let items: Vec<(Uuid, &str, Uuid, &str, i32, i32)> = vec![
        (
            item_shape_id,
            ASSET_SHAPE,
            body_parts_folder_id,
            "Default Shape",
            ASSET_TYPE_BODYPART,
            0,
        ),
        (
            item_skin_id,
            ASSET_SKIN,
            body_parts_folder_id,
            "Default Skin",
            ASSET_TYPE_BODYPART,
            1,
        ),
        (
            item_hair_id,
            ASSET_HAIR,
            body_parts_folder_id,
            "Default Hair",
            ASSET_TYPE_BODYPART,
            2,
        ),
        (
            item_eyes_id,
            ASSET_EYES,
            body_parts_folder_id,
            "Default Eyes",
            ASSET_TYPE_BODYPART,
            3,
        ),
        (
            item_shirt_id,
            ASSET_SHIRT,
            clothing_folder_id,
            "Default Shirt",
            ASSET_TYPE_CLOTHING,
            4,
        ),
        (
            item_pants_id,
            ASSET_PANTS,
            clothing_folder_id,
            "Default Pants",
            ASSET_TYPE_CLOTHING,
            5,
        ),
    ];

    for (item_id, asset_uuid, folder_id, name, asset_type, wearable_type) in &items {
        let _ = sqlx::query(item_query)
            .bind(item_id.to_string())
            .bind(*asset_uuid)
            .bind(*asset_type)
            .bind(folder_id.to_string())
            .bind(&user_str)
            .bind(*name)
            .bind(PERM_ALL)
            .bind(&user_str)
            .bind(created_timestamp)
            .bind(*wearable_type)
            .execute(pool)
            .await;
    }

    let link_query = r#"
        INSERT INTO inventoryitems
        (inventoryid, assetid, assettype, parentfolderid, avatarid, inventoryname, inventorydescription,
         inventorynextpermissions, inventorycurrentpermissions, invtype, creatorid,
         inventorybasepermissions, inventoryeveryonepermissions, inventorygrouppermissions,
         saleprice, saletype, creationdate, groupid, groupowned, flags)
        VALUES ($1::uuid, $2::uuid, 24, $3::uuid, $4::uuid, $5, '',
                $6, $6, 18, $7,
                $6, 0, 0,
                0, 0, $8, '00000000-0000-0000-0000-000000000000'::uuid, 0, $9)
        ON CONFLICT (inventoryid) DO NOTHING
    "#;

    let cof_links: Vec<(Uuid, &str, i32)> = vec![
        (item_shape_id, "Default Shape", 0),
        (item_skin_id, "Default Skin", 1),
        (item_hair_id, "Default Hair", 2),
        (item_eyes_id, "Default Eyes", 3),
        (item_shirt_id, "Default Shirt", 4),
        (item_pants_id, "Default Pants", 5),
    ];

    for (target_item_id, name, wearable_type) in &cof_links {
        let _ = sqlx::query(link_query)
            .bind(Uuid::new_v4().to_string())
            .bind(target_item_id.to_string())
            .bind(cof_folder_id.to_string())
            .bind(&user_str)
            .bind(*name)
            .bind(PERM_COPY)
            .bind(&user_str)
            .bind(created_timestamp)
            .bind(*wearable_type)
            .execute(pool)
            .await;
    }

    let avatar_query = r#"
        INSERT INTO avatars (principalid, name, value)
        VALUES ($1::uuid, $2, $3)
        ON CONFLICT DO NOTHING
    "#;

    let avatar_entries: Vec<(&str, String)> = vec![
        ("BodyItem", item_shape_id.to_string()),
        ("BodyAsset", ASSET_SHAPE.to_string()),
        ("SkinItem", item_skin_id.to_string()),
        ("SkinAsset", ASSET_SKIN.to_string()),
        ("HairItem", item_hair_id.to_string()),
        ("HairAsset", ASSET_HAIR.to_string()),
        ("EyesItem", item_eyes_id.to_string()),
        ("EyesAsset", ASSET_EYES.to_string()),
        ("Wearable 0:0", format!("{}:{}", item_shape_id, ASSET_SHAPE)),
        ("Wearable 1:0", format!("{}:{}", item_skin_id, ASSET_SKIN)),
        ("Wearable 2:0", format!("{}:{}", item_hair_id, ASSET_HAIR)),
        ("Wearable 3:0", format!("{}:{}", item_eyes_id, ASSET_EYES)),
        ("Wearable 4:0", format!("{}:{}", item_shirt_id, ASSET_SHIRT)),
        ("Wearable 5:0", format!("{}:{}", item_pants_id, ASSET_PANTS)),
    ];

    for (name, value) in &avatar_entries {
        let _ = sqlx::query(avatar_query)
            .bind(&user_str)
            .bind(*name)
            .bind(value)
            .execute(pool)
            .await;
    }

    info!(
        "Created default body parts, clothing, COF links, and appearance for user {}",
        user_id
    );
    Ok(())
}
