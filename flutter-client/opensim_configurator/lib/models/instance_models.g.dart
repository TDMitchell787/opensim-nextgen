// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'instance_models.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

InstanceMetrics _$InstanceMetricsFromJson(Map<String, dynamic> json) =>
    InstanceMetrics(
      cpuUsage: (json['cpuUsage'] as num).toDouble(),
      memoryUsageMb: (json['memoryUsageMb'] as num).toInt(),
      memoryTotalMb: (json['memoryTotalMb'] as num).toInt(),
      activeUsers: (json['activeUsers'] as num).toInt(),
      activeRegions: (json['activeRegions'] as num).toInt(),
      networkTxBytes: (json['networkTxBytes'] as num).toInt(),
      networkRxBytes: (json['networkRxBytes'] as num).toInt(),
      dbConnections: (json['dbConnections'] as num).toInt(),
      websocketConnections: (json['websocketConnections'] as num).toInt(),
      requestRatePerSec: (json['requestRatePerSec'] as num).toDouble(),
      errorRatePerSec: (json['errorRatePerSec'] as num).toDouble(),
      uptimeSeconds: (json['uptimeSeconds'] as num).toInt(),
    );

Map<String, dynamic> _$InstanceMetricsToJson(InstanceMetrics instance) =>
    <String, dynamic>{
      'cpuUsage': instance.cpuUsage,
      'memoryUsageMb': instance.memoryUsageMb,
      'memoryTotalMb': instance.memoryTotalMb,
      'activeUsers': instance.activeUsers,
      'activeRegions': instance.activeRegions,
      'networkTxBytes': instance.networkTxBytes,
      'networkRxBytes': instance.networkRxBytes,
      'dbConnections': instance.dbConnections,
      'websocketConnections': instance.websocketConnections,
      'requestRatePerSec': instance.requestRatePerSec,
      'errorRatePerSec': instance.errorRatePerSec,
      'uptimeSeconds': instance.uptimeSeconds,
    };

ComponentHealth _$ComponentHealthFromJson(Map<String, dynamic> json) =>
    ComponentHealth(
      name: json['name'] as String,
      status: $enumDecode(_$HealthStateEnumMap, json['status']),
      message: json['message'] as String?,
      responseTimeMs: (json['responseTimeMs'] as num?)?.toInt(),
    );

Map<String, dynamic> _$ComponentHealthToJson(ComponentHealth instance) =>
    <String, dynamic>{
      'name': instance.name,
      'status': _$HealthStateEnumMap[instance.status]!,
      'message': instance.message,
      'responseTimeMs': instance.responseTimeMs,
    };

const _$HealthStateEnumMap = {
  HealthState.healthy: 'healthy',
  HealthState.degraded: 'degraded',
  HealthState.unhealthy: 'unhealthy',
  HealthState.unknown: 'unknown',
};

HealthStatus _$HealthStatusFromJson(Map<String, dynamic> json) => HealthStatus(
  overall: $enumDecode(_$HealthStateEnumMap, json['overall']),
  components: (json['components'] as Map<String, dynamic>).map(
    (k, e) => MapEntry(k, ComponentHealth.fromJson(e as Map<String, dynamic>)),
  ),
  lastCheck: DateTime.parse(json['lastCheck'] as String),
  responseTimeMs: (json['responseTimeMs'] as num).toInt(),
);

Map<String, dynamic> _$HealthStatusToJson(HealthStatus instance) =>
    <String, dynamic>{
      'overall': _$HealthStateEnumMap[instance.overall]!,
      'components': instance.components,
      'lastCheck': instance.lastCheck.toIso8601String(),
      'responseTimeMs': instance.responseTimeMs,
    };

InstancePorts _$InstancePortsFromJson(Map<String, dynamic> json) =>
    InstancePorts(
      websocket: (json['websocket'] as num?)?.toInt() ?? 9001,
      admin: (json['admin'] as num?)?.toInt() ?? 9200,
      metrics: (json['metrics'] as num?)?.toInt() ?? 9100,
      http: (json['http'] as num?)?.toInt() ?? 9000,
      udp: (json['udp'] as num?)?.toInt() ?? 9000,
    );

Map<String, dynamic> _$InstancePortsToJson(InstancePorts instance) =>
    <String, dynamic>{
      'websocket': instance.websocket,
      'admin': instance.admin,
      'metrics': instance.metrics,
      'http': instance.http,
      'udp': instance.udp,
    };

