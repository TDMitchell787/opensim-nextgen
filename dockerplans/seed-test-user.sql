-- OpenSim NextGen — Test User Seed
-- Creates a test user account after migrations have run.
-- Login: First=Test Last=User Password=test_password
--
-- Password hash format (OpenSim-compatible salted MD5):
--   passwordhash = md5( md5(plaintext) || ':' || salt )
--   The viewer sends "$1$<md5(plaintext)>"; the server strips "$1$",
--   recomputes md5(viewer_hash || ':' || salt) and compares to passwordhash.
--   See rust/src/database/user_accounts.rs::authenticate_user_opensim.
--
-- All identifiers are unquoted/lowercase to match the schema produced by
-- the SQLx migrations (Postgres folds unquoted identifiers to lowercase).
-- The salt is 32 hex chars to fit `passwordsalt character(32)`.

DO $$
DECLARE
    test_user_id UUID := 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee';
    test_salt CHAR(32) := 'a1b2c3d4e5f67890abcdef1234567890';
    password_hash TEXT;
    root_folder_id UUID := 'aaaaaaaa-bbbb-cccc-dddd-ffffffffffff';
BEGIN
    password_hash := md5(md5('test_password') || ':' || test_salt);

    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'useraccounts') THEN
        INSERT INTO useraccounts (principalid, scopeid, firstname, lastname, email,
            serviceurls, created, userlevel, userflags, usertitle, active)
        VALUES (
            test_user_id,
            '00000000-0000-0000-0000-000000000000',
            'Test', 'User',
            'test@opensim-nextgen.local',
            'HomeURI= InventoryServerURI= AssetServerURI=',
            EXTRACT(EPOCH FROM NOW())::INTEGER,
            0, 0, '', 1
        )
        ON CONFLICT (principalid) DO NOTHING;

        RAISE NOTICE 'useraccounts: Test User created or already exists';
    ELSE
        RAISE NOTICE 'useraccounts table not found — migrations may not have run yet';
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'auth') THEN
        INSERT INTO auth (uuid, passwordhash, passwordsalt, webloginkey, accounttype)
        VALUES (
            test_user_id,
            password_hash,
            test_salt,
            '',
            'UserAccount'
        )
        ON CONFLICT (uuid) DO NOTHING;

        RAISE NOTICE 'auth: credentials created for Test User';
    ELSE
        RAISE NOTICE 'auth table not found — migrations may not have run yet';
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'inventoryfolders') THEN
        INSERT INTO inventoryfolders (folderid, agentid, parentfolderid, foldername, type, version)
        VALUES
            (root_folder_id, test_user_id, '00000000-0000-0000-0000-000000000000', 'My Inventory', 8, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Animations', 20, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Body Parts', 13, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Calling Cards', 2, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Clothing', 5, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Current Outfit', 46, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Favorites', 23, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Gestures', 21, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Landmarks', 3, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Lost And Found', 16, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Notecards', 7, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Objects', 6, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Photo Album', 15, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Scripts', 10, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Sounds', 1, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Textures', 0, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Trash', 14, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Settings', 56, 1),
            (uuid_generate_v4(), test_user_id, root_folder_id, 'Materials', 57, 1)
        ON CONFLICT (folderid) DO NOTHING;

        RAISE NOTICE 'inventoryfolders: 19 folders created for Test User';
    ELSE
        RAISE NOTICE 'inventoryfolders table not found — migrations may not have run yet';
    END IF;

END $$;
