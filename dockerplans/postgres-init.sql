-- OpenSim NextGen — PostgreSQL Initialization
-- Runs once on first container start (docker-entrypoint-initdb.d/)
-- Core tables are created by the Rust migration engine on app startup

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Additional database users
CREATE USER opensim_readonly WITH PASSWORD 'readonly_2024';
CREATE USER opensim_backup WITH PASSWORD 'backup_2024';

-- Permissions for main user
GRANT ALL PRIVILEGES ON DATABASE opensim TO opensim;

-- Read-only access for monitoring
GRANT CONNECT ON DATABASE opensim TO opensim_readonly;
GRANT USAGE ON SCHEMA public TO opensim_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO opensim_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO opensim_readonly;

-- Backup user permissions
GRANT CONNECT ON DATABASE opensim TO opensim_backup;
GRANT USAGE ON SCHEMA public TO opensim_backup;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO opensim_backup;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO opensim_backup;

-- Monitoring schema
CREATE SCHEMA IF NOT EXISTS monitoring;
GRANT ALL ON SCHEMA monitoring TO opensim;
GRANT USAGE ON SCHEMA monitoring TO opensim_readonly;

-- Server metrics
CREATE TABLE IF NOT EXISTS monitoring.server_stats (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    instance_id VARCHAR(255) NOT NULL,
    active_users INTEGER DEFAULT 0,
    active_regions INTEGER DEFAULT 0,
    total_objects BIGINT DEFAULT 0,
    memory_usage_mb INTEGER DEFAULT 0,
    cpu_usage_percent DECIMAL(5,2) DEFAULT 0.0,
    uptime_seconds BIGINT DEFAULT 0
);

-- Per-region metrics
CREATE TABLE IF NOT EXISTS monitoring.region_stats (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    region_id UUID NOT NULL,
    region_name VARCHAR(255) NOT NULL,
    avatar_count INTEGER DEFAULT 0,
    object_count INTEGER DEFAULT 0,
    active_scripts INTEGER DEFAULT 0,
    physics_engine VARCHAR(50),
    physics_fps DECIMAL(6,2) DEFAULT 0.0,
    last_activity TIMESTAMPTZ DEFAULT NOW()
);

-- Generic performance metrics
CREATE TABLE IF NOT EXISTS monitoring.performance_metrics (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    metric_name VARCHAR(255) NOT NULL,
    metric_value DECIMAL(15,6) NOT NULL,
    instance_id VARCHAR(255),
    region_id UUID,
    tags JSONB
);

-- Monitoring indexes
CREATE INDEX IF NOT EXISTS idx_server_stats_ts ON monitoring.server_stats(timestamp);
CREATE INDEX IF NOT EXISTS idx_region_stats_ts ON monitoring.region_stats(timestamp);
CREATE INDEX IF NOT EXISTS idx_region_stats_region ON monitoring.region_stats(region_id);
CREATE INDEX IF NOT EXISTS idx_perf_metrics_ts ON monitoring.performance_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_perf_metrics_name ON monitoring.performance_metrics(metric_name);

-- Monitoring permissions
GRANT ALL ON ALL TABLES IN SCHEMA monitoring TO opensim;
GRANT ALL ON ALL SEQUENCES IN SCHEMA monitoring TO opensim;
GRANT SELECT ON ALL TABLES IN SCHEMA monitoring TO opensim_readonly;

-- Cleanup function for old monitoring data
CREATE OR REPLACE FUNCTION monitoring.cleanup_old_data(retention_days INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER := 0;
    partial INTEGER;
BEGIN
    DELETE FROM monitoring.server_stats
    WHERE timestamp < NOW() - INTERVAL '1 day' * retention_days;
    GET DIAGNOSTICS partial = ROW_COUNT;
    deleted_count := deleted_count + partial;

    DELETE FROM monitoring.region_stats
    WHERE timestamp < NOW() - INTERVAL '1 day' * retention_days;
    GET DIAGNOSTICS partial = ROW_COUNT;
    deleted_count := deleted_count + partial;

    DELETE FROM monitoring.performance_metrics
    WHERE timestamp < NOW() - INTERVAL '1 day' * retention_days;
    GET DIAGNOSTICS partial = ROW_COUNT;
    deleted_count := deleted_count + partial;

    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Runtime configuration table
CREATE TABLE IF NOT EXISTS public.server_config (
    id SERIAL PRIMARY KEY,
    config_key VARCHAR(255) UNIQUE NOT NULL,
    config_value TEXT NOT NULL,
    config_type VARCHAR(50) DEFAULT 'string',
    description TEXT,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    updated_by VARCHAR(255) DEFAULT 'system'
);

INSERT INTO public.server_config (config_key, config_value, config_type, description) VALUES
    ('server.max_users', '200', 'integer', 'Maximum concurrent users'),
    ('server.max_regions', '4', 'integer', 'Configured region count'),
    ('physics.default_engine', 'ubODE', 'string', 'Default physics engine'),
    ('cache.default_ttl', '3600', 'integer', 'Default cache TTL in seconds'),
    ('monitoring.retention_days', '30', 'integer', 'Days to retain monitoring data'),
    ('backup.auto_enabled', 'true', 'boolean', 'Enable automatic backups'),
    ('backup.interval_hours', '24', 'integer', 'Hours between automatic backups')
ON CONFLICT (config_key) DO NOTHING;

-- Audit log
CREATE TABLE IF NOT EXISTS public.audit_log (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    user_id UUID,
    user_name VARCHAR(255),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id VARCHAR(255),
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT
);

CREATE INDEX IF NOT EXISTS idx_audit_ts ON public.audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_user ON public.audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_action ON public.audit_log(action);

GRANT INSERT, SELECT ON public.audit_log TO opensim;
GRANT SELECT ON public.audit_log TO opensim_readonly;

-- Ensure opensim user can create tables (for migrations)
GRANT CREATE ON SCHEMA public TO opensim;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO opensim;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA monitoring TO opensim;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO opensim_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA monitoring GRANT SELECT ON TABLES TO opensim_readonly;

-- Log completion
INSERT INTO public.audit_log (action, resource_type, resource_id, new_values)
VALUES ('DATABASE_INIT', 'postgresql', 'opensim', '{"status": "completed", "regions": 4, "version": "1.0.0"}');

DO $$
BEGIN
    RAISE NOTICE 'OpenSim NextGen PostgreSQL initialization complete';
    RAISE NOTICE 'Users: opensim (main), opensim_readonly, opensim_backup';
    RAISE NOTICE 'Schemas: public, monitoring';
    RAISE NOTICE 'Core tables will be created by Rust migrations on first startup';
END $$;
