import 'dart:convert';
import 'package:http/http.dart' as http;
import '../models/console_command_models.dart';

class ConsoleService {
  final String baseUrl;
  final String? apiKey;

  ConsoleService({
    required this.baseUrl,
    this.apiKey,
  });

  Map<String, String> get _headers => {
        'Content-Type': 'application/json',
        if (apiKey != null) 'X-API-Key': apiKey!,
      };

  /// Get all available commands organized by group
  Map<CommandGroup, List<ConsoleCommand>> getCommandsByGroup() {
    final commands = getAllCommands();
    final grouped = <CommandGroup, List<ConsoleCommand>>{};

    for (final cmd in commands) {
      grouped.putIfAbsent(cmd.group, () => []).add(cmd);
    }

    return grouped;
  }

  /// Get all available console commands
  List<ConsoleCommand> getAllCommands() {
    return [
      // ===== USERS (12 commands) =====
      ConsoleCommand(
        name: 'create user',
        group: CommandGroup.users,
        description: 'Create a new user account',
        syntax: 'create user [firstname] [lastname] [password] [email]',
        implemented: true,
        apiEndpoint: '/admin/users',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'firstname', description: 'First name', type: ParamType.string),
          CommandParam(name: 'lastname', description: 'Last name', type: ParamType.string),
          CommandParam(name: 'password', description: 'Password', type: ParamType.string),
          CommandParam(name: 'email', description: 'Email address', type: ParamType.string),
          CommandParam(name: 'user_level', description: 'User level (0=User, 100=Admin, 200=God)', type: ParamType.integer, required: false, defaultValue: '0'),
        ],
      ),
      ConsoleCommand(
        name: 'show users',
        group: CommandGroup.users,
        description: 'List all user accounts',
        syntax: 'show users [limit]',
        implemented: true,
        apiEndpoint: '/admin/users',
        httpMethod: 'GET',
        params: [
          CommandParam(name: 'limit', description: 'Maximum users to show', type: ParamType.integer, required: false, defaultValue: '100'),
        ],
      ),
      ConsoleCommand(
        name: 'show account',
        group: CommandGroup.users,
        description: 'Display user account details',
        syntax: 'show account [firstname] [lastname]',
        implemented: true,
        apiEndpoint: '/admin/users/account',
        httpMethod: 'GET',
        params: [
          CommandParam(name: 'firstname', description: 'First name', type: ParamType.string),
          CommandParam(name: 'lastname', description: 'Last name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'reset user password',
        group: CommandGroup.users,
        description: 'Reset user password',
        syntax: 'reset user password [firstname] [lastname] [newpassword]',
        implemented: true,
        apiEndpoint: '/admin/users/password',
        httpMethod: 'PUT',
        params: [
          CommandParam(name: 'firstname', description: 'First name', type: ParamType.string),
          CommandParam(name: 'lastname', description: 'Last name', type: ParamType.string),
          CommandParam(name: 'new_password', description: 'New password', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'set user level',
        group: CommandGroup.users,
        description: 'Set user permission level',
        syntax: 'set user level [firstname] [lastname] [level]',
        implemented: true,
        apiEndpoint: '/admin/users/level',
        httpMethod: 'PUT',
        params: [
          CommandParam(name: 'firstname', description: 'First name', type: ParamType.string),
          CommandParam(name: 'lastname', description: 'Last name', type: ParamType.string),
          CommandParam(name: 'user_level', description: 'Level (0-255)', type: ParamType.integer),
        ],
      ),
      ConsoleCommand(
        name: 'delete user',
        group: CommandGroup.users,
        description: 'Delete user account permanently',
        syntax: 'delete user [firstname] [lastname]',
        implemented: true,
        apiEndpoint: '/admin/users/delete',
        httpMethod: 'DELETE',
        params: [
          CommandParam(name: 'firstname', description: 'First name', type: ParamType.string),
          CommandParam(name: 'lastname', description: 'Last name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'kick user',
        group: CommandGroup.users,
        description: 'Kick user from region',
        syntax: 'kick user [firstname] [lastname] [message]',
        implemented: true,
        apiEndpoint: '/console/users/kick',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'firstname', description: 'First name', type: ParamType.string),
          CommandParam(name: 'lastname', description: 'Last name', type: ParamType.string),
          CommandParam(name: 'message', description: 'Kick message', type: ParamType.string, required: false),
        ],
      ),
      ConsoleCommand(
        name: 'login level',
        group: CommandGroup.users,
        description: 'Set minimum login level',
        syntax: 'login level [level]',
        implemented: true,
        apiEndpoint: '/console/login/level',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'level', description: 'Minimum level (0-255)', type: ParamType.integer),
        ],
      ),
      ConsoleCommand(
        name: 'login reset',
        group: CommandGroup.users,
        description: 'Reset login restrictions',
        syntax: 'login reset',
        implemented: true,
        apiEndpoint: '/console/login/reset',
        httpMethod: 'POST',
        params: [],
      ),
      ConsoleCommand(
        name: 'login text',
        group: CommandGroup.users,
        description: 'Set login text message',
        syntax: 'login text [message]',
        implemented: true,
        apiEndpoint: '/console/login/text',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'message', description: 'Login text to display', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'show grid user',
        group: CommandGroup.users,
        description: 'Show grid user details',
        syntax: 'show grid user [firstname] [lastname]',
        implemented: true,
        apiEndpoint: '/console/users/grid-user',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'firstname', description: 'First name', type: ParamType.string),
          CommandParam(name: 'lastname', description: 'Last name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'show grid users online',
        group: CommandGroup.users,
        description: 'Show all online grid users',
        syntax: 'show grid users online',
        implemented: true,
        apiEndpoint: '/console/users/grid-users-online',
        httpMethod: 'GET',
        params: [],
      ),

      // ===== REGIONS (10 commands) =====
      ConsoleCommand(
        name: 'show regions',
        group: CommandGroup.regions,
        description: 'List all regions',
        syntax: 'show regions',
        implemented: true,
        apiEndpoint: '/console/regions',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'show region',
        group: CommandGroup.regions,
        description: 'Show region details',
        syntax: 'show region [name]',
        implemented: true,
        apiEndpoint: '/console/regions/:name',
        httpMethod: 'GET',
        params: [
          CommandParam(name: 'name', description: 'Region name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'create region',
        group: CommandGroup.regions,
        description: 'Create a new region',
        syntax: 'create region [name] [template]',
        implemented: true,
        apiEndpoint: '/console/regions/create',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'name', description: 'Region name', type: ParamType.string),
          CommandParam(name: 'template', description: 'Template to use', type: ParamType.string, required: false),
        ],
      ),
      ConsoleCommand(
        name: 'restart',
        group: CommandGroup.regions,
        description: 'Restart region',
        syntax: 'restart [delay]',
        implemented: true,
        apiEndpoint: '/console/regions/:name/restart',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'name', description: 'Region name', type: ParamType.string),
          CommandParam(name: 'delay', description: 'Delay in seconds', type: ParamType.integer, required: false, defaultValue: '30'),
        ],
      ),
      ConsoleCommand(
        name: 'delete-region',
        group: CommandGroup.regions,
        description: 'Delete region permanently',
        syntax: 'delete-region [name]',
        implemented: true,
        apiEndpoint: '/console/regions/delete',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'name', description: 'Region name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'change region',
        group: CommandGroup.regions,
        description: 'Change to a different region',
        syntax: 'change region [name]',
        implemented: true,
        apiEndpoint: '/console/regions/change',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'name', description: 'Region name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'show ratings',
        group: CommandGroup.regions,
        description: 'Show region ratings',
        syntax: 'show ratings',
        implemented: true,
        apiEndpoint: '/console/regions/ratings',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'show neighbours',
        group: CommandGroup.regions,
        description: 'Show region neighbours',
        syntax: 'show neighbours',
        implemented: true,
        apiEndpoint: '/console/regions/neighbours',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'show regions in view',
        group: CommandGroup.regions,
        description: 'Show regions in view distance',
        syntax: 'show regions in view',
        implemented: true,
        apiEndpoint: '/console/regions/inview',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'show connections',
        group: CommandGroup.regions,
        description: 'Show active connections',
        syntax: 'show connections',
        implemented: true,
        apiEndpoint: '/console/connections',
        httpMethod: 'GET',
        params: [],
      ),

      // ===== TERRAIN (18 commands) =====
      ConsoleCommand(
        name: 'terrain load',
        group: CommandGroup.terrain,
        description: 'Load terrain from file',
        syntax: 'terrain load [filename]',
        implemented: true,
        apiEndpoint: '/console/terrain/load',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'filename', description: 'Terrain file path', type: ParamType.file, placeholder: '/terrains/heightmap.raw'),
        ],
      ),
      ConsoleCommand(
        name: 'terrain save',
        group: CommandGroup.terrain,
        description: 'Save terrain to file',
        syntax: 'terrain save [filename]',
        implemented: true,
        apiEndpoint: '/console/terrain/save',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'filename', description: 'Output file path', type: ParamType.path, placeholder: '/terrains/backup.png'),
        ],
      ),
      ConsoleCommand(
        name: 'terrain fill',
        group: CommandGroup.terrain,
        description: 'Fill terrain with uniform height',
        syntax: 'terrain fill [height]',
        implemented: true,
        apiEndpoint: '/console/terrain/fill',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'height', description: 'Height value', type: ParamType.number, defaultValue: '25'),
          CommandParam(name: 'region', description: 'Region name filter (optional, applies to all if empty)', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'terrain elevate',
        group: CommandGroup.terrain,
        description: 'Raise terrain by amount',
        syntax: 'terrain elevate [amount] [region]',
        implemented: true,
        apiEndpoint: '/console/terrain/elevate',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'amount', description: 'Amount to raise', type: ParamType.number),
          CommandParam(name: 'region', description: 'Region name filter (optional, applies to all if empty)', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'terrain lower',
        group: CommandGroup.terrain,
        description: 'Lower terrain by amount',
        syntax: 'terrain lower [amount] [region]',
        implemented: true,
        apiEndpoint: '/console/terrain/lower',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'amount', description: 'Amount to lower', type: ParamType.number),
          CommandParam(name: 'region', description: 'Region name filter (optional, applies to all if empty)', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'terrain multiply',
        group: CommandGroup.terrain,
        description: 'Multiply terrain by factor',
        syntax: 'terrain multiply [factor] [region]',
        implemented: true,
        apiEndpoint: '/console/terrain/multiply',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'factor', description: 'Multiply factor', type: ParamType.number),
          CommandParam(name: 'region', description: 'Region name filter (optional, applies to all if empty)', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'terrain stats',
        group: CommandGroup.terrain,
        description: 'Show terrain statistics',
        syntax: 'terrain stats',
        implemented: true,
        apiEndpoint: '/console/terrain/stats',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'terrain bake',
        group: CommandGroup.terrain,
        description: 'Bake terrain to storage',
        syntax: 'terrain bake',
        implemented: true,
        apiEndpoint: '/console/terrain/bake',
        httpMethod: 'POST',
        params: [],
      ),
      ConsoleCommand(
        name: 'terrain revert',
        group: CommandGroup.terrain,
        description: 'Revert to last baked terrain',
        syntax: 'terrain revert',
        implemented: true,
        apiEndpoint: '/console/terrain/revert',
        httpMethod: 'POST',
        params: [],
      ),
      ConsoleCommand(
        name: 'terrain show',
        group: CommandGroup.terrain,
        description: 'Show terrain info',
        syntax: 'terrain show',
        implemented: true,
        apiEndpoint: '/console/terrain/show',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'terrain load-tile',
        group: CommandGroup.terrain,
        description: 'Load terrain tile',
        syntax: 'terrain load-tile [filename] [x] [y]',
        implemented: true,
        apiEndpoint: '/console/terrain/load-tile',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'filename', description: 'Tile file path', type: ParamType.file),
          CommandParam(name: 'x', description: 'X coordinate', type: ParamType.integer),
          CommandParam(name: 'y', description: 'Y coordinate', type: ParamType.integer),
        ],
      ),
      ConsoleCommand(
        name: 'terrain save-tile',
        group: CommandGroup.terrain,
        description: 'Save terrain tile',
        syntax: 'terrain save-tile [filename] [x] [y]',
        implemented: true,
        apiEndpoint: '/console/terrain/save-tile',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'filename', description: 'Output file path', type: ParamType.path),
          CommandParam(name: 'x', description: 'X coordinate', type: ParamType.integer),
          CommandParam(name: 'y', description: 'Y coordinate', type: ParamType.integer),
        ],
      ),
      ConsoleCommand(
        name: 'terrain effect',
        group: CommandGroup.terrain,
        description: 'Apply terrain effect',
        syntax: 'terrain effect [effect_name]',
        implemented: true,
        apiEndpoint: '/console/terrain/effect',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'effect', description: 'Effect name', type: ParamType.choice, choices: ['normalize', 'smooth', 'noise']),
        ],
      ),
      ConsoleCommand(
        name: 'terrain flip',
        group: CommandGroup.terrain,
        description: 'Flip terrain',
        syntax: 'terrain flip [direction]',
        implemented: true,
        apiEndpoint: '/console/terrain/flip',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'direction', description: 'Flip direction', type: ParamType.choice, choices: ['x', 'y']),
        ],
      ),
      ConsoleCommand(
        name: 'terrain rescale',
        group: CommandGroup.terrain,
        description: 'Rescale terrain heights',
        syntax: 'terrain rescale [min] [max]',
        implemented: true,
        apiEndpoint: '/console/terrain/rescale',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'min', description: 'Minimum height', type: ParamType.number),
          CommandParam(name: 'max', description: 'Maximum height', type: ParamType.number),
        ],
      ),
      ConsoleCommand(
        name: 'terrain min',
        group: CommandGroup.terrain,
        description: 'Set terrain minimum height',
        syntax: 'terrain min [value]',
        implemented: true,
        apiEndpoint: '/console/terrain/min',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'value', description: 'Minimum value', type: ParamType.number),
        ],
      ),
      ConsoleCommand(
        name: 'terrain max',
        group: CommandGroup.terrain,
        description: 'Set terrain maximum height',
        syntax: 'terrain max [value]',
        implemented: true,
        apiEndpoint: '/console/terrain/max',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'value', description: 'Maximum value', type: ParamType.number),
        ],
      ),
      ConsoleCommand(
        name: 'terrain modify',
        group: CommandGroup.terrain,
        description: 'Modify terrain at position',
        syntax: 'terrain modify [x] [y] [height]',
        implemented: true,
        apiEndpoint: '/console/terrain/modify',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'x', description: 'X coordinate', type: ParamType.integer),
          CommandParam(name: 'y', description: 'Y coordinate', type: ParamType.integer),
          CommandParam(name: 'height', description: 'Height value', type: ParamType.number),
        ],
      ),

      // ===== OBJECTS (12 commands) =====
      ConsoleCommand(
        name: 'backup',
        group: CommandGroup.objects,
        description: 'Backup region to OAR',
        syntax: 'backup',
        implemented: true,
        apiEndpoint: '/console/objects/backup',
        httpMethod: 'POST',
        params: [],
      ),
      ConsoleCommand(
        name: 'show object name',
        group: CommandGroup.objects,
        description: 'Find objects by name',
        syntax: 'show object name [name]',
        implemented: true,
        apiEndpoint: '/console/objects/show',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'search_type', description: 'Search type', type: ParamType.string, defaultValue: 'name'),
          CommandParam(name: 'name', description: 'Object name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'delete object name',
        group: CommandGroup.objects,
        description: 'Delete objects by name',
        syntax: 'delete object name [name]',
        implemented: true,
        apiEndpoint: '/console/objects/delete',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'delete_type', description: 'Delete type', type: ParamType.string, defaultValue: 'name'),
          CommandParam(name: 'name', description: 'Object name', type: ParamType.string),
          CommandParam(name: 'regex', description: 'Use regex matching', type: ParamType.boolean, required: false, defaultValue: 'false'),
        ],
      ),
      ConsoleCommand(
        name: 'delete object owner',
        group: CommandGroup.objects,
        description: 'Delete objects by owner',
        syntax: 'delete object owner [uuid]',
        implemented: true,
        apiEndpoint: '/console/objects/delete',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'delete_type', description: 'Delete type', type: ParamType.string, defaultValue: 'owner'),
          CommandParam(name: 'owner_id', description: 'Owner UUID', type: ParamType.uuid),
        ],
      ),
      ConsoleCommand(
        name: 'show part id',
        group: CommandGroup.objects,
        description: 'Show part by ID',
        syntax: 'show part id [uuid]',
        implemented: true,
        apiEndpoint: '/console/parts/show',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'search_by', description: 'Search type', type: ParamType.string, defaultValue: 'id'),
          CommandParam(name: 'uuid', description: 'Part UUID', type: ParamType.uuid),
        ],
      ),
      ConsoleCommand(
        name: 'show part name',
        group: CommandGroup.objects,
        description: 'Show part by name',
        syntax: 'show part name [name]',
        implemented: true,
        apiEndpoint: '/console/parts/show',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'search_by', description: 'Search type', type: ParamType.string, defaultValue: 'name'),
          CommandParam(name: 'name', description: 'Part name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'show part pos',
        group: CommandGroup.objects,
        description: 'Show part by position',
        syntax: 'show part pos [x] [y] [z]',
        implemented: true,
        apiEndpoint: '/console/parts/show',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'search_by', description: 'Search type', type: ParamType.string, defaultValue: 'pos'),
          CommandParam(name: 'x', description: 'X coordinate', type: ParamType.number),
          CommandParam(name: 'y', description: 'Y coordinate', type: ParamType.number),
          CommandParam(name: 'z', description: 'Z coordinate', type: ParamType.number),
        ],
      ),
      ConsoleCommand(
        name: 'dump object id',
        group: CommandGroup.objects,
        description: 'Dump object to XML',
        syntax: 'dump object id [uuid]',
        implemented: true,
        apiEndpoint: '/console/objects/dump',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'uuid', description: 'Object UUID', type: ParamType.uuid),
        ],
      ),
      ConsoleCommand(
        name: 'edit scale',
        group: CommandGroup.objects,
        description: 'Edit prim scale',
        syntax: 'edit scale [uuid] [x] [y] [z]',
        implemented: true,
        apiEndpoint: '/console/objects/edit-scale',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'uuid', description: 'Prim UUID', type: ParamType.uuid),
          CommandParam(name: 'x', description: 'X scale', type: ParamType.number),
          CommandParam(name: 'y', description: 'Y scale', type: ParamType.number),
          CommandParam(name: 'z', description: 'Z scale', type: ParamType.number),
        ],
      ),
      ConsoleCommand(
        name: 'rotate scene',
        group: CommandGroup.objects,
        description: 'Rotate all scene objects',
        syntax: 'rotate scene [degrees]',
        implemented: true,
        apiEndpoint: '/console/scene/rotate',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'degrees', description: 'Rotation in degrees', type: ParamType.number),
        ],
      ),
      ConsoleCommand(
        name: 'scale scene',
        group: CommandGroup.objects,
        description: 'Scale all scene objects',
        syntax: 'scale scene [factor]',
        implemented: true,
        apiEndpoint: '/console/scene/scale',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'factor', description: 'Scale factor', type: ParamType.number),
        ],
      ),
      ConsoleCommand(
        name: 'translate scene',
        group: CommandGroup.objects,
        description: 'Translate all scene objects',
        syntax: 'translate scene [x] [y] [z]',
        implemented: true,
        apiEndpoint: '/console/scene/translate',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'x', description: 'X offset', type: ParamType.number),
          CommandParam(name: 'y', description: 'Y offset', type: ParamType.number),
          CommandParam(name: 'z', description: 'Z offset', type: ParamType.number),
        ],
      ),

      // ===== ESTATES (4 commands) =====
      ConsoleCommand(
        name: 'estate create',
        group: CommandGroup.estates,
        description: 'Create new estate',
        syntax: 'estate create [name]',
        implemented: true,
        apiEndpoint: '/console/estates/create',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'name', description: 'Estate name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'estate set owner',
        group: CommandGroup.estates,
        description: 'Set estate owner',
        syntax: 'estate set owner [estate] [firstname] [lastname]',
        implemented: true,
        apiEndpoint: '/console/estates/set-owner',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'estate', description: 'Estate name', type: ParamType.string),
          CommandParam(name: 'firstname', description: 'Owner first name', type: ParamType.string),
          CommandParam(name: 'lastname', description: 'Owner last name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'estate set name',
        group: CommandGroup.estates,
        description: 'Set estate name',
        syntax: 'estate set name [estate] [newname]',
        implemented: true,
        apiEndpoint: '/console/estates/set-name',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'estate', description: 'Current estate name', type: ParamType.string),
          CommandParam(name: 'newname', description: 'New estate name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'estate link region',
        group: CommandGroup.estates,
        description: 'Link region to estate',
        syntax: 'estate link region [estate] [region]',
        implemented: true,
        apiEndpoint: '/console/estates/link-region',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'estate', description: 'Estate name', type: ParamType.string),
          CommandParam(name: 'region', description: 'Region name', type: ParamType.string),
        ],
      ),

      // ===== ARCHIVING (6 commands) =====
      ConsoleCommand(
        name: 'load iar',
        group: CommandGroup.archiving,
        description: 'Load inventory archive',
        syntax: 'load iar [firstname] [lastname] [path]',
        implemented: true,
        apiEndpoint: '/admin/archives/iar/load',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'firstname', description: 'User first name', type: ParamType.string),
          CommandParam(name: 'lastname', description: 'User last name', type: ParamType.string),
          CommandParam(name: 'file_path', description: 'IAR file path', type: ParamType.file),
          CommandParam(name: 'merge', description: 'Merge with existing', type: ParamType.boolean, required: false, defaultValue: 'false'),
        ],
      ),
      ConsoleCommand(
        name: 'save iar',
        group: CommandGroup.archiving,
        description: 'Save inventory archive',
        syntax: 'save iar [firstname] [lastname] [path]',
        implemented: true,
        apiEndpoint: '/admin/archives/iar/save',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'firstname', description: 'User first name', type: ParamType.string),
          CommandParam(name: 'lastname', description: 'User last name', type: ParamType.string),
          CommandParam(name: 'file_path', description: 'Output file path', type: ParamType.path),
          CommandParam(name: 'include_assets', description: 'Include assets', type: ParamType.boolean, required: false, defaultValue: 'true'),
        ],
      ),
      ConsoleCommand(
        name: 'load oar',
        group: CommandGroup.archiving,
        description: 'Load region archive',
        syntax: 'load oar [path]',
        implemented: true,
        apiEndpoint: '/admin/archives/oar/load',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'region_name', description: 'Target region', type: ParamType.string),
          CommandParam(name: 'file_path', description: 'OAR file path', type: ParamType.file),
          CommandParam(name: 'merge', description: 'Merge with existing', type: ParamType.boolean, required: false, defaultValue: 'false'),
        ],
      ),
      ConsoleCommand(
        name: 'save oar',
        group: CommandGroup.archiving,
        description: 'Save region archive',
        syntax: 'save oar [path]',
        implemented: true,
        apiEndpoint: '/admin/archives/oar/save',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'region_name', description: 'Source region', type: ParamType.string),
          CommandParam(name: 'file_path', description: 'Output file path', type: ParamType.path),
          CommandParam(name: 'include_assets', description: 'Include assets', type: ParamType.boolean, required: false, defaultValue: 'true'),
        ],
      ),
      ConsoleCommand(
        name: 'load xml',
        group: CommandGroup.archiving,
        description: 'Load objects from XML',
        syntax: 'load xml [path]',
        implemented: true,
        apiEndpoint: '/console/xml/load',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'file_path', description: 'XML file path', type: ParamType.file),
        ],
      ),
      ConsoleCommand(
        name: 'save xml',
        group: CommandGroup.archiving,
        description: 'Save objects to XML',
        syntax: 'save xml [path]',
        implemented: true,
        apiEndpoint: '/console/xml/save',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'file_path', description: 'Output XML file path', type: ParamType.path),
        ],
      ),

      // ===== ASSETS (10 commands) =====
      ConsoleCommand(
        name: 'show asset',
        group: CommandGroup.assets,
        description: 'Display asset info',
        syntax: 'show asset [uuid]',
        implemented: true,
        apiEndpoint: '/console/assets/show',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'uuid', description: 'Asset UUID', type: ParamType.uuid),
        ],
      ),
      ConsoleCommand(
        name: 'dump asset',
        group: CommandGroup.assets,
        description: 'Dump asset to file',
        syntax: 'dump asset [uuid] [path]',
        implemented: true,
        apiEndpoint: '/console/assets/dump',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'uuid', description: 'Asset UUID', type: ParamType.uuid),
          CommandParam(name: 'file_path', description: 'Output file path', type: ParamType.path),
        ],
      ),
      ConsoleCommand(
        name: 'delete asset',
        group: CommandGroup.assets,
        description: 'Delete asset',
        syntax: 'delete asset [uuid]',
        implemented: true,
        apiEndpoint: '/console/assets/delete',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'uuid', description: 'Asset UUID', type: ParamType.uuid),
        ],
      ),
      ConsoleCommand(
        name: 'fcache status',
        group: CommandGroup.assets,
        description: 'Show cache status',
        syntax: 'fcache status',
        implemented: true,
        apiEndpoint: '/console/fcache/status',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'fcache clear',
        group: CommandGroup.assets,
        description: 'Clear asset cache',
        syntax: 'fcache clear [file|memory]',
        implemented: true,
        apiEndpoint: '/console/fcache/clear',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'target', description: 'What to clear', type: ParamType.choice, required: false, choices: ['file', 'memory', 'all']),
        ],
      ),
      ConsoleCommand(
        name: 'fcache assets',
        group: CommandGroup.assets,
        description: 'List cached assets',
        syntax: 'fcache assets',
        implemented: true,
        apiEndpoint: '/console/fcache/assets',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'fcache expire',
        group: CommandGroup.assets,
        description: 'Expire old cache entries',
        syntax: 'fcache expire [days]',
        implemented: true,
        apiEndpoint: '/console/fcache/expire',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'days', description: 'Days to expire', type: ParamType.integer, defaultValue: '7'),
        ],
      ),
      ConsoleCommand(
        name: 'fcache clear negatives',
        group: CommandGroup.assets,
        description: 'Clear negative cache entries',
        syntax: 'fcache clear negatives',
        implemented: true,
        apiEndpoint: '/console/fcache/clearnegatives',
        httpMethod: 'POST',
        params: [],
      ),
      ConsoleCommand(
        name: 'fcache cache default assets',
        group: CommandGroup.assets,
        description: 'Cache default assets',
        syntax: 'fcache cache default assets',
        implemented: true,
        apiEndpoint: '/console/fcache/cachedefaultassets',
        httpMethod: 'POST',
        params: [],
      ),
      ConsoleCommand(
        name: 'fcache delete default assets',
        group: CommandGroup.assets,
        description: 'Delete default assets from cache',
        syntax: 'fcache delete default assets',
        implemented: true,
        apiEndpoint: '/console/fcache/deletedefaultassets',
        httpMethod: 'POST',
        params: [],
      ),

      // ===== COMMS (4 commands) =====
      ConsoleCommand(
        name: 'show circuits',
        group: CommandGroup.comms,
        description: 'Show active circuits',
        syntax: 'show circuits',
        implemented: true,
        apiEndpoint: '/console/circuits',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'show pending-objects',
        group: CommandGroup.comms,
        description: 'Show pending object updates',
        syntax: 'show pending-objects',
        implemented: true,
        apiEndpoint: '/console/comms/pending-objects',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'force update',
        group: CommandGroup.comms,
        description: 'Force update all clients',
        syntax: 'force update',
        implemented: true,
        apiEndpoint: '/console/scene/force-update',
        httpMethod: 'POST',
        params: [],
      ),

      // ===== HYPERGRID (4 commands) =====
      ConsoleCommand(
        name: 'link-region',
        group: CommandGroup.hypergrid,
        description: 'Link to external region',
        syntax: 'link-region [grid-uri] [region-name]',
        implemented: true,
        apiEndpoint: '/console/hypergrid/link',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'grid_uri', description: 'Grid URI', type: ParamType.string),
          CommandParam(name: 'region_name', description: 'Region name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'unlink-region',
        group: CommandGroup.hypergrid,
        description: 'Unlink external region',
        syntax: 'unlink-region [region-name]',
        implemented: true,
        apiEndpoint: '/console/hypergrid/unlink',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'region_name', description: 'Region name', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'show hyperlinks',
        group: CommandGroup.hypergrid,
        description: 'Show hypergrid links',
        syntax: 'show hyperlinks',
        implemented: true,
        apiEndpoint: '/console/hypergrid/links',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'link-mapping',
        group: CommandGroup.hypergrid,
        description: 'Set link mapping',
        syntax: 'link-mapping [x] [y]',
        implemented: true,
        apiEndpoint: '/console/hypergrid/mapping',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'x', description: 'X coordinate', type: ParamType.integer),
          CommandParam(name: 'y', description: 'Y coordinate', type: ParamType.integer),
        ],
      ),

      // ===== DATABASE (3 commands) =====
      ConsoleCommand(
        name: 'database stats',
        group: CommandGroup.database,
        description: 'Show database statistics',
        syntax: 'database stats',
        implemented: true,
        apiEndpoint: '/admin/database/stats',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'database health',
        group: CommandGroup.database,
        description: 'Check database health',
        syntax: 'database health',
        implemented: true,
        apiEndpoint: '/admin/database/health',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'database backup',
        group: CommandGroup.database,
        description: 'Create database backup',
        syntax: 'database backup [name]',
        implemented: true,
        apiEndpoint: '/admin/database/backup',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'backup_name', description: 'Backup name', type: ParamType.string),
          CommandParam(name: 'include_users', description: 'Include users', type: ParamType.boolean, required: false, defaultValue: 'true'),
          CommandParam(name: 'include_regions', description: 'Include regions', type: ParamType.boolean, required: false, defaultValue: 'true'),
        ],
      ),

      // ===== GENERAL (9 commands) =====
      ConsoleCommand(
        name: 'show info',
        group: CommandGroup.general,
        description: 'Show server information',
        syntax: 'show info',
        implemented: true,
        apiEndpoint: '/console/info',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'show version',
        group: CommandGroup.general,
        description: 'Show server version',
        syntax: 'show version',
        implemented: true,
        apiEndpoint: '/console/info',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'shutdown',
        group: CommandGroup.general,
        description: 'Shutdown server',
        syntax: 'shutdown',
        implemented: true,
        apiEndpoint: '/console/shutdown',
        httpMethod: 'POST',
        params: [],
      ),
      ConsoleCommand(
        name: 'quit',
        group: CommandGroup.general,
        description: 'Quit server immediately',
        syntax: 'quit',
        implemented: true,
        apiEndpoint: '/console/general/quit',
        httpMethod: 'POST',
        params: [],
      ),
      ConsoleCommand(
        name: 'show modules',
        group: CommandGroup.general,
        description: 'Show loaded modules',
        syntax: 'show modules',
        implemented: true,
        apiEndpoint: '/console/general/modules',
        httpMethod: 'GET',
        params: [],
      ),
      ConsoleCommand(
        name: 'command-script',
        group: CommandGroup.general,
        description: 'Run command script',
        syntax: 'command-script [path]',
        implemented: true,
        apiEndpoint: '/console/general/command-script',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'file_path', description: 'Script file path', type: ParamType.file),
        ],
      ),
      ConsoleCommand(
        name: 'config show',
        group: CommandGroup.general,
        description: 'Show configuration section',
        syntax: 'config show [section]',
        implemented: true,
        apiEndpoint: '/console/config/show',
        httpMethod: 'GET',
        params: [
          CommandParam(name: 'section', description: 'Config section', type: ParamType.string, required: false),
        ],
      ),
      ConsoleCommand(
        name: 'config get',
        group: CommandGroup.general,
        description: 'Get configuration value',
        syntax: 'config get [section] [key]',
        implemented: true,
        apiEndpoint: '/console/config/get',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'section', description: 'Config section', type: ParamType.string),
          CommandParam(name: 'key', description: 'Config key', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'config set',
        group: CommandGroup.general,
        description: 'Set configuration value',
        syntax: 'config set [section] [key] [value]',
        implemented: true,
        apiEndpoint: '/console/config/set',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'section', description: 'Config section', type: ParamType.string),
          CommandParam(name: 'key', description: 'Config key', type: ParamType.string),
          CommandParam(name: 'value', description: 'New value', type: ParamType.string),
        ],
      ),
      ConsoleCommand(
        name: 'set log level',
        group: CommandGroup.general,
        description: 'Set logging level',
        syntax: 'set log level [level]',
        implemented: true,
        apiEndpoint: '/console/log/level',
        httpMethod: 'POST',
        params: [
          CommandParam(name: 'level', description: 'Log level', type: ParamType.choice, choices: ['trace', 'debug', 'info', 'warn', 'error']),
        ],
      ),
      ConsoleCommand(
        name: 'force gc',
        group: CommandGroup.general,
        description: 'Force garbage collection',
        syntax: 'force gc',
        implemented: true,
        apiEndpoint: '/console/general/force-gc',
        httpMethod: 'POST',
        params: [],
      ),
    ];
  }

  /// Execute a console command
  Future<CommandResult> executeCommand(ConsoleCommand command, Map<String, dynamic> params) async {
    if (!command.implemented || command.apiEndpoint == null) {
      return CommandResult(
        success: false,
        message: 'Command not implemented yet',
        error: '${command.name} is not yet available',
      );
    }

    try {
      String endpoint = command.apiEndpoint!;

      // Handle path parameters (e.g., /console/regions/:name)
      if (endpoint.contains(':')) {
        final pathParams = RegExp(r':(\w+)').allMatches(endpoint);
        for (final match in pathParams) {
          final paramName = match.group(1)!;
          if (params.containsKey(paramName)) {
            endpoint = endpoint.replaceFirst(':$paramName', params[paramName].toString());
            params.remove(paramName);
          }
        }
      }

      final uri = Uri.parse('$baseUrl$endpoint');
      http.Response response;

      switch (command.httpMethod) {
        case 'GET':
          final queryUri = uri.replace(queryParameters: params.map((k, v) => MapEntry(k, v.toString())));
          response = await http.get(queryUri, headers: _headers);
          break;
        case 'POST':
          response = await http.post(uri, headers: _headers, body: jsonEncode(params));
          break;
        case 'PUT':
          response = await http.put(uri, headers: _headers, body: jsonEncode(params));
          break;
        case 'DELETE':
          response = await http.delete(uri, headers: _headers, body: jsonEncode(params));
          break;
        default:
          return CommandResult(
            success: false,
            message: 'Unknown HTTP method',
            error: 'Method ${command.httpMethod} not supported',
          );
      }

      if (response.statusCode >= 200 && response.statusCode < 300) {
        final data = jsonDecode(response.body);
        return CommandResult(
          success: data['success'] ?? true,
          message: data['message'] ?? 'Command executed successfully',
          data: data['data'],
        );
      } else {
        return CommandResult(
          success: false,
          message: 'Command failed',
          error: 'HTTP ${response.statusCode}: ${response.body}',
        );
      }
    } catch (e) {
      return CommandResult(
        success: false,
        message: 'Command execution error',
        error: e.toString(),
      );
    }
  }
}
