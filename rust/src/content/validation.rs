//! Content Validation for OpenSim Next
//!
//! Provides content validation, format checking, and quality assessment.

use std::path::Path;
use std::fs;
use super::{ContentResult, ContentValidationResult, ContentType, ContentError};

const MAX_TEXTURE_SIZE: u64 = 10 * 1024 * 1024;
const MAX_MODEL_SIZE: u64 = 50 * 1024 * 1024;
const MAX_AUDIO_SIZE: u64 = 10 * 1024 * 1024;
const MAX_SCRIPT_SIZE: u64 = 64 * 1024;
const MAX_ANIMATION_SIZE: u64 = 5 * 1024 * 1024;
const MAX_VIDEO_SIZE: u64 = 100 * 1024 * 1024;

pub struct ContentValidator {
    max_texture_size: u64,
    max_model_size: u64,
    max_audio_size: u64,
    max_script_size: u64,
}

impl ContentValidator {
    pub fn new() -> Self {
        Self {
            max_texture_size: MAX_TEXTURE_SIZE,
            max_model_size: MAX_MODEL_SIZE,
            max_audio_size: MAX_AUDIO_SIZE,
            max_script_size: MAX_SCRIPT_SIZE,
        }
    }

    pub async fn validate_content(
        &self,
        content_type: &ContentType,
        file_path: &Path,
    ) -> ContentResult<ContentValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        if !file_path.exists() {
            return Err(ContentError::ValidationFailed {
                reason: format!("File not found: {:?}", file_path),
            });
        }

        let metadata = fs::metadata(file_path).map_err(|e| ContentError::ValidationFailed {
            reason: format!("Cannot read file metadata: {}", e),
        })?;
        let file_size = metadata.len();

        let (max_size, type_name) = match content_type {
            ContentType::Texture => (self.max_texture_size, "texture"),
            ContentType::Model3D => (self.max_model_size, "3D model"),
            ContentType::Audio => (self.max_audio_size, "audio"),
            ContentType::Video => (MAX_VIDEO_SIZE, "video"),
            ContentType::Script => (self.max_script_size, "script"),
            ContentType::Animation => (MAX_ANIMATION_SIZE, "animation"),
            ContentType::Notecard => (self.max_script_size, "notecard"),
            ContentType::Gesture => (self.max_script_size, "gesture"),
            ContentType::Landmark => (1024, "landmark"),
            ContentType::Wearable => (self.max_texture_size, "wearable"),
            ContentType::ParticleSystem => (64 * 1024, "particle system"),
            ContentType::Custom(_) => (self.max_model_size, "custom"),
        };

        if file_size > max_size {
            errors.push(format!(
                "File size ({} bytes) exceeds maximum allowed for {} ({} bytes)",
                file_size, type_name, max_size
            ));
        } else if file_size > max_size * 80 / 100 {
            warnings.push(format!(
                "File size ({} bytes) is close to maximum for {} ({}% of limit)",
                file_size,
                type_name,
                (file_size * 100 / max_size)
            ));
        }

        let (format_valid, format_errors, format_warnings) =
            self.validate_format(content_type, file_path).await;

        if !format_valid {
            errors.extend(format_errors);
        }
        warnings.extend(format_warnings);

        let performance_score =
            self.calculate_performance_score(content_type, file_size, &errors, &warnings);
        let security_score = self.calculate_security_score(content_type, file_path).await;
        let quality_score = self.calculate_quality_score(&errors, &warnings, file_size, max_size);

        if performance_score < 70.0 {
            recommendations.push("Consider optimizing content for better performance".to_string());
        }
        if security_score < 80.0 {
            recommendations.push("Review content for potential security concerns".to_string());
        }
        if quality_score < 80.0 {
            recommendations.push("Content quality could be improved".to_string());
        }
        if errors.is_empty() && warnings.is_empty() {
            recommendations.push("Content passes all validation checks".to_string());
        }

