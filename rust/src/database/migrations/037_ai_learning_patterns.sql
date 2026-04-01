-- Migration 037: AI Learning Patterns Persistence
-- Phase 68.25: P1 Learning Persistence Implementation
-- Stores learned patterns for EADS learning system persistence across restarts

-- AI Learned Patterns table
CREATE TABLE IF NOT EXISTS ai_learned_patterns (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    category TEXT NOT NULL,
    pattern_data BLOB NOT NULL,
    quality_score REAL DEFAULT 0.0,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- AI Learning Metrics table for tracking pattern performance over time
CREATE TABLE IF NOT EXISTS ai_learning_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern_id TEXT NOT NULL,
    metric_type TEXT NOT NULL,
    metric_value REAL NOT NULL,
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (pattern_id) REFERENCES ai_learned_patterns(id) ON DELETE CASCADE
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_ai_patterns_category ON ai_learned_patterns(category);
CREATE INDEX IF NOT EXISTS idx_ai_patterns_quality ON ai_learned_patterns(quality_score);
CREATE INDEX IF NOT EXISTS idx_ai_patterns_updated ON ai_learned_patterns(updated_at);
CREATE INDEX IF NOT EXISTS idx_ai_metrics_pattern ON ai_learning_metrics(pattern_id);
CREATE INDEX IF NOT EXISTS idx_ai_metrics_type ON ai_learning_metrics(metric_type);
CREATE INDEX IF NOT EXISTS idx_ai_metrics_recorded ON ai_learning_metrics(recorded_at);
