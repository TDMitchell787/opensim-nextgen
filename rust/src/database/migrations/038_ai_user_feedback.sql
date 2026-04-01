-- Migration 038: AI User Feedback System
-- Phase 68.25: P4 User Feedback Loop Implementation
-- Stores user feedback and recommendation outcomes for learning optimization

-- AI User Feedback table
CREATE TABLE IF NOT EXISTS ai_user_feedback (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    content_id TEXT NOT NULL,
    feedback_type TEXT NOT NULL,
    feedback_value REAL,
    feedback_text TEXT,
    context_data TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- AI Recommendation Outcomes table
CREATE TABLE IF NOT EXISTS ai_recommendation_outcomes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recommendation_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    recommendation_type TEXT NOT NULL,
    was_accepted INTEGER DEFAULT 0,
    time_to_decision_ms INTEGER,
    subsequent_satisfaction REAL,
    context_data TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- AI User Preferences table for aggregated preference profiles
CREATE TABLE IF NOT EXISTS ai_user_preferences (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL UNIQUE,
    preference_data TEXT NOT NULL,
    total_interactions INTEGER DEFAULT 0,
    last_interaction_at TIMESTAMP,
    preference_score REAL DEFAULT 0.5,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- AI Pattern Affinity table for user-pattern relationships
CREATE TABLE IF NOT EXISTS ai_pattern_affinity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    pattern_id TEXT NOT NULL,
    affinity_score REAL DEFAULT 0.5,
    interaction_count INTEGER DEFAULT 0,
    last_interaction_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, pattern_id)
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_feedback_user ON ai_user_feedback(user_id);
CREATE INDEX IF NOT EXISTS idx_feedback_content ON ai_user_feedback(content_id);
CREATE INDEX IF NOT EXISTS idx_feedback_type ON ai_user_feedback(feedback_type);
CREATE INDEX IF NOT EXISTS idx_feedback_created ON ai_user_feedback(created_at);

CREATE INDEX IF NOT EXISTS idx_outcomes_user ON ai_recommendation_outcomes(user_id);
CREATE INDEX IF NOT EXISTS idx_outcomes_rec ON ai_recommendation_outcomes(recommendation_id);
CREATE INDEX IF NOT EXISTS idx_outcomes_type ON ai_recommendation_outcomes(recommendation_type);
CREATE INDEX IF NOT EXISTS idx_outcomes_accepted ON ai_recommendation_outcomes(was_accepted);
CREATE INDEX IF NOT EXISTS idx_outcomes_created ON ai_recommendation_outcomes(created_at);

CREATE INDEX IF NOT EXISTS idx_prefs_user ON ai_user_preferences(user_id);
CREATE INDEX IF NOT EXISTS idx_prefs_score ON ai_user_preferences(preference_score);

CREATE INDEX IF NOT EXISTS idx_affinity_user ON ai_pattern_affinity(user_id);
CREATE INDEX IF NOT EXISTS idx_affinity_pattern ON ai_pattern_affinity(pattern_id);
CREATE INDEX IF NOT EXISTS idx_affinity_score ON ai_pattern_affinity(affinity_score);