        Ok(ContentValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            performance_score: performance_score as f32,
            security_score: security_score as f32,
            quality_score: quality_score as f32,
            recommendations,
        })
    }

    async fn validate_format(
        &self,
        content_type: &ContentType,
        file_path: &Path,
    ) -> (bool, Vec<String>, Vec<String>) {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let data = match fs::read(file_path) {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!("Cannot read file: {}", e));
                return (false, errors, warnings);
            }
        };

        if data.is_empty() {
            errors.push("File is empty".to_string());
            return (false, errors, warnings);
        }

        let valid = match content_type {
            ContentType::Texture => {
                if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
                    true
                } else if data.len() >= 8 && &data[0..8] == b"\x89PNG\r\n\x1a\n" {
                    true
                } else if data.len() >= 2 && data[0] == 0xFF && data[1] == 0x4F {
                    true
                } else if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
                    true
                } else if data.len() >= 18 {
                    warnings.push(
                        "TGA or unknown format - consider converting to J2K for efficiency"
                            .to_string(),
                    );
                    true
                } else {
                    errors.push("Unknown or invalid texture format".to_string());
                    false
                }
            }
            ContentType::Model3D => {
                if data.len() >= 4 && &data[0..4] == b"glTF" {
                    true
                } else if data.len() >= 4 && (&data[0..4] == b"FBX\x00" || &data[0..4] == b"Fbx\x00")
                {
                    true
                } else if data.len() >= 6 && data[0..6].windows(5).any(|w| w == b"solid") {
                    warnings.push("STL format detected - may need conversion".to_string());
                    true
                } else if data.len() > 0 {
                    warnings.push(
                        "Model format could not be determined - may require conversion".to_string(),
                    );
                    true
                } else {
                    errors.push("Invalid model data".to_string());
                    false
                }
            }
            ContentType::Audio => {
                if data.len() >= 4 && &data[0..4] == b"OggS" {
                    true
                } else if data.len() >= 4 && &data[0..4] == b"RIFF" {
                    true
                } else if data.len() >= 3
                    && (&data[0..3] == b"ID3" || (data[0] == 0xFF && (data[1] & 0xE0) == 0xE0))
                {
                    warnings
                        .push("MP3 format - may need conversion to OGG for streaming".to_string());
                    true
                } else {
                    errors.push("Unknown or invalid audio format".to_string());
                    false
                }
            }
            ContentType::Video => {
                if data.len() >= 8 && (&data[4..8] == b"ftyp" || &data[4..8] == b"moov") {
                    true
                } else if data.len() >= 4 && &data[0..4] == b"RIFF" {
                    true
                } else {
                    warnings.push("Video format could not be verified".to_string());
                    true
                }
            }
            ContentType::Script => {
                let content = String::from_utf8_lossy(&data);
                if content.contains("default") && content.contains("state_entry") {
                    true
                } else if content.contains("llSay")
                    || content.contains("llListen")
                    || content.contains("llGetOwner")
                {
                    warnings.push("Script may be incomplete - no default state found".to_string());
                    true
                } else {
                    warnings.push("Script content could not be validated as LSL".to_string());
                    true
                }
            }
            ContentType::Animation => {
                if data.len() >= 4 && &data[0..4] == b"ANIM" {
                    true
                } else if data.len() >= 9
                    && String::from_utf8_lossy(&data[0..9])
                        .to_lowercase()
                        .contains("hierarchy")
                {
                    true
                } else {
                    warnings.push("Animation format could not be determined".to_string());
                    true
                }
            }
            ContentType::Notecard | ContentType::Gesture | ContentType::Landmark => true,
            ContentType::Wearable | ContentType::ParticleSystem => true,
            ContentType::Custom(_) => {
                warnings.push("Custom content type - validation limited".to_string());
                true
            }
        };

        (valid, errors, warnings)
    }

    fn calculate_performance_score(
        &self,
        content_type: &ContentType,
        file_size: u64,
        errors: &[String],
        warnings: &[String],
    ) -> f64 {
        let mut score: f64 = 100.0;

        let optimal_size: u64 = match content_type {
            ContentType::Texture => 512 * 1024,
            ContentType::Model3D => 5 * 1024 * 1024,
            ContentType::Audio => 1 * 1024 * 1024,
            ContentType::Video => 20 * 1024 * 1024,
            ContentType::Script => 16 * 1024,
            ContentType::Animation => 512 * 1024,
            _ => 64 * 1024,
        };

        if file_size > optimal_size {
            let excess_ratio = (file_size - optimal_size) as f64 / optimal_size as f64;
            score -= (excess_ratio * 20.0).min(30.0);
        }

        score -= errors.len() as f64 * 15.0;
        score -= warnings.len() as f64 * 5.0;

        score.max(0.0).min(100.0)
    }

    async fn calculate_security_score(&self, content_type: &ContentType, file_path: &Path) -> f64 {
        let mut score: f64 = 100.0;

        if let ContentType::Script = content_type {
            if let Ok(content) = fs::read_to_string(file_path) {
                let dangerous_patterns = [
                    "llEmail",
                    "llHTTPRequest",
                    "llLoadURL",
                    "llMapDestination",
                    "llTeleportAgent",
                    "llGiveInventory",
                    "llGiveMoney",
                ];

                for pattern in dangerous_patterns {
                    if content.contains(pattern) {
                        score -= 5.0;
                    }
                }

                if content.contains("llGetOwnerKey") && content.contains("llKey2Name") {
                    score -= 5.0;
                }
            }
        }

        score.max(0.0).min(100.0)
    }

    fn calculate_quality_score(
        &self,
        errors: &[String],
        warnings: &[String],
        file_size: u64,
        max_size: u64,
    ) -> f64 {
        let mut score: f64 = 100.0;

        score -= errors.len() as f64 * 20.0;
        score -= warnings.len() as f64 * 5.0;

        let size_ratio = file_size as f64 / max_size as f64;
        if size_ratio > 0.9 {
            score -= 10.0;
        } else if size_ratio > 0.75 {
            score -= 5.0;
        }

        score.max(0.0).min(100.0)
    }
}
