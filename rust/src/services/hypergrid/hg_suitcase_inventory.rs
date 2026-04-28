use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::services::traits::{
    InventoryCollection, InventoryFolder, InventoryItem, InventoryServiceTrait,
};

const SUITCASE_FOLDER_TYPE: i32 = 100;

const SUITCASE_SYSTEM_FOLDERS: &[(i32, &str)] = &[
    (20, "Animations"),
    (13, "Body Parts"),
    (5, "Clothing"),
    (46, "Current Outfit"),
    (23, "Favorites"),
    (21, "Gestures"),
    (3, "Landmarks"),
    (16, "Lost And Found"),
    (7, "Notecards"),
    (6, "Objects"),
    (15, "Photo Album"),
    (10, "Scripts"),
    (1, "Sounds"),
    (0, "Textures"),
    (14, "Trash"),
    (56, "Settings"),
];

struct CachedTree {
    folder_ids: HashSet<Uuid>,
    created_at: Instant,
}

pub struct HGSuitcaseInventoryService {
    inner: Arc<dyn InventoryServiceTrait>,
    tree_cache: Arc<RwLock<HashMap<Uuid, CachedTree>>>,
}

impl HGSuitcaseInventoryService {
    pub fn new(inner: Arc<dyn InventoryServiceTrait>) -> Self {
        Self {
            inner,
            tree_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_suitcase_folder(&self, principal_id: Uuid) -> Result<InventoryFolder> {
        let skeleton = self.inner.get_inventory_skeleton(principal_id).await?;
        if let Some(suitcase) = skeleton
            .iter()
            .find(|f| f.folder_type == SUITCASE_FOLDER_TYPE)
        {
            return Ok(suitcase.clone());
        }

        let root = self
            .inner
            .get_root_folder(principal_id)
            .await?
            .ok_or_else(|| anyhow!("No root folder for user {}", principal_id))?;

        if let Some(suitcase) = skeleton
            .iter()
            .find(|f| f.name == "My Suitcase" && f.parent_id == root.folder_id)
        {
            return Ok(suitcase.clone());
        }

        info!(
            "[HG-SUITCASE] Creating suitcase for foreign user {}",
            principal_id
        );
        let suitcase_id = Uuid::new_v4();
        let suitcase = InventoryFolder {
            folder_id: suitcase_id,
            parent_id: root.folder_id,
            owner_id: principal_id,
            name: "My Suitcase".to_string(),
            folder_type: SUITCASE_FOLDER_TYPE,
            version: 1,
        };
        self.inner.create_folder(&suitcase).await?;

        for &(ftype, fname) in SUITCASE_SYSTEM_FOLDERS {
            let sub = InventoryFolder {
                folder_id: Uuid::new_v4(),
                parent_id: suitcase_id,
                owner_id: principal_id,
                name: fname.to_string(),
                folder_type: ftype,
                version: 1,
            };
            if let Err(e) = self.inner.create_folder(&sub).await {
                warn!(
                    "[HG-SUITCASE] Failed to create subfolder '{}' for {}: {}",
                    fname, principal_id, e
                );
            }
        }

        info!(
            "[HG-SUITCASE] Created suitcase with {} subfolders for {}",
            SUITCASE_SYSTEM_FOLDERS.len(),
            principal_id
        );
        Ok(suitcase)
    }

    async fn get_suitcase_tree(&self, principal_id: Uuid) -> Result<HashSet<Uuid>> {
        {
            let cache = self.tree_cache.read().await;
            if let Some(cached) = cache.get(&principal_id) {
                if cached.created_at.elapsed().as_secs() < 300 {
                    return Ok(cached.folder_ids.clone());
                }
            }
        }

        let suitcase = self.get_suitcase_folder(principal_id).await?;
        let skeleton = self.inner.get_inventory_skeleton(principal_id).await?;

        let mut tree = HashSet::new();
        tree.insert(suitcase.folder_id);

        let mut changed = true;
        while changed {
            changed = false;
            for folder in &skeleton {
                if !tree.contains(&folder.folder_id) && tree.contains(&folder.parent_id) {
                    tree.insert(folder.folder_id);
                    changed = true;
                }
            }
        }

        if let Some(cof) = skeleton.iter().find(|f| f.folder_type == 46) {
            tree.insert(cof.folder_id);
        }

        {
            let mut cache = self.tree_cache.write().await;
            cache.insert(
                principal_id,
                CachedTree {
                    folder_ids: tree.clone(),
                    created_at: Instant::now(),
                },
            );
        }

        debug!(
            "[HG-SUITCASE] Built suitcase tree for {}: {} folders",
            principal_id,
            tree.len()
        );
        Ok(tree)
    }

    async fn is_within_suitcase(&self, principal_id: Uuid, folder_id: Uuid) -> Result<bool> {
        let tree = self.get_suitcase_tree(principal_id).await?;
        Ok(tree.contains(&folder_id))
    }

    pub fn invalidate_cache(&self, principal_id: Uuid) {
        let cache = self.tree_cache.clone();
        tokio::spawn(async move {
            let mut c = cache.write().await;
            c.remove(&principal_id);
        });
    }

    async fn redirect_to_suitcase_folder(
        &self,
        principal_id: Uuid,
        original_folder_id: Uuid,
        asset_type: i32,
    ) -> Result<Option<Uuid>> {
        let target_folder_type = if asset_type == 6 {
            6
        }
        // Object
        else if asset_type == 0 {
            0
        }
        // Texture
        else if asset_type == 1 {
            1
        }
        // Sound
        else if asset_type == 3 {
            3
        }
        // Landmark
        else if asset_type == 5 {
            5
        }
        // Clothing
        else if asset_type == 7 {
            7
        }
        // Notecard
        else if asset_type == 10 {
            10
        }
        // Script
        else if asset_type == 13 {
            13
        }
        // Body Part
        else if asset_type == 20 {
            20
        }
        // Animation
        else if asset_type == 21 {
            21
        }
        // Gesture
        else if asset_type == 56 {
            56
        }
        // Settings
        else {
            let folder = self.inner.get_folder(original_folder_id).await?;
            match folder {
                Some(f) if f.folder_type >= 0 => f.folder_type,
                _ => 6,
            }
        };

        let suitcase = self.get_suitcase_folder(principal_id).await?;
        let skeleton = self.inner.get_inventory_skeleton(principal_id).await?;

        if let Some(matching) = skeleton
            .iter()
            .find(|f| f.parent_id == suitcase.folder_id && f.folder_type == target_folder_type)
        {
            info!(
                "[HG-SUITCASE] Redirected item from folder {} to suitcase subfolder {} (type={})",
                original_folder_id, matching.folder_id, target_folder_type
            );
            return Ok(Some(matching.folder_id));
        }

        if let Some(objects) = skeleton
            .iter()
            .find(|f| f.parent_id == suitcase.folder_id && f.folder_type == 6)
        {
            info!("[HG-SUITCASE] Redirected item from folder {} to suitcase Objects folder {} (fallback)", original_folder_id, objects.folder_id);
            return Ok(Some(objects.folder_id));
        }

        let folder_name = SUITCASE_SYSTEM_FOLDERS
            .iter()
            .find(|(t, _)| *t == target_folder_type)
            .map(|(_, name)| *name)
            .unwrap_or("Objects");

        let new_folder_id = Uuid::new_v4();
        let new_folder = InventoryFolder {
            folder_id: new_folder_id,
            owner_id: principal_id,
            parent_id: suitcase.folder_id,
            name: folder_name.to_string(),
            folder_type: target_folder_type,
            version: 1,
        };
        info!(
            "[HG-SUITCASE] Auto-creating suitcase subfolder '{}' (type={}) for user {}",
            folder_name, target_folder_type, principal_id
        );
        if let Ok(true) = self.inner.create_folder(&new_folder).await {
            self.invalidate_cache(principal_id);
            return Ok(Some(new_folder_id));
        }

        warn!("[HG-SUITCASE] Failed to auto-create suitcase subfolder for redirect (user={}, type={})", principal_id, target_folder_type);
        Ok(None)
    }
}

#[async_trait]
impl InventoryServiceTrait for HGSuitcaseInventoryService {
    async fn get_folder(&self, folder_id: Uuid) -> Result<Option<InventoryFolder>> {
        self.inner.get_folder(folder_id).await
    }

    async fn get_root_folder(&self, principal_id: Uuid) -> Result<Option<InventoryFolder>> {
        match self.get_suitcase_folder(principal_id).await {
            Ok(suitcase) => Ok(Some(suitcase)),
            Err(e) => {
                warn!(
                    "[HG-SUITCASE] Failed to get suitcase for {}: {}, falling back to real root",
                    principal_id, e
                );
                self.inner.get_root_folder(principal_id).await
            }
        }
    }

    async fn get_folder_content(
        &self,
        principal_id: Uuid,
        folder_id: Uuid,
    ) -> Result<InventoryCollection> {
        if !self.is_within_suitcase(principal_id, folder_id).await? {
            warn!(
                "[HG-SUITCASE] Blocked get_folder_content for {} — folder {} outside suitcase",
                principal_id, folder_id
            );
            return Ok(InventoryCollection::default());
        }
        self.inner.get_folder_content(principal_id, folder_id).await
    }

    async fn create_folder(&self, folder: &InventoryFolder) -> Result<bool> {
        if !self
            .is_within_suitcase(folder.owner_id, folder.parent_id)
            .await?
        {
            warn!(
                "[HG-SUITCASE] Blocked create_folder for {} — parent {} outside suitcase",
                folder.owner_id, folder.parent_id
            );
            return Ok(false);
        }
        let result = self.inner.create_folder(folder).await?;
        self.invalidate_cache(folder.owner_id);
        Ok(result)
    }

    async fn update_folder(&self, folder: &InventoryFolder) -> Result<bool> {
        if !self
            .is_within_suitcase(folder.owner_id, folder.folder_id)
            .await?
        {
            warn!(
                "[HG-SUITCASE] Blocked update_folder for {} — folder {} outside suitcase",
                folder.owner_id, folder.folder_id
            );
            return Ok(false);
        }
        self.inner.update_folder(folder).await
    }

    async fn delete_folders(&self, _principal_id: Uuid, _folder_ids: &[Uuid]) -> Result<bool> {
        warn!("[HG-SUITCASE] Blocked delete_folders for foreign user");
        Ok(false)
    }

    async fn get_item(&self, item_id: Uuid) -> Result<Option<InventoryItem>> {
        self.inner.get_item(item_id).await
    }

    async fn add_item(&self, item: &InventoryItem) -> Result<bool> {
        if !self
            .is_within_suitcase(item.owner_id, item.folder_id)
            .await?
        {
            if let Ok(Some(redirect_folder)) = self
                .redirect_to_suitcase_folder(item.owner_id, item.folder_id, item.asset_type)
                .await
            {
                let mut redirected = item.clone();
                redirected.folder_id = redirect_folder;
                return self.inner.add_item(&redirected).await;
            }
            warn!("[HG-SUITCASE] Blocked add_item for {} — folder {} outside suitcase, no redirect available", item.owner_id, item.folder_id);
            return Ok(false);
        }
        self.inner.add_item(item).await
    }

    async fn update_item(&self, item: &InventoryItem) -> Result<bool> {
        if !self
            .is_within_suitcase(item.owner_id, item.folder_id)
            .await?
        {
            warn!(
                "[HG-SUITCASE] Blocked update_item for {} — folder {} outside suitcase",
                item.owner_id, item.folder_id
            );
            return Ok(false);
        }
        self.inner.update_item(item).await
    }

    async fn delete_items(&self, _principal_id: Uuid, _item_ids: &[Uuid]) -> Result<bool> {
        warn!("[HG-SUITCASE] Blocked delete_items for foreign user");
        Ok(false)
    }

    async fn get_inventory_skeleton(&self, principal_id: Uuid) -> Result<Vec<InventoryFolder>> {
        let tree = self.get_suitcase_tree(principal_id).await?;
        let all_folders = self.inner.get_inventory_skeleton(principal_id).await?;
        let filtered: Vec<InventoryFolder> = all_folders
            .into_iter()
            .filter(|f| tree.contains(&f.folder_id))
            .collect();
        debug!(
            "[HG-SUITCASE] Skeleton for {}: {} folders (filtered from full inventory)",
            principal_id,
            filtered.len()
        );
        Ok(filtered)
    }

    async fn move_items(&self, principal_id: Uuid, items: &[(Uuid, Uuid)]) -> Result<bool> {
        let tree = self.get_suitcase_tree(principal_id).await?;
        let mut filtered_moves = Vec::new();
        for &(item_id, dest_folder_id) in items {
            if let Ok(Some(item)) = self.inner.get_item(item_id).await {
                if !tree.contains(&item.folder_id) {
                    warn!("[HG-SUITCASE] Blocked move_items: item {} source folder {} outside suitcase", item_id, item.folder_id);
                    continue;
                }
            }
            let actual_dest = if tree.contains(&dest_folder_id) {
                dest_folder_id
            } else {
                if let Ok(Some(redirect)) = self
                    .redirect_to_suitcase_folder(principal_id, dest_folder_id, 6)
                    .await
                {
                    info!(
                        "[HG-SUITCASE] Redirected item {} move to suitcase folder {}",
                        item_id, redirect
                    );
                    redirect
                } else {
                    warn!("[HG-SUITCASE] Blocked move_items: item {} dest folder {} outside suitcase, no redirect", item_id, dest_folder_id);
                    continue;
                }
            };
            filtered_moves.push((item_id, actual_dest));
        }
        if filtered_moves.is_empty() {
            return Ok(true);
        }
        self.inner.move_items(principal_id, &filtered_moves).await
    }

    async fn move_folder(
        &self,
        _principal_id: Uuid,
        _folder_id: Uuid,
        _new_parent_id: Uuid,
    ) -> Result<bool> {
        warn!("[HG-SUITCASE] Blocked move_folder for foreign user");
        Ok(false)
    }

    async fn purge_folder(&self, _principal_id: Uuid, _folder_id: Uuid) -> Result<bool> {
        warn!("[HG-SUITCASE] Blocked purge_folder for foreign user");
        Ok(false)
    }

    async fn get_active_gestures(&self, principal_id: Uuid) -> Result<Vec<InventoryItem>> {
        self.inner.get_active_gestures(principal_id).await
    }

    async fn get_multiple_folders_content(
        &self,
        principal_id: Uuid,
        folder_ids: &[Uuid],
    ) -> Result<Vec<InventoryCollection>> {
        let tree = self.get_suitcase_tree(principal_id).await?;
        let filtered_ids: Vec<Uuid> = folder_ids
            .iter()
            .filter(|id| tree.contains(id))
            .copied()
            .collect();
        if filtered_ids.len() < folder_ids.len() {
            warn!(
                "[HG-SUITCASE] Filtered {} of {} folder requests (outside suitcase)",
                folder_ids.len() - filtered_ids.len(),
                folder_ids.len()
            );
        }
        self.inner
            .get_multiple_folders_content(principal_id, &filtered_ids)
            .await
    }

    async fn get_asset_permissions(&self, principal_id: Uuid, asset_id: Uuid) -> Result<i32> {
        self.inner
            .get_asset_permissions(principal_id, asset_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suitcase_folder_type() {
        assert_eq!(SUITCASE_FOLDER_TYPE, 100);
    }

    #[test]
    fn test_system_folders_count() {
        assert_eq!(SUITCASE_SYSTEM_FOLDERS.len(), 16);
    }
}
