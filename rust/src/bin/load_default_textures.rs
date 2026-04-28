use anyhow::Result;
use sqlx::PgPool;
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://opensim:password@localhost/opensim".to_string());

    let pool = PgPool::connect(&database_url).await?;

    let assets_dir = Path::new("./assets/assets/TexturesAssetSet");

    let textures = vec![
        (
            "c228d1cf-4b5d-4ba8-84f4-899a0796aa97",
            "default_avatar.jp2",
            "Default Avatar",
        ),
        (
            "6522e74d-1660-4e7f-b601-6f48c1659a77",
            "default_iris.jp2",
            "Default Iris",
        ),
        (
            "5a9f4a74-30f2-821c-b88d-70499d3e7183",
            "FALLBACK_BAKED_HEAD_32X32.j2k",
            "Fallback Baked Head",
        ),
        (
            "ae2de45c-d252-50b8-5c6e-19f39ce79317",
            "FALLBACK_BAKED_UPPER_32X32.j2k",
            "Fallback Baked Upper",
        ),
        (
            "24daea5f-0539-cfcf-047f-fbc40b2786ba",
            "FALLBACK_BAKED_LOWER_32X32.j2k",
            "Fallback Baked Lower",
        ),
        (
            "5748decc-f629-461c-9a36-a35a221fe21f",
            "FALLBACK_BAKED_EYES_32X32.j2k",
            "Fallback Baked Eyes",
        ),
        (
            "89556747-24cb-43ed-920b-47caed15465f",
            "FALLBACK_BAKED_HAIR_32X32.j2k",
            "Fallback Baked Hair",
        ),
        (
            "abb783e6-3e93-26c0-248a-247666855da3",
            "Terrain Grass-abb783e6-3e93-26c0-248a-247666855da3.texture",
            "Terrain Grass",
        ),
        (
            "b8d3965a-ad78-bf43-699b-bff8eca6c975",
            "Terrain Dirt-b8d3965a-ad78-bf43-699b-bff8eca6c975.texture",
            "Terrain Dirt",
        ),
        (
            "179cdabd-398a-9b6b-1391-4dc333ba321f",
            "Terrain Mountain-179cdabd-398a-9b6b-1391-4dc333ba321f.texture",
            "Terrain Mountain",
        ),
        (
            "beb169c7-11ea-fff2-efe5-0f24dc881df2",
            "Terrain Rock-beb169c7-11ea-fff2-efe5-0f24dc881df2.texture",
            "Terrain Rock",
        ),
        (
            "058c75c0-a0d5-f2f8-43f3-e9699a89c2fc",
            "058c75c0-a0d5-f2f8-43f3-e9699a89c2fc.j2c",
            "UI Texture",
        ),
        (
            "073c9723-540c-5449-cdd4-0e87fdc159e3",
            "073c9723-540c-5449-cdd4-0e87fdc159e3.j2c",
            "UI Texture",
        ),
        (
            "11b4c57c-56b3-04ed-1f82-2004363882e4",
            "11b4c57c-56b3-04ed-1f82-2004363882e4.j2c",
            "UI Texture",
        ),
        (
            "12149143-f599-91a7-77ac-b52a3c0f59cd",
            "12149143-f599-91a7-77ac-b52a3c0f59cd.j2c",
            "UI Texture",
        ),
        (
            "2bfd3884-7e27-69b9-ba3a-3e673f680004",
            "2bfd3884-7e27-69b9-ba3a-3e673f680004.j2c",
            "UI Texture",
        ),
        (
            "43c32285-d658-1793-c123-bf86315de055",
            "43c32285-d658-1793-c123-bf86315de055.j2c",
            "UI Texture",
        ),
        (
            "4726f13e-bd07-f2fb-feb0-bfa2ac58ab61",
            "4726f13e-bd07-f2fb-feb0-bfa2ac58ab61.j2c",
            "UI Texture",
        ),
        (
            "6c9fa78a-1c69-2168-325b-3e03ffa348ce",
            "6c9fa78a-1c69-2168-325b-3e03ffa348ce.j2c",
            "UI Texture",
        ),
        (
            "735198cf-6ea0-2550-e222-21d3c6a341ae",
            "735198cf-6ea0-2550-e222-21d3c6a341ae.j2c",
            "UI Texture",
        ),
        (
            "822ded49-9a6c-f61c-cb89-6df54f42cdf4",
            "822ded49-9a6c-f61c-cb89-6df54f42cdf4.j2c",
            "UI Texture",
        ),
        (
            "83b77fc6-10b4-63ec-4de7-f40629f238c5",
            "83b77fc6-10b4-63ec-4de7-f40629f238c5.j2c",
            "UI Texture",
        ),
        (
            "92e66e00-f56f-598a-7997-048aa64cde18",
            "92e66e00-f56f-598a-7997-048aa64cde18.j2c",
            "UI Texture",
        ),
        (
            "9deab416-9c63-78d6-d558-9a156f12044c",
            "9deab416-9c63-78d6-d558-9a156f12044c.j2c",
            "UI Texture",
        ),
        (
            "ae874d1a-93ef-54fb-5fd3-eb0cb156afc0",
            "ae874d1a-93ef-54fb-5fd3-eb0cb156afc0.j2c",
            "UI Texture",
        ),
        (
            "b4ba225c-373f-446d-9f7e-6cb7b5cf9b3d",
            "b4ba225c-373f-446d-9f7e-6cb7b5cf9b3d.j2c",
            "UI Texture",
        ),
        (
            "b8eed5f0-64b7-6e12-b67f-43fa8e773440",
            "b8eed5f0-64b7-6e12-b67f-43fa8e773440.j2c",
            "UI Texture",
        ),
        (
            "d21e44ca-ff1c-a96e-b2ef-c0753426b7d9",
            "d21e44ca-ff1c-a96e-b2ef-c0753426b7d9.j2c",
            "UI Texture",
        ),
        (
            "d9258671-868f-7511-c321-7baef9e948a4",
            "d9258671-868f-7511-c321-7baef9e948a4.j2c",
            "UI Texture",
        ),
        (
            "db9d39ec-a896-c287-1ced-64566217021e",
            "db9d39ec-a896-c287-1ced-64566217021e.j2c",
            "UI Texture",
        ),
        (
            "e569711a-27c2-aad4-9246-0c910239a179",
            "e569711a-27c2-aad4-9246-0c910239a179.j2c",
            "UI Texture",
        ),
        (
            "f2d7b6f6-4200-1e9a-fd5b-96459e950f94",
            "f2d7b6f6-4200-1e9a-fd5b-96459e950f94.j2c",
            "UI Texture",
        ),
    ];

    for (uuid_str, filename, name) in textures {
        let asset_id = Uuid::parse_str(uuid_str)?;
        let file_path = assets_dir.join(filename);

        if !file_path.exists() {
            println!("⚠️  File not found: {}", filename);
            continue;
        }

        let data = fs::read(&file_path)?;

        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM assets WHERE id = $1)")
            .bind(asset_id)
            .fetch_one(&pool)
            .await?;

        if exists {
            println!("✓ Texture already exists: {} ({})", name, uuid_str);
            continue;
        }

        sqlx::query(
            r#"
            INSERT INTO assets (id, name, description, asset_type, local, temporary, data, creator_id, asset_flags, size_bytes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(asset_id)
        .bind(name)
        .bind(if name.starts_with("Terrain") {
            format!("Default terrain texture: {}", name)
        } else if name == "UI Texture" {
            format!("Default UI texture: {}", uuid_str)
        } else {
            format!("Default avatar texture: {}", name)
        })
        .bind(0) // asset_type 0 = texture
        .bind(true) // local
        .bind(false) // temporary
        .bind(&data)
        .bind(Uuid::nil()) // creator_id
        .bind(0) // asset_flags
        .bind(data.len() as i64) // size_bytes
        .execute(&pool)
        .await?;

        println!(
            "✅ Loaded texture: {} ({}) - {} bytes",
            name,
            uuid_str,
            data.len()
        );
    }

    println!("\n🎉 Default textures loaded successfully!");

    Ok(())
}
