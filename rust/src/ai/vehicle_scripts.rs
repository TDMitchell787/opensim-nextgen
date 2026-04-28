use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tracing::info;

static LIBRARY: OnceLock<VehicleScriptLibrary> = OnceLock::new();

pub struct VehicleScriptLibrary {
    scripts: HashMap<String, String>,
}

impl VehicleScriptLibrary {
    pub fn global() -> &'static VehicleScriptLibrary {
        LIBRARY.get_or_init(|| {
            let mut lib = VehicleScriptLibrary {
                scripts: HashMap::new(),
            };
            let base = PathBuf::from("content");
            let dirs = [
                "gaia_land_vehicles",
                "gaia_flying_vehicles",
                "gaia_flying_vtol_vehicles",
                "gaia_sailing_vessel",
                "gaia_starship",
                "gaia_helicarrier",
            ];
            for dir in &dirs {
                let path = base.join(dir);
                if path.exists() {
                    lib.load_from_directory(&path);
                }
            }
            info!(
                "[VEHICLE_SCRIPTS] Loaded {} scripts from {} directories",
                lib.scripts.len(),
                dirs.len()
            );
            lib
        })
    }

    fn load_from_directory(&mut self, path: &Path) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.extension().map(|e| e == "lsl").unwrap_or(false) {
                    if let Some(name) = file_path.file_name().and_then(|n| n.to_str()) {
                        if let Ok(source) = std::fs::read_to_string(&file_path) {
                            info!(
                                "[VEHICLE_SCRIPTS] Loaded: {} ({} bytes)",
                                name,
                                source.len()
                            );
                            self.scripts.insert(name.to_string(), source);
                        }
                    }
                }
            }
        }
    }

    pub fn get_script(&self, name: &str) -> Option<&str> {
        self.scripts.get(name).map(|s| s.as_str())
    }

    pub fn get_script_with_tuning(
        &self,
        name: &str,
        tuning: &HashMap<String, f32>,
    ) -> Option<String> {
        let source = self.scripts.get(name)?;
        if tuning.is_empty() {
            return Some(source.clone());
        }
        let mut result = source.clone();
        for (param, value) in tuning {
            let patterns = [
                format!("float {} = ", param),
                format!("integer {} = ", param),
            ];
            for pattern in &patterns {
                if let Some(start) = result.find(pattern.as_str()) {
                    let after_eq = start + pattern.len();
                    if let Some(semi) = result[after_eq..].find(';') {
                        let old_val = &result[after_eq..after_eq + semi];
                        let new_val = if pattern.starts_with("integer") {
                            format!("{}", *value as i32)
                        } else {
                            format!("{:.1}", value)
                        };
                        info!(
                            "[VEHICLE_SCRIPTS] Tuning {} in {}: {} → {}",
                            param,
                            name,
                            old_val.trim(),
                            new_val
                        );
                        result = format!(
                            "{}{}{}",
                            &result[..after_eq],
                            new_val,
                            &result[after_eq + semi..]
                        );
                    }
                }
            }
        }
        Some(result)
    }

    pub fn list_scripts(&self) -> Vec<&str> {
        self.scripts.keys().map(|k| k.as_str()).collect()
    }

    pub fn script_count(&self) -> usize {
        self.scripts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_creation() {
        let lib = VehicleScriptLibrary {
            scripts: HashMap::new(),
        };
        assert_eq!(lib.script_count(), 0);
        assert!(lib.get_script("nonexistent.lsl").is_none());
    }

    #[test]
    fn test_tuning_substitution() {
        let mut lib = VehicleScriptLibrary {
            scripts: HashMap::new(),
        };
        lib.scripts.insert(
            "test.lsl".to_string(),
            "float MAX_SPEED = 40.0;\ninteger HUD_CH = -14710;\nfloat TURN_RATE = 2.5;".to_string(),
        );

        let mut tuning = HashMap::new();
        tuning.insert("MAX_SPEED".to_string(), 60.0);
        tuning.insert("HUD_CH".to_string(), -14720.0);

        let result = lib.get_script_with_tuning("test.lsl", &tuning).unwrap();
        assert!(result.contains("float MAX_SPEED = 60.0;"));
        assert!(result.contains("integer HUD_CH = -14720;"));
        assert!(result.contains("float TURN_RATE = 2.5;"));
    }

    #[test]
    fn test_empty_tuning_returns_original() {
        let mut lib = VehicleScriptLibrary {
            scripts: HashMap::new(),
        };
        let original = "float X = 1.0;".to_string();
        lib.scripts.insert("test.lsl".to_string(), original.clone());
        let result = lib
            .get_script_with_tuning("test.lsl", &HashMap::new())
            .unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_load_from_content_directory() {
        let lib = VehicleScriptLibrary::global();
        if lib.script_count() > 0 {
            assert!(lib.get_script("car_controller.lsl").is_some());
        }
    }
}
