-- Migration 008: AI Learning Patterns Persistence (PostgreSQL)
-- Phase 68.25: P1 Learning Persistence Implementation
-- Stores learned patterns for EADS learning system persistence across restarts

-- AI Learned Patterns table
CREATE TABLE IF NOT EXISTS ai_learned_patterns (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    category VARCHAR(100) NOT NULL,
    pattern_data BYTEA NOT NULL,
    quality_score DOUBLE PRECISION DEFAULT 0.0,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- AI Learning Metrics table for tracking pattern performance over time
CREATE TABLE IF NOT EXISTS ai_learning_metrics (
    id SERIAL PRIMARY KEY,
    pattern_id VARCHAR(36) NOT NULL REFERENCES ai_learned_patterns(id) ON DELETE CASCADE,
    metric_type VARCHAR(100) NOT NULL,
    metric_value DOUBLE PRECISION NOT NULL,
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_ai_patterns_category ON ai_learned_patterns(category);
CREATE INDEX IF NOT EXISTS idx_ai_patterns_quality ON ai_learned_patterns(quality_score);
CREATE INDEX IF NOT EXISTS idx_ai_patterns_updated ON ai_learned_patterns(updated_at);
CREATE INDEX IF NOT EXISTS idx_ai_metrics_pattern ON ai_learning_metrics(pattern_id);
CREATE INDEX IF NOT EXISTS idx_ai_metrics_type ON ai_learning_metrics(metric_type);
CREATE INDEX IF NOT EXISTS idx_ai_metrics_recorded ON ai_learning_metrics(recorded_at);

-- Update trigger for updated_at
CREATE OR REPLACE FUNCTION update_ai_patterns_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS ai_patterns_updated_at ON ai_learned_patterns;
CREATE TRIGGER ai_patterns_updated_at
    BEFORE UPDATE ON ai_learned_patterns
    FOR EACH ROW
    EXECUTE FUNCTION update_ai_patterns_updated_at();
