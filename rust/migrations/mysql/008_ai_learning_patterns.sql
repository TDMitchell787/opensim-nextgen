-- Migration 008: AI Learning Patterns Persistence (MySQL/MariaDB)
-- Phase 68.25: P1 Learning Persistence Implementation
-- Stores learned patterns for EADS learning system persistence across restarts

-- AI Learned Patterns table
CREATE TABLE IF NOT EXISTS ai_learned_patterns (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    category VARCHAR(100) NOT NULL,
    pattern_data MEDIUMBLOB NOT NULL,
    quality_score DOUBLE DEFAULT 0.0,
    usage_count INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_ai_patterns_category (category),
    INDEX idx_ai_patterns_quality (quality_score),
    INDEX idx_ai_patterns_updated (updated_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- AI Learning Metrics table for tracking pattern performance over time
CREATE TABLE IF NOT EXISTS ai_learning_metrics (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    pattern_id VARCHAR(36) NOT NULL,
    metric_type VARCHAR(100) NOT NULL,
    metric_value DOUBLE NOT NULL,
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_ai_metrics_pattern (pattern_id),
    INDEX idx_ai_metrics_type (metric_type),
    INDEX idx_ai_metrics_recorded (recorded_at),
    CONSTRAINT fk_ai_metrics_pattern FOREIGN KEY (pattern_id) REFERENCES ai_learned_patterns(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
