-- User accounts and authentication
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(64) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    salt VARCHAR(32) NOT NULL,
    first_name VARCHAR(64) NOT NULL,
    last_name VARCHAR(64) NOT NULL,
    home_region_id UUID,
    home_position_x REAL DEFAULT 128.0,
    home_position_y REAL DEFAULT 128.0,
    home_position_z REAL DEFAULT 25.0,
    home_look_at_x REAL DEFAULT 1.0,
    home_look_at_y REAL DEFAULT 0.0,
    home_look_at_z REAL DEFAULT 0.0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_login TIMESTAMP WITH TIME ZONE,
    user_level INTEGER DEFAULT 0,
    user_flags INTEGER DEFAULT 0,
    god_level INTEGER DEFAULT 0,
    custom_type VARCHAR(32) DEFAULT 'UserAccount'
);

-- User profiles
CREATE TABLE IF NOT EXISTS user_profiles (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    about_text TEXT DEFAULT '',
    first_life_about_text TEXT DEFAULT '',
    image_id UUID,
    first_life_image_id UUID,
    web_url VARCHAR(512) DEFAULT '',
    wants_to_mask INTEGER DEFAULT 0,
    wants_to_text VARCHAR(255) DEFAULT '',
    skills_mask INTEGER DEFAULT 0,
    skills_text VARCHAR(255) DEFAULT '',
    languages VARCHAR(255) DEFAULT '',
    partner_id UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User sessions for authentication
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token VARCHAR(255) UNIQUE NOT NULL,
    secure_session_token VARCHAR(255) UNIQUE NOT NULL,
    region_id UUID,
    position_x REAL DEFAULT 128.0,
    position_y REAL DEFAULT 128.0,
    position_z REAL DEFAULT 25.0,
    look_at_x REAL DEFAULT 1.0,
    look_at_y REAL DEFAULT 0.0,
    look_at_z REAL DEFAULT 0.0,
    login_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    logout_time TIMESTAMP WITH TIME ZONE,
    last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

-- User friends/relationships
CREATE TABLE IF NOT EXISTS user_friends (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    friend_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    flags INTEGER DEFAULT 0,
    offered INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, friend_id)
);

-- User avatar appearance
CREATE TABLE IF NOT EXISTS user_appearance (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    appearance_data TEXT, -- Serialized appearance data
    wearables TEXT, -- Serialized wearables data
    attachments TEXT, -- Serialized attachments data
    visual_params BYTEA, -- Binary visual parameters
    texture_data TEXT, -- Serialized texture data
    avatar_height REAL DEFAULT 1.8,
    hip_offset REAL DEFAULT 0.0,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_user_sessions_token ON user_sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_user_friends_user_id ON user_friends(user_id);
CREATE INDEX IF NOT EXISTS idx_user_friends_friend_id ON user_friends(friend_id);