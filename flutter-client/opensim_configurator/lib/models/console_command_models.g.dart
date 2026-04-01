// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'console_command_models.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

CommandParam _$CommandParamFromJson(Map<String, dynamic> json) => CommandParam(
      name: json['name'] as String,
      description: json['description'] as String,
      type: $enumDecode(_$ParamTypeEnumMap, json['type']),
      required: json['required'] as bool? ?? true,
      defaultValue: json['defaultValue'] as String?,
      choices:
          (json['choices'] as List<dynamic>?)?.map((e) => e as String).toList(),
      placeholder: json['placeholder'] as String?,
    );

Map<String, dynamic> _$CommandParamToJson(CommandParam instance) =>
    <String, dynamic>{
      'name': instance.name,
      'description': instance.description,
      'type': _$ParamTypeEnumMap[instance.type]!,
      'required': instance.required,
      'defaultValue': instance.defaultValue,
      'choices': instance.choices,
      'placeholder': instance.placeholder,
    };

const _$ParamTypeEnumMap = {
  ParamType.string: 'string',
  ParamType.number: 'number',
  ParamType.integer: 'integer',
  ParamType.boolean: 'boolean',
  ParamType.uuid: 'uuid',
  ParamType.file: 'file',
  ParamType.path: 'path',
  ParamType.choice: 'choice',
};

ConsoleCommand _$ConsoleCommandFromJson(Map<String, dynamic> json) =>
    ConsoleCommand(
      name: json['name'] as String,
      group: $enumDecode(_$CommandGroupEnumMap, json['group']),
      description: json['description'] as String,
      syntax: json['syntax'] as String,
      params: (json['params'] as List<dynamic>?)
              ?.map((e) => CommandParam.fromJson(e as Map<String, dynamic>))
              .toList() ??
          const [],
      implemented: json['implemented'] as bool? ?? false,
      apiEndpoint: json['apiEndpoint'] as String?,
      httpMethod: json['httpMethod'] as String?,
    );

Map<String, dynamic> _$ConsoleCommandToJson(ConsoleCommand instance) =>
    <String, dynamic>{
      'name': instance.name,
      'group': _$CommandGroupEnumMap[instance.group]!,
      'description': instance.description,
      'syntax': instance.syntax,
      'params': instance.params.map((e) => e.toJson()).toList(),
      'implemented': instance.implemented,
      'apiEndpoint': instance.apiEndpoint,
      'httpMethod': instance.httpMethod,
    };

const _$CommandGroupEnumMap = {
  CommandGroup.users: 'users',
  CommandGroup.regions: 'regions',
  CommandGroup.terrain: 'terrain',
  CommandGroup.objects: 'objects',
  CommandGroup.estates: 'estates',
  CommandGroup.archiving: 'archiving',
  CommandGroup.assets: 'assets',
  CommandGroup.comms: 'comms',
  CommandGroup.hypergrid: 'hypergrid',
  CommandGroup.general: 'general',
  CommandGroup.database: 'database',
};

CommandResult _$CommandResultFromJson(Map<String, dynamic> json) =>
    CommandResult(
      success: json['success'] as bool,
      message: json['message'] as String,
      data: json['data'],
      error: json['error'] as String?,
      timestamp: json['timestamp'] == null
          ? null
          : DateTime.parse(json['timestamp'] as String),
    );

Map<String, dynamic> _$CommandResultToJson(CommandResult instance) =>
    <String, dynamic>{
      'success': instance.success,
      'message': instance.message,
      'data': instance.data,
      'error': instance.error,
      'timestamp': instance.timestamp.toIso8601String(),
    };

CommandExecution _$CommandExecutionFromJson(Map<String, dynamic> json) =>
    CommandExecution(
      command: json['command'] as String,
      params: Map<String, dynamic>.from(json['params'] as Map),
      result: json['result'] == null
          ? null
          : CommandResult.fromJson(json['result'] as Map<String, dynamic>),
      executedAt: json['executedAt'] == null
          ? null
          : DateTime.parse(json['executedAt'] as String),
    );

Map<String, dynamic> _$CommandExecutionToJson(CommandExecution instance) =>
    <String, dynamic>{
      'command': instance.command,
      'params': instance.params,
      'result': instance.result?.toJson(),
      'executedAt': instance.executedAt.toIso8601String(),
    };

T $enumDecode<T>(Map<T, dynamic> enumMap, Object? source) {
  for (final entry in enumMap.entries) {
    if (entry.value == source) {
      return entry.key;
    }
  }
  throw ArgumentError('Unknown enum value: $source');
}