TlsConfig _$TlsConfigFromJson(Map<String, dynamic> json) => TlsConfig(
  enabled: json['enabled'] as bool? ?? false,
  caCertPath: json['caCertPath'] as String?,
  verifyHost: json['verifyHost'] as bool? ?? true,
);

Map<String, dynamic> _$TlsConfigToJson(TlsConfig instance) => <String, dynamic>{
  'enabled': instance.enabled,
  'caCertPath': instance.caCertPath,
  'verifyHost': instance.verifyHost,
};

ServerInstance _$ServerInstanceFromJson(
  Map<String, dynamic> json,
) => ServerInstance(
  id: json['id'] as String,
  name: json['name'] as String,
  description: json['description'] as String? ?? '',
  host: json['host'] as String,
  ports: InstancePorts.fromJson(json['ports'] as Map<String, dynamic>),
  apiKey: json['apiKey'] as String,
  environment:
      $enumDecodeNullable(_$InstanceEnvironmentEnumMap, json['environment']) ??
      InstanceEnvironment.development,
  autoConnect: json['autoConnect'] as bool? ?? true,
  tags:
      (json['tags'] as List<dynamic>?)?.map((e) => e as String).toList() ??
      const [],
  tls:
      json['tls'] == null
          ? null
          : TlsConfig.fromJson(json['tls'] as Map<String, dynamic>),
  status:
      $enumDecodeNullable(_$InstanceStatusEnumMap, json['status']) ??
      InstanceStatus.unknown,
  metrics:
      json['metrics'] == null
          ? null
          : InstanceMetrics.fromJson(json['metrics'] as Map<String, dynamic>),
  health:
      json['health'] == null
          ? null
          : HealthStatus.fromJson(json['health'] as Map<String, dynamic>),
  version: json['version'] as String?,
  lastSeen:
      json['lastSeen'] == null
          ? null
          : DateTime.parse(json['lastSeen'] as String),
  connected: json['connected'] as bool? ?? false,
);

Map<String, dynamic> _$ServerInstanceToJson(ServerInstance instance) =>
    <String, dynamic>{
      'id': instance.id,
      'name': instance.name,
      'description': instance.description,
      'host': instance.host,
      'ports': instance.ports,
      'apiKey': instance.apiKey,
      'environment': _$InstanceEnvironmentEnumMap[instance.environment]!,
      'autoConnect': instance.autoConnect,
      'tags': instance.tags,
      'tls': instance.tls,
      'status': _$InstanceStatusEnumMap[instance.status]!,
      'metrics': instance.metrics,
      'health': instance.health,
      'version': instance.version,
      'lastSeen': instance.lastSeen.toIso8601String(),
      'connected': instance.connected,
    };

const _$InstanceEnvironmentEnumMap = {
  InstanceEnvironment.development: 'development',
  InstanceEnvironment.staging: 'staging',
  InstanceEnvironment.production: 'production',
};

const _$InstanceStatusEnumMap = {
  InstanceStatus.starting: 'starting',
  InstanceStatus.running: 'running',
  InstanceStatus.stopping: 'stopping',
  InstanceStatus.stopped: 'stopped',
  InstanceStatus.error: 'error',
  InstanceStatus.maintenance: 'maintenance',
  InstanceStatus.unknown: 'unknown',
  InstanceStatus.disconnected: 'disconnected',
};

ConsoleEntry _$ConsoleEntryFromJson(Map<String, dynamic> json) => ConsoleEntry(
  instanceId: json['instanceId'] as String,
  content: json['content'] as String,
  outputType: $enumDecode(_$ConsoleOutputTypeEnumMap, json['outputType']),
  timestamp: DateTime.parse(json['timestamp'] as String),
);

Map<String, dynamic> _$ConsoleEntryToJson(ConsoleEntry instance) =>
    <String, dynamic>{
      'instanceId': instance.instanceId,
      'content': instance.content,
      'outputType': _$ConsoleOutputTypeEnumMap[instance.outputType]!,
      'timestamp': instance.timestamp.toIso8601String(),
    };

const _$ConsoleOutputTypeEnumMap = {
  ConsoleOutputType.stdout: 'stdout',
  ConsoleOutputType.stderr: 'stderr',
  ConsoleOutputType.info: 'info',
  ConsoleOutputType.warning: 'warning',
  ConsoleOutputType.error: 'error',
  ConsoleOutputType.debug: 'debug',
  ConsoleOutputType.command: 'command',
};

CommandResult _$CommandResultFromJson(Map<String, dynamic> json) =>
    CommandResult(
      success: json['success'] as bool,
      message: json['message'] as String,
      data: json['data'] as Map<String, dynamic>?,
      durationMs: (json['durationMs'] as num?)?.toInt() ?? 0,
    );

