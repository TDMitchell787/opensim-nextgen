-- OpenSim Next PostgreSQL Initialization Script
-- Creates database schema, users, and initial configuration

-- Create additional databases if needed
-- CREATE DATABASE opensim_test;
-- CREATE DATABASE opensim_backup;

-- Create database users with appropriate permissions
-- Note: Main 'opensim' user is created by environment variables

-- Create read-only user for monitoring/reporting
CREATE USER opensim_readonly WITH PASSWORD 'CHANGEME';

-- Create backup user with specific permissions
CREATE USER opensim_backup WITH PASSWORD 'CHANGEME';

-- Grant permissions to main opensim user
GRANT ALL PRIVILEGES ON DATABASE opensim TO opensim;

-- Grant read-only access to monitoring user
GRANT CONNECT ON DATABASE opensim TO opensim_readonly;
GRANT USAGE ON SCHEMA public TO opensim_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO opensim_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO opensim_readonly;

-- Grant backup permissions
GRANT CONNECT ON DATABASE opensim TO opensim_backup;
GRANT USAGE ON SCHEMA public TO opensim_backup;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO opensim_backup;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO opensim_backup;

-- Create monitoring schema for metrics
CREATE SCHEMA IF NOT EXISTS monitoring;
GRANT ALL ON SCHEMA monitoring TO opensim;
GRANT USAGE ON SCHEMA monitoring TO opensim_readonly;

-- Create monitoring tables for application metrics
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

CREATE TABLE IF NOT EXISTS monitoring.performance_metrics (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    metric_name VARCHAR(255) NOT NULL,
    metric_value DECIMAL(15,6) NOT NULL,
    instance_id VARCHAR(255),
    region_id UUID,
    tags JSONB
);

-- Create indexes for monitoring tables
CREATE INDEX IF NOT EXISTS idx_server_stats_timestamp ON monitoring.server_stats(timestamp);
CREATE INDEX IF NOT EXISTS idx_server_stats_instance ON monitoring.server_stats(instance_id);
CREATE INDEX IF NOT EXISTS idx_region_stats_timestamp ON monitoring.region_stats(timestamp);
CREATE INDEX IF NOT EXISTS idx_region_stats_region ON monitoring.region_stats(region_id);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_timestamp ON monitoring.performance_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_performance_metrics_name ON monitoring.performance_metrics(metric_name);

-- Grant permissions on monitoring tables
GRANT ALL ON ALL TABLES IN SCHEMA monitoring TO opensim;
GRANT ALL ON ALL SEQUENCES IN SCHEMA monitoring TO opensim;
GRANT SELECT ON ALL TABLES IN SCHEMA monitoring TO opensim_readonly;

