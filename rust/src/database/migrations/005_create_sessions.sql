-- Extended session and presence management
CREATE TABLE IF NOT EXISTS user_presence (
    user_id UUID PRIMARY KEY,
    region_id UUID,
    session_id UUID UNIQUE NOT NULL,
    secure_session_id UUID UNIQUE NOT NULL,
    position_x REAL DEFAULT 128.0,
    position_y REAL DEFAULT 128.0,
    position_z REAL DEFAULT 25.0,
    look_at_x REAL DEFAULT 1.0,
    look_at_y REAL DEFAULT 0.0,
    look_at_z REAL DEFAULT 0.0,
    velocity_x REAL DEFAULT 0.0,
    velocity_y REAL DEFAULT 0.0,
    velocity_z REAL DEFAULT 0.0,
    avatar_type VARCHAR(32) DEFAULT 'User',
    login_flags INTEGER DEFAULT 0,
    login_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    logout_time TIMESTAMP WITH TIME ZONE,
    last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_online BOOLEAN DEFAULT TRUE
);

-- Grid presence for inter-region tracking
CREATE TABLE IF NOT EXISTS grid_presence (
    user_id UUID NOT NULL,
    region_id UUID NOT NULL,
    position_data TEXT, -- JSON serialized position data
    login_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_update TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY(user_id, region_id)
);

-- User preferences and client settings
CREATE TABLE IF NOT EXISTS user_preferences (
    user_id UUID PRIMARY KEY,
    im_via_email BOOLEAN DEFAULT FALSE,
    visible_online BOOLEAN DEFAULT TRUE,
    email_on_friend_request BOOLEAN DEFAULT TRUE,
    list_in_directory BOOLEAN DEFAULT TRUE,
    adult_rating INTEGER DEFAULT 0, -- 0=PG, 1=Mature, 2=Adult
    language VARCHAR(10) DEFAULT 'en',
    hover_height REAL DEFAULT 0.0,
    daylight_savings BOOLEAN DEFAULT TRUE,
    ever_logged_in BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Authentication tokens and API keys
CREATE TABLE IF NOT EXISTS auth_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    token_type VARCHAR(32) NOT NULL, -- 'session', 'api', 'temp', 'reset'
    token_value VARCHAR(255) UNIQUE NOT NULL,
    token_secret VARCHAR(255), -- For OAuth-style tokens
    scope VARCHAR(255) DEFAULT 'user', -- Permissions scope
    expires_at TIMESTAMP WITH TIME ZONE,
    revoked_at TIMESTAMP WITH TIME ZONE,
    last_used TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB -- Additional token metadata
);

-- Login history for security/audit
CREATE TABLE IF NOT EXISTS login_history (
    id SERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    login_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    logout_time TIMESTAMP WITH TIME ZONE,
    client_ip VARCHAR(45),
    client_version VARCHAR(255),
    viewer_name VARCHAR(128),
    platform VARCHAR(64),
    mac_address VARCHAR(17), -- MAC address if available
    region_id UUID, -- Login region
    success BOOLEAN DEFAULT TRUE,
    failure_reason VARCHAR(255),
    session_duration INTERVAL
);

-- User groups and roles
CREATE TABLE IF NOT EXISTS user_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_name VARCHAR(128) UNIQUE NOT NULL,
    charter TEXT DEFAULT '',
    insignia_id UUID, -- Group logo asset
    founder_id UUID NOT NULL,
    membership_fee INTEGER DEFAULT 0,
    open_enrollment BOOLEAN DEFAULT FALSE,
    show_in_list BOOLEAN DEFAULT TRUE,
    allow_publish BOOLEAN DEFAULT TRUE,
    mature_publish BOOLEAN DEFAULT FALSE,
    owner_role_id UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Group membership
CREATE TABLE IF NOT EXISTS group_membership (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    role_id UUID,
    selected_role_id UUID,
    contribution INTEGER DEFAULT 0,
    list_in_profile BOOLEAN DEFAULT TRUE,
    accept_notices BOOLEAN DEFAULT TRUE,
    access_token VARCHAR(255),
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(group_id, user_id)
);

-- Group roles and permissions
CREATE TABLE IF NOT EXISTS group_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    role_name VARCHAR(128) NOT NULL,
    description VARCHAR(255) DEFAULT '',
    title VARCHAR(128) DEFAULT '',
    powers BIGINT DEFAULT 0, -- Bitmask of permissions
    is_owner BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User bans and restrictions
CREATE TABLE IF NOT EXISTS user_bans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    banned_by UUID NOT NULL,
    ban_type VARCHAR(32) DEFAULT 'region', -- 'region', 'grid', 'estate'
    target_id UUID, -- Region, estate, or grid ID
    ban_flags INTEGER DEFAULT 0,
    ban_reason TEXT DEFAULT '',
    ban_start TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    ban_end TIMESTAMP WITH TIME ZONE, -- NULL for permanent
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_presence_region ON user_presence(region_id);
CREATE INDEX IF NOT EXISTS idx_user_presence_session ON user_presence(session_id);
CREATE INDEX IF NOT EXISTS idx_user_presence_online ON user_presence(is_online);
CREATE INDEX IF NOT EXISTS idx_grid_presence_user ON grid_presence(user_id);
CREATE INDEX IF NOT EXISTS idx_grid_presence_region ON grid_presence(region_id);
CREATE INDEX IF NOT EXISTS idx_auth_tokens_user ON auth_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_auth_tokens_type ON auth_tokens(token_type);
CREATE INDEX IF NOT EXISTS idx_auth_tokens_expires ON auth_tokens(expires_at);
CREATE INDEX IF NOT EXISTS idx_login_history_user ON login_history(user_id);
CREATE INDEX IF NOT EXISTS idx_login_history_time ON login_history(login_time);
CREATE INDEX IF NOT EXISTS idx_group_membership_group ON group_membership(group_id);
CREATE INDEX IF NOT EXISTS idx_group_membership_user ON group_membership(user_id);
CREATE INDEX IF NOT EXISTS idx_group_roles_group ON group_roles(group_id);
CREATE INDEX IF NOT EXISTS idx_user_bans_user ON user_bans(user_id);
CREATE INDEX IF NOT EXISTS idx_user_bans_active ON user_bans(is_active, ban_end);