Map<String, dynamic> _$CommandResultToJson(CommandResult instance) =>
    <String, dynamic>{
      'success': instance.success,
      'message': instance.message,
      'data': instance.data,
      'durationMs': instance.durationMs,
    };

BatchResult _$BatchResultFromJson(Map<String, dynamic> json) => BatchResult(
  instanceId: json['instanceId'] as String,
  status: json['status'] as String,
  message: json['message'] as String,
  data: json['data'] as Map<String, dynamic>?,
  durationMs: (json['durationMs'] as num).toInt(),
);

Map<String, dynamic> _$BatchResultToJson(BatchResult instance) =>
    <String, dynamic>{
      'instanceId': instance.instanceId,
      'status': instance.status,
      'message': instance.message,
      'data': instance.data,
      'durationMs': instance.durationMs,
    };

InstancesConfig _$InstancesConfigFromJson(Map<String, dynamic> json) =>
    InstancesConfig(
      controller: ControllerConfig.fromJson(
        json['controller'] as Map<String, dynamic>,
      ),
      instances:
          (json['instances'] as List<dynamic>)
              .map(
                (e) => InstanceConfigEntry.fromJson(e as Map<String, dynamic>),
              )
              .toList(),
    );

Map<String, dynamic> _$InstancesConfigToJson(InstancesConfig instance) =>
    <String, dynamic>{
      'controller': instance.controller,
      'instances': instance.instances,
    };

ControllerConfig _$ControllerConfigFromJson(
  Map<String, dynamic> json,
) => ControllerConfig(
  discoveryMode: json['discoveryMode'] as String? ?? 'config',
  healthCheckIntervalMs:
      (json['healthCheckIntervalMs'] as num?)?.toInt() ?? 5000,
  heartbeatTimeoutMs: (json['heartbeatTimeoutMs'] as num?)?.toInt() ?? 15000,
  reconnectDelayMs: (json['reconnectDelayMs'] as num?)?.toInt() ?? 3000,
  maxReconnectAttempts: (json['maxReconnectAttempts'] as num?)?.toInt() ?? 5,
  commandTimeoutMs: (json['commandTimeoutMs'] as num?)?.toInt() ?? 30000,
);

Map<String, dynamic> _$ControllerConfigToJson(ControllerConfig instance) =>
    <String, dynamic>{
      'discoveryMode': instance.discoveryMode,
      'healthCheckIntervalMs': instance.healthCheckIntervalMs,
      'heartbeatTimeoutMs': instance.heartbeatTimeoutMs,
      'reconnectDelayMs': instance.reconnectDelayMs,
      'maxReconnectAttempts': instance.maxReconnectAttempts,
      'commandTimeoutMs': instance.commandTimeoutMs,
    };

InstanceConfigEntry _$InstanceConfigEntryFromJson(Map<String, dynamic> json) =>
    InstanceConfigEntry(
      id: json['id'] as String,
      name: json['name'] as String,
      description: json['description'] as String? ?? '',
      host: json['host'] as String,
      websocketPort: (json['websocketPort'] as num?)?.toInt() ?? 9001,
      adminPort: (json['adminPort'] as num?)?.toInt() ?? 9200,
      metricsPort: (json['metricsPort'] as num?)?.toInt() ?? 9100,
      httpPort: (json['httpPort'] as num?)?.toInt() ?? 9000,
      udpPort: (json['udpPort'] as num?)?.toInt() ?? 9000,
      apiKey: json['apiKey'] as String,
      environment:
          $enumDecodeNullable(
            _$InstanceEnvironmentEnumMap,
            json['environment'],
          ) ??
          InstanceEnvironment.development,
      autoConnect: json['autoConnect'] as bool? ?? true,
      tags:
          (json['tags'] as List<dynamic>?)?.map((e) => e as String).toList() ??
          const [],
    );

Map<String, dynamic> _$InstanceConfigEntryToJson(
  InstanceConfigEntry instance,
) => <String, dynamic>{
  'id': instance.id,
  'name': instance.name,
  'description': instance.description,
  'host': instance.host,
  'websocketPort': instance.websocketPort,
  'adminPort': instance.adminPort,
  'metricsPort': instance.metricsPort,
  'httpPort': instance.httpPort,
  'udpPort': instance.udpPort,
  'apiKey': instance.apiKey,
  'environment': _$InstanceEnvironmentEnumMap[instance.environment]!,
  'autoConnect': instance.autoConnect,
  'tags': instance.tags,
};