-- Create function to clean old monitoring data
CREATE OR REPLACE FUNCTION monitoring.cleanup_old_data(retention_days INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER := 0;
BEGIN
    -- Clean server stats older than retention period
    DELETE FROM monitoring.server_stats 
    WHERE timestamp < NOW() - INTERVAL '1 day' * retention_days;
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    -- Clean region stats older than retention period
    DELETE FROM monitoring.region_stats 
    WHERE timestamp < NOW() - INTERVAL '1 day' * retention_days;
    GET DIAGNOSTICS deleted_count = deleted_count + ROW_COUNT;
    
    -- Clean performance metrics older than retention period
    DELETE FROM monitoring.performance_metrics 
    WHERE timestamp < NOW() - INTERVAL '1 day' * retention_days;
    GET DIAGNOSTICS deleted_count = deleted_count + ROW_COUNT;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Create configuration table for runtime settings
CREATE TABLE IF NOT EXISTS public.server_config (
    id SERIAL PRIMARY KEY,
    config_key VARCHAR(255) UNIQUE NOT NULL,
    config_value TEXT NOT NULL,
    config_type VARCHAR(50) DEFAULT 'string',
    description TEXT,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    updated_by VARCHAR(255) DEFAULT 'system'
);

-- Insert default configuration values
INSERT INTO public.server_config (config_key, config_value, config_type, description) VALUES
    ('server.max_users', '1000', 'integer', 'Maximum concurrent users'),
    ('server.max_regions', '100', 'integer', 'Maximum regions per instance'),
    ('physics.default_engine', 'ODE', 'string', 'Default physics engine for new regions'),
    ('physics.max_bodies', '10000', 'integer', 'Maximum physics bodies per region'),
    ('cache.default_ttl', '3600', 'integer', 'Default cache TTL in seconds'),
    ('websocket.max_connections', '1000', 'integer', 'Maximum WebSocket connections'),
    ('monitoring.metrics_retention_days', '30', 'integer', 'Days to retain monitoring metrics'),
    ('backup.auto_enabled', 'true', 'boolean', 'Enable automatic backups'),
    ('backup.interval_hours', '24', 'integer', 'Hours between automatic backups')
ON CONFLICT (config_key) DO NOTHING;

-- Create indexes on configuration table
CREATE INDEX IF NOT EXISTS idx_server_config_key ON public.server_config(config_key);
CREATE INDEX IF NOT EXISTS idx_server_config_type ON public.server_config(config_type);

-- Create audit log table for tracking changes
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

-- Create indexes on audit log
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON public.audit_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_audit_log_user ON public.audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_action ON public.audit_log(action);
CREATE INDEX IF NOT EXISTS idx_audit_log_resource ON public.audit_log(resource_type, resource_id);

-- Grant permissions on audit log
GRANT INSERT, SELECT ON public.audit_log TO opensim;
GRANT SELECT ON public.audit_log TO opensim_readonly;

-- Create function to automatically update config timestamps
CREATE OR REPLACE FUNCTION update_config_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for config updates
DROP TRIGGER IF EXISTS trigger_update_config_timestamp ON public.server_config;
CREATE TRIGGER trigger_update_config_timestamp
    BEFORE UPDATE ON public.server_config
    FOR EACH ROW
    EXECUTE FUNCTION update_config_timestamp();

-- Create function for audit logging
CREATE OR REPLACE FUNCTION log_audit_trail()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        INSERT INTO public.audit_log (action, resource_type, resource_id, old_values)
        VALUES (TG_OP, TG_TABLE_NAME, OLD.id::text, row_to_json(OLD));
        RETURN OLD;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO public.audit_log (action, resource_type, resource_id, old_values, new_values)
        VALUES (TG_OP, TG_TABLE_NAME, NEW.id::text, row_to_json(OLD), row_to_json(NEW));
        RETURN NEW;
    ELSIF TG_OP = 'INSERT' THEN
        INSERT INTO public.audit_log (action, resource_type, resource_id, new_values)
        VALUES (TG_OP, TG_TABLE_NAME, NEW.id::text, row_to_json(NEW));
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Enable audit logging for important tables (will be created later by OpenSim)
-- These triggers will be created when the tables exist
-- CREATE TRIGGER audit_users AFTER INSERT OR UPDATE OR DELETE ON users FOR EACH ROW EXECUTE FUNCTION log_audit_trail();
-- CREATE TRIGGER audit_regions AFTER INSERT OR UPDATE OR DELETE ON regions FOR EACH ROW EXECUTE FUNCTION log_audit_trail();

-- Create database statistics view for monitoring
CREATE OR REPLACE VIEW monitoring.database_stats AS
SELECT 
    schemaname,
    tablename,
    n_tup_ins AS inserts,
    n_tup_upd AS updates,
    n_tup_del AS deletes,
    n_live_tup AS live_tuples,
    n_dead_tup AS dead_tuples,
    last_vacuum,
    last_autovacuum,
    last_analyze,
    last_autoanalyze
FROM pg_stat_user_tables
ORDER BY schemaname, tablename;

-- Grant access to monitoring view
GRANT SELECT ON monitoring.database_stats TO opensim_readonly;

-- Create connection monitoring view
CREATE OR REPLACE VIEW monitoring.connection_stats AS
SELECT 
    datname AS database,
    usename AS username,
    client_addr,
    client_port,
    backend_start,
    query_start,
    state,
    query
FROM pg_stat_activity
WHERE datname = 'opensim';

-- Grant access to connection monitoring view (requires superuser, will be limited)
-- GRANT SELECT ON monitoring.connection_stats TO opensim_readonly;

-- Final permissions setup
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO opensim;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA monitoring TO opensim;

-- Ensure opensim user can create tables in public schema
GRANT CREATE ON SCHEMA public TO opensim;

-- Set default permissions for future objects
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO opensim_readonly;
ALTER DEFAULT PRIVILEGES IN SCHEMA monitoring GRANT SELECT ON TABLES TO opensim_readonly;

-- Create maintenance procedures
CREATE OR REPLACE FUNCTION maintenance.vacuum_analyze_all()
RETURNS TEXT AS $$
DECLARE
    result TEXT := '';
    rec RECORD;
BEGIN
    FOR rec IN 
        SELECT schemaname, tablename 
        FROM pg_tables 
        WHERE schemaname IN ('public', 'monitoring')
    LOOP
        EXECUTE 'VACUUM ANALYZE ' || quote_ident(rec.schemaname) || '.' || quote_ident(rec.tablename);
        result := result || rec.schemaname || '.' || rec.tablename || ' ';
    END LOOP;
    
    RETURN 'Vacuumed and analyzed: ' || result;
END;
$$ LANGUAGE plpgsql;

-- Log initialization completion
INSERT INTO public.audit_log (action, resource_type, resource_id, new_values) 
VALUES ('DATABASE_INIT', 'postgresql', 'opensim', '{"status": "completed", "version": "1.0.0"}');

-- Display initialization summary
DO $$
BEGIN
    RAISE NOTICE 'OpenSim Next PostgreSQL initialization completed successfully';
    RAISE NOTICE 'Created users: opensim (main), opensim_readonly (monitoring), opensim_backup (backup)';
    RAISE NOTICE 'Created schemas: public (main), monitoring (metrics)';
    RAISE NOTICE 'Created monitoring tables: server_stats, region_stats, performance_metrics';
    RAISE NOTICE 'Created configuration system with audit logging';
    RAISE NOTICE 'Ready for OpenSim Next server connection';
END $$;