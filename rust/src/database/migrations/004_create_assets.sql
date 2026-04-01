-- Asset storage and management
CREATE TABLE IF NOT EXISTS assets (
    id UUID PRIMARY KEY,
    asset_name VARCHAR(255) NOT NULL,
    description VARCHAR(255) DEFAULT '',
    asset_type INTEGER NOT NULL,
    local BOOLEAN DEFAULT TRUE,
    temporary BOOLEAN DEFAULT FALSE,
    data BYTEA, -- Binary asset data
    data_length INTEGER DEFAULT 0,
    creator_id UUID,
    create_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    access_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    flags INTEGER DEFAULT 0
);

-- Asset metadata for optimization
CREATE TABLE IF NOT EXISTS asset_metadata (
    asset_id UUID PRIMARY KEY REFERENCES assets(id) ON DELETE CASCADE,
    content_type VARCHAR(128),
    sha256_hash VARCHAR(64),
    file_extension VARCHAR(10),
    compression_type INTEGER DEFAULT 0,
    original_size INTEGER DEFAULT 0,
    compressed_size INTEGER DEFAULT 0,
    upload_ip VARCHAR(45),
    uploader_id UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Asset types lookup
CREATE TABLE IF NOT EXISTS asset_types (
    type_id INTEGER PRIMARY KEY,
    type_name VARCHAR(64) NOT NULL,
    description VARCHAR(255) DEFAULT '',
    mime_type VARCHAR(128) DEFAULT ''
);

-- Insert standard asset types
INSERT INTO asset_types (type_id, type_name, description, mime_type) VALUES
    (-1, 'Unknown', 'Unknown asset type', 'application/octet-stream'),
    (0, 'Texture', 'Image texture', 'image/jpeg'),
    (1, 'Sound', 'Audio file', 'audio/wav'),
    (2, 'CallingCard', 'Calling card', 'application/octet-stream'),
    (3, 'Landmark', 'Landmark data', 'application/octet-stream'),
    (5, 'Clothing', 'Clothing item', 'application/octet-stream'),
    (6, 'Object', 'Primitive object', 'application/octet-stream'),
    (7, 'Notecard', 'Text notecard', 'text/plain'),
    (8, 'Folder', 'Category folder', 'application/octet-stream'),
    (10, 'LSLText', 'LSL script text', 'text/plain'),
    (11, 'LSLBytecode', 'Compiled LSL', 'application/octet-stream'),
    (12, 'TextureTGA', 'TGA texture', 'image/tga'),
    (13, 'Bodypart', 'Avatar body part', 'application/octet-stream'),
    (17, 'SoundWAV', 'WAV audio', 'audio/wav'),
    (19, 'ImageTGA', 'TGA image', 'image/tga'),
    (20, 'ImageJPEG', 'JPEG image', 'image/jpeg'),
    (21, 'Animation', 'Animation data', 'application/octet-stream'),
    (22, 'Gesture', 'Gesture data', 'application/octet-stream'),
    (24, 'Simstate', 'Simulator state', 'application/octet-stream'),
    (25, 'FavoriteFolder', 'Favorite folder', 'application/octet-stream'),
    (26, 'Link', 'Inventory link', 'application/octet-stream'),
    (27, 'LinkFolder', 'Folder link', 'application/octet-stream'),
    (28, 'CurrentOutfitFolder', 'Current outfit', 'application/octet-stream'),
    (46, 'OutfitFolder', 'Outfit folder', 'application/octet-stream'),
    (47, 'MyOutfitsFolder', 'My outfits folder', 'application/octet-stream'),
    (49, 'Mesh', '3D mesh data', 'application/octet-stream')
ON CONFLICT (type_id) DO NOTHING;

-- Asset references - track what assets are used where
CREATE TABLE IF NOT EXISTS asset_references (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    referencing_type VARCHAR(32) NOT NULL, -- 'inventory', 'scene_object', 'user_profile', etc.
    referencing_id UUID NOT NULL, -- ID of the referencing entity
    reference_field VARCHAR(64), -- Which field references this asset
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Asset access log for usage tracking
CREATE TABLE IF NOT EXISTS asset_access_log (
    id SERIAL PRIMARY KEY,
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    accessor_id UUID, -- User or system that accessed the asset
    accessor_type VARCHAR(32) DEFAULT 'user', -- 'user', 'system', 'script'
    access_type VARCHAR(16) DEFAULT 'read', -- 'read', 'write', 'delete'
    client_ip VARCHAR(45),
    user_agent VARCHAR(255),
    access_time TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Asset storage locations (for distributed storage)
CREATE TABLE IF NOT EXISTS asset_storage (
    asset_id UUID PRIMARY KEY REFERENCES assets(id) ON DELETE CASCADE,
    storage_type VARCHAR(32) DEFAULT 'database', -- 'database', 'filesystem', 's3', etc.
    storage_path VARCHAR(512), -- Path or URL to the actual asset data
    storage_server VARCHAR(255), -- Server/node where asset is stored
    backup_locations TEXT[], -- Array of backup storage locations
    checksum VARCHAR(64), -- File integrity checksum
    last_verified TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_assets_type ON assets(asset_type);
CREATE INDEX IF NOT EXISTS idx_assets_creator ON assets(creator_id);
CREATE INDEX IF NOT EXISTS idx_assets_create_time ON assets(create_time);
CREATE INDEX IF NOT EXISTS idx_assets_access_time ON assets(access_time);
CREATE INDEX IF NOT EXISTS idx_asset_metadata_hash ON asset_metadata(sha256_hash);
CREATE INDEX IF NOT EXISTS idx_asset_metadata_uploader ON asset_metadata(uploader_id);
CREATE INDEX IF NOT EXISTS idx_asset_references_asset ON asset_references(asset_id);
CREATE INDEX IF NOT EXISTS idx_asset_references_referencing ON asset_references(referencing_type, referencing_id);
CREATE INDEX IF NOT EXISTS idx_asset_access_log_asset ON asset_access_log(asset_id);
CREATE INDEX IF NOT EXISTS idx_asset_access_log_accessor ON asset_access_log(accessor_id);
CREATE INDEX IF NOT EXISTS idx_asset_access_log_time ON asset_access_log(access_time);