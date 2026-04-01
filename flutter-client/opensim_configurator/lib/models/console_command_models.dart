import 'package:json_annotation/json_annotation.dart';

part 'console_command_models.g.dart';

enum CommandGroup {
  @JsonValue('users')
  users,
  @JsonValue('regions')
  regions,
  @JsonValue('terrain')
  terrain,
  @JsonValue('objects')
  objects,
  @JsonValue('estates')
  estates,
  @JsonValue('archiving')
  archiving,
  @JsonValue('assets')
  assets,
  @JsonValue('comms')
  comms,
  @JsonValue('hypergrid')
  hypergrid,
  @JsonValue('general')
  general,
  @JsonValue('database')
  database,
}

enum ParamType {
  @JsonValue('string')
  string,
  @JsonValue('number')
  number,
  @JsonValue('integer')
  integer,
  @JsonValue('boolean')
  boolean,
  @JsonValue('uuid')
  uuid,
  @JsonValue('file')
  file,
  @JsonValue('path')
  path,
  @JsonValue('choice')
  choice,
}

@JsonSerializable()
class CommandParam {
  final String name;
  final String description;
  final ParamType type;
  final bool required;
  final String? defaultValue;
  final List<String>? choices;
  final String? placeholder;

  CommandParam({
    required this.name,
    required this.description,
    required this.type,
    this.required = true,
    this.defaultValue,
    this.choices,
    this.placeholder,
  });

  factory CommandParam.fromJson(Map<String, dynamic> json) =>
      _$CommandParamFromJson(json);
  Map<String, dynamic> toJson() => _$CommandParamToJson(this);
}

@JsonSerializable()
class ConsoleCommand {
  final String name;
  final CommandGroup group;
  final String description;
  final String syntax;
  final List<CommandParam> params;
  final bool implemented;
  final String? apiEndpoint;
  final String? httpMethod;

  ConsoleCommand({
    required this.name,
    required this.group,
    required this.description,
    required this.syntax,
    this.params = const [],
    this.implemented = false,
    this.apiEndpoint,
    this.httpMethod,
  });

  String get groupLabel => _groupLabels[group] ?? group.name;

  static const Map<CommandGroup, String> _groupLabels = {
    CommandGroup.users: 'Users',
    CommandGroup.regions: 'Regions',
    CommandGroup.terrain: 'Terrain',
    CommandGroup.objects: 'Objects',
    CommandGroup.estates: 'Estates',
    CommandGroup.archiving: 'Archiving',
    CommandGroup.assets: 'Assets',
    CommandGroup.comms: 'Comms',
    CommandGroup.hypergrid: 'Hypergrid',
    CommandGroup.general: 'General',
    CommandGroup.database: 'Database',
  };

  factory ConsoleCommand.fromJson(Map<String, dynamic> json) =>
      _$ConsoleCommandFromJson(json);
  Map<String, dynamic> toJson() => _$ConsoleCommandToJson(this);
}

@JsonSerializable()
class CommandResult {
  final bool success;
  final String message;
  final dynamic data;
  final String? error;
  final DateTime timestamp;

  CommandResult({
    required this.success,
    required this.message,
    this.data,
    this.error,
    DateTime? timestamp,
  }) : timestamp = timestamp ?? DateTime.now();

  factory CommandResult.fromJson(Map<String, dynamic> json) =>
      _$CommandResultFromJson(json);
  Map<String, dynamic> toJson() => _$CommandResultToJson(this);
}

@JsonSerializable()
class CommandExecution {
  final String command;
  final Map<String, dynamic> params;
  final CommandResult? result;
  final DateTime executedAt;

  CommandExecution({
    required this.command,
    required this.params,
    this.result,
    DateTime? executedAt,
  }) : executedAt = executedAt ?? DateTime.now();

  factory CommandExecution.fromJson(Map<String, dynamic> json) =>
      _$CommandExecutionFromJson(json);
  Map<String, dynamic> toJson() => _$CommandExecutionToJson(this);
}
