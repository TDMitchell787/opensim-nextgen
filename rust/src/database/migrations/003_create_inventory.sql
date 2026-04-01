-- User inventory system
CREATE TABLE IF NOT EXISTS inventory_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    parent_id UUID, -- NULL for root folder
    folder_name VARCHAR(255) NOT NULL,
    folder_type INTEGER DEFAULT 0,
    version INTEGER DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Inventory items
CREATE TABLE IF NOT EXISTS inventory_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    folder_id UUID NOT NULL REFERENCES inventory_folders(id) ON DELETE CASCADE,
    asset_id UUID NOT NULL,
    asset_type INTEGER NOT NULL,
    inventory_type INTEGER NOT NULL,
    item_name VARCHAR(255) NOT NULL,
    description VARCHAR(255) DEFAULT '',
    creation_date TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    creator_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    last_owner_id UUID,
    group_id UUID,
    group_owned BOOLEAN DEFAULT FALSE,
    sale_price INTEGER DEFAULT 0,
    sale_type INTEGER DEFAULT 0,
    flags INTEGER DEFAULT 0,
    base_permissions INTEGER DEFAULT 0,
    current_permissions INTEGER DEFAULT 0,
    everyone_permissions INTEGER DEFAULT 0,
    group_permissions INTEGER DEFAULT 0,
    next_permissions INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Wearable items (clothing, body parts)
CREATE TABLE IF NOT EXISTS inventory_wearables (
    item_id UUID PRIMARY KEY REFERENCES inventory_items(id) ON DELETE CASCADE,
    wearable_type INTEGER NOT NULL,
    permissions INTEGER DEFAULT 0,
    for_sale INTEGER DEFAULT 0,
    sale_price INTEGER DEFAULT 0,
    wearable_data TEXT -- Serialized wearable parameters
);

-- Gesture items
CREATE TABLE IF NOT EXISTS inventory_gestures (
    item_id UUID PRIMARY KEY REFERENCES inventory_items(id) ON DELETE CASCADE,
    gesture_data BYTEA, -- Binary gesture data
    active BOOLEAN DEFAULT FALSE
);

-- Landmark items
CREATE TABLE IF NOT EXISTS inventory_landmarks (
    item_id UUID PRIMARY KEY REFERENCES inventory_items(id) ON DELETE CASCADE,
    region_id UUID,
    position_x REAL DEFAULT 128.0,
    position_y REAL DEFAULT 128.0,
    position_z REAL DEFAULT 25.0,
    region_name VARCHAR(255),
    parcel_name VARCHAR(255),
    global_x REAL DEFAULT 0.0,
    global_y REAL DEFAULT 0.0
);

-- Notecard items
CREATE TABLE IF NOT EXISTS inventory_notecards (
    item_id UUID PRIMARY KEY REFERENCES inventory_items(id) ON DELETE CASCADE,
    text_content TEXT DEFAULT '',
    embedded_items TEXT -- JSON array of embedded inventory items
);

-- Script items
CREATE TABLE IF NOT EXISTS inventory_scripts (
    item_id UUID PRIMARY KEY REFERENCES inventory_items(id) ON DELETE CASCADE,
    script_source TEXT NOT NULL,
    bytecode BYTEA, -- Compiled script bytecode
    running_state INTEGER DEFAULT 0, -- 0=not running, 1=running, 2=suspended
    compile_errors TEXT,
    last_compiled TIMESTAMP WITH TIME ZONE,
    memory_usage INTEGER DEFAULT 0,
    script_delay REAL DEFAULT 0.0
);

-- Object (prim) inventory - items inside objects
CREATE TABLE IF NOT EXISTS object_inventory (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    object_id UUID NOT NULL, -- References scene_objects.group_id
    item_id UUID NOT NULL,
    asset_id UUID NOT NULL,
    asset_type INTEGER NOT NULL,
    inventory_type INTEGER NOT NULL,
    item_name VARCHAR(255) NOT NULL,
    description VARCHAR(255) DEFAULT '',
    creator_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    last_owner_id UUID,
    group_id UUID,
    group_owned BOOLEAN DEFAULT FALSE,
    base_permissions INTEGER DEFAULT 0,
    current_permissions INTEGER DEFAULT 0,
    everyone_permissions INTEGER DEFAULT 0,
    group_permissions INTEGER DEFAULT 0,
    next_permissions INTEGER DEFAULT 0,
    flags INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Inventory categories/types lookup
CREATE TABLE IF NOT EXISTS inventory_types (
    type_id INTEGER PRIMARY KEY,
    type_name VARCHAR(64) NOT NULL,
    description VARCHAR(255) DEFAULT ''
);

-- Insert standard inventory types
INSERT INTO inventory_types (type_id, type_name, description) VALUES
    (-1, 'None', 'No specific type'),
    (0, 'Texture', 'Image texture'),
    (1, 'Sound', 'Audio file'),
    (3, 'Landmark', 'Location bookmark'),
    (5, 'Clothing', 'Wearable clothing'),
    (6, 'Object', '3D object/primitive'),
    (7, 'Notecard', 'Text document'),
    (10, 'LSLText', 'LSL script source'),
    (13, 'BodyPart', 'Avatar body part'),
    (20, 'Animation', 'Animation file'),
    (21, 'Gesture', 'Avatar gesture'),
    (22, 'Simstate', 'Simulator state file'),
    (24, 'Link', 'Symbolic link'),
    (25, 'LinkFolder', 'Folder link')
ON CONFLICT (type_id) DO NOTHING;

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_inventory_folders_user ON inventory_folders(user_id);
CREATE INDEX IF NOT EXISTS idx_inventory_folders_parent ON inventory_folders(parent_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_user ON inventory_items(user_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_folder ON inventory_items(folder_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_asset ON inventory_items(asset_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_owner ON inventory_items(owner_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_creator ON inventory_items(creator_id);
CREATE INDEX IF NOT EXISTS idx_object_inventory_object ON object_inventory(object_id);
CREATE INDEX IF NOT EXISTS idx_object_inventory_owner ON object_inventory(owner_id);