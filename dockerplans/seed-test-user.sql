-- OpenSim NextGen — Test User Seed
-- Creates a test user account after migrations have run
-- Login: First=Test Last=User Password=test_password
--
-- Password hash format (OpenSim compatible):
--   passwordHash = md5( md5(plaintext_password) || ':' || salt )
--   The viewer sends $1$<md5(password)> and the server verifies against this.

DO $$
DECLARE
    test_user_id UUID := 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee';
    test_salt TEXT := 'a1b2c3d4-e5f6-7890-abcd-ef1234567890';
    password_md5 TEXT;
    password_hash TEXT;
    root_folder_id UUID := 'aaaaaaaa-bbbb-cccc-dddd-ffffffffffff';
BEGIN
    -- Compute password hash: md5(md5('test_password') || ':' || salt)
    password_md5 := md5('test_password');
    password_hash := md5(password_md5 || ':' || test_salt);

    -- Insert into UserAccounts (if table exists and user doesn't already exist)
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'useraccounts') THEN
        INSERT INTO "UserAccounts" ("PrincipalID", "ScopeID", "FirstName", "LastName", "Email",
            "ServiceURLs", "Created", "UserLevel", "UserFlags", "UserTitle", "active")
        VALUES (
            test_user_id,
            '00000000-0000-0000-0000-000000000000',
            'Test', 'User',
            'test@opensim-nextgen.local',
            'HomeURI= InventoryServerURI= AssetServerURI=',
            EXTRACT(EPOCH FROM NOW())::INTEGER,
            0, 0, '', 1
        )
        ON CONFLICT ("PrincipalID") DO NOTHING;

        RAISE NOTICE 'UserAccounts: Test User created or already exists';
    ELSE
        RAISE NOTICE 'UserAccounts table not found — migrations may not have run yet';
    END IF;

    -- Insert into auth table
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'auth') THEN
        INSERT INTO auth (uuid, "passwordHash", "passwordSalt", "webLoginKey", "accountType")
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

    -- Create root inventory folder
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'inventoryfolders') THEN
        INSERT INTO inventoryfolders ("folderID", "agentID", "parentFolderID", "folderName", type, version)
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
        ON CONFLICT ("folderID") DO NOTHING;

        RAISE NOTICE 'inventoryfolders: 19 folders created for Test User';
    ELSE
        RAISE NOTICE 'inventoryfolders table not found — migrations may not have run yet';
    END IF;

END $$;
