import 'package:json_annotation/json_annotation.dart';

part 'instance_models.g.dart';

enum InstanceStatus {
  @JsonValue('discovered')
  discovered,
  @JsonValue('starting')
  starting,
  @JsonValue('running')
  running,
  @JsonValue('stopping')
  stopping,
  @JsonValue('stopped')
  stopped,
  @JsonValue('error')
  error,
  @JsonValue('maintenance')
  maintenance,
  @JsonValue('unknown')
  unknown,
  @JsonValue('disconnected')
  disconnected,
}

enum InstanceEnvironment {
  @JsonValue('development')
  development,
  @JsonValue('staging')
  staging,
  @JsonValue('production')
  production,
}

enum HealthState {
  @JsonValue('healthy')
  healthy,
  @JsonValue('degraded')
  degraded,
  @JsonValue('unhealthy')
  unhealthy,
  @JsonValue('unknown')
  unknown,
}

enum InstanceCommand {
  start,
  stop,
  restart,
  shutdown,
  forceShutdown,
  reload,
  backup,
  getStatus,
  getLogs,
  getMetrics,
}

enum ConsoleOutputType {
  stdout,
  stderr,
  info,
  warning,
  error,
  debug,
  command,
}

enum SubscriptionChannel {
  statusUpdates,
  metrics,
  logs,
  console,
  alerts,
  userActivity,
  regionActivity,
}

@JsonSerializable()
class InstanceMetrics {
  final double cpuUsage;
  final int memoryUsageMb;
  final int memoryTotalMb;
  final int activeUsers;
  final int activeRegions;
  final int networkTxBytes;
  final int networkRxBytes;
  final int dbConnections;
  final int websocketConnections;
  final double requestRatePerSec;
  final double errorRatePerSec;
  final int uptimeSeconds;

  InstanceMetrics({
    required this.cpuUsage,
    required this.memoryUsageMb,
    required this.memoryTotalMb,
    required this.activeUsers,
    required this.activeRegions,
    required this.networkTxBytes,
    required this.networkRxBytes,
    required this.dbConnections,
    required this.websocketConnections,
    required this.requestRatePerSec,
    required this.errorRatePerSec,
    required this.uptimeSeconds,
  });

  double get memoryUsagePercent =>
      memoryTotalMb > 0 ? (memoryUsageMb / memoryTotalMb) * 100 : 0;

  String get uptimeFormatted {
    final hours = uptimeSeconds ~/ 3600;
    final minutes = (uptimeSeconds % 3600) ~/ 60;
    final seconds = uptimeSeconds % 60;
    if (hours > 0) {
      return '${hours}h ${minutes}m ${seconds}s';
    } else if (minutes > 0) {
      return '${minutes}m ${seconds}s';
    }
    return '${seconds}s';
  }

  factory InstanceMetrics.fromJson(Map<String, dynamic> json) =>
      _$InstanceMetricsFromJson(json);
  Map<String, dynamic> toJson() => _$InstanceMetricsToJson(this);
}

@JsonSerializable()
class ComponentHealth {
  final String name;
  final HealthState status;
  final String? message;
  final int? responseTimeMs;

  ComponentHealth({
    required this.name,
    required this.status,
    this.message,
    this.responseTimeMs,
  });

  factory ComponentHealth.fromJson(Map<String, dynamic> json) =>
      _$ComponentHealthFromJson(json);
  Map<String, dynamic> toJson() => _$ComponentHealthToJson(this);
}

@JsonSerializable()
class HealthStatus {
  final HealthState overall;
  final Map<String, ComponentHealth> components;
  final DateTime lastCheck;
  final int responseTimeMs;

  HealthStatus({
    required this.overall,
    required this.components,
    required this.lastCheck,
    required this.responseTimeMs,
  });

  factory HealthStatus.fromJson(Map<String, dynamic> json) =>
      _$HealthStatusFromJson(json);
  Map<String, dynamic> toJson() => _$HealthStatusToJson(this);
}

@JsonSerializable()
class InstancePorts {
  final int websocket;
  final int admin;
  final int metrics;
  final int http;
  final int udp;

  InstancePorts({
    this.websocket = 9001,
    this.admin = 9200,
    this.metrics = 9100,
    this.http = 9000,
    this.udp = 9000,
  });

  factory InstancePorts.fromJson(Map<String, dynamic> json) =>
      _$InstancePortsFromJson(json);
  Map<String, dynamic> toJson() => _$InstancePortsToJson(this);
}

@JsonSerializable()
class TlsConfig {
  final bool enabled;
  final String? caCertPath;
  final bool verifyHost;

  TlsConfig({
    this.enabled = false,
    this.caCertPath,
    this.verifyHost = true,
  });

  factory TlsConfig.fromJson(Map<String, dynamic> json) =>
      _$TlsConfigFromJson(json);
  Map<String, dynamic> toJson() => _$TlsConfigToJson(this);
}

@JsonSerializable()
class ServerInstance {
  final String id;
  final String name;
  final String description;
  final String host;
  final InstancePorts ports;
  final String apiKey;
  final InstanceEnvironment environment;
  final bool autoConnect;
  final List<String> tags;
  final TlsConfig tls;

  InstanceStatus status;
  InstanceMetrics? metrics;
  HealthStatus? health;
  String? version;
  DateTime lastSeen;
  bool connected;

  ServerInstance({
    required this.id,
    required this.name,
    this.description = '',
    required this.host,
    required this.ports,
    required this.apiKey,
    this.environment = InstanceEnvironment.development,
    this.autoConnect = true,
    this.tags = const [],
    TlsConfig? tls,
    this.status = InstanceStatus.unknown,
    this.metrics,
    this.health,
    this.version,
    DateTime? lastSeen,
    this.connected = false,
  })  : tls = tls ?? TlsConfig(),
        lastSeen = lastSeen ?? DateTime.now();

  String get websocketUrl {
    final protocol = tls.enabled ? 'wss' : 'ws';
    return '$protocol://$host:${ports.websocket}';
  }

  String get adminUrl {
    final protocol = tls.enabled ? 'https' : 'http';
    return '$protocol://$host:${ports.admin}';
  }

  String get metricsUrl {
    final protocol = tls.enabled ? 'https' : 'http';
    return '$protocol://$host:${ports.metrics}';
  }

  String get httpUrl {
    final protocol = tls.enabled ? 'https' : 'http';
    return '$protocol://$host:${ports.http}';
  }

  bool get isHealthy =>
      health?.overall == HealthState.healthy && status == InstanceStatus.running;

  bool get needsAttention =>
      health?.overall == HealthState.degraded ||
      health?.overall == HealthState.unhealthy ||
      status == InstanceStatus.error;

  factory ServerInstance.fromJson(Map<String, dynamic> json) =>
      _$ServerInstanceFromJson(json);
  Map<String, dynamic> toJson() => _$ServerInstanceToJson(this);
}

@JsonSerializable()
class ConsoleEntry {
  final String instanceId;
  final String content;
  final ConsoleOutputType outputType;
  final DateTime timestamp;

  ConsoleEntry({
    required this.instanceId,
    required this.content,
    required this.outputType,
    required this.timestamp,
  });

  factory ConsoleEntry.fromJson(Map<String, dynamic> json) =>
      _$ConsoleEntryFromJson(json);
  Map<String, dynamic> toJson() => _$ConsoleEntryToJson(this);
}

@JsonSerializable()
class CommandResult {
  final bool success;
  final String message;
  final Map<String, dynamic>? data;
  final int durationMs;

  CommandResult({
    required this.success,
    required this.message,
    this.data,
    this.durationMs = 0,
  });

  factory CommandResult.success(String message, {Map<String, dynamic>? data}) =>
      CommandResult(success: true, message: message, data: data);

  factory CommandResult.failure(String message) =>
      CommandResult(success: false, message: message);

  factory CommandResult.fromJson(Map<String, dynamic> json) =>
      _$CommandResultFromJson(json);
  Map<String, dynamic> toJson() => _$CommandResultToJson(this);
}

@JsonSerializable()
class BatchResult {
  final String instanceId;
  final String status;
  final String message;
  final Map<String, dynamic>? data;
  final int durationMs;

  BatchResult({
    required this.instanceId,
    required this.status,
    required this.message,
    this.data,
    required this.durationMs,
  });

  bool get isSuccess => status == 'success';

  factory BatchResult.fromJson(Map<String, dynamic> json) =>
      _$BatchResultFromJson(json);
  Map<String, dynamic> toJson() => _$BatchResultToJson(this);
}

@JsonSerializable()
class InstancesConfig {
  final ControllerConfig controller;
  final List<InstanceConfigEntry> instances;

  InstancesConfig({
    required this.controller,
    required this.instances,
  });

  factory InstancesConfig.fromJson(Map<String, dynamic> json) =>
      _$InstancesConfigFromJson(json);
  Map<String, dynamic> toJson() => _$InstancesConfigToJson(this);
}

@JsonSerializable()
class ControllerConfig {
  final String discoveryMode;
  final int healthCheckIntervalMs;
  final int heartbeatTimeoutMs;
  final int reconnectDelayMs;
  final int maxReconnectAttempts;
  final int commandTimeoutMs;

  ControllerConfig({
    this.discoveryMode = 'config',
    this.healthCheckIntervalMs = 5000,
    this.heartbeatTimeoutMs = 15000,
    this.reconnectDelayMs = 3000,
    this.maxReconnectAttempts = 5,
    this.commandTimeoutMs = 30000,
  });

  factory ControllerConfig.fromJson(Map<String, dynamic> json) =>
      _$ControllerConfigFromJson(json);
  Map<String, dynamic> toJson() => _$ControllerConfigToJson(this);
}

@JsonSerializable()
class InstanceConfigEntry {
  final String id;
  final String name;
  final String description;
  final String host;
  final int websocketPort;
  final int adminPort;
  final int metricsPort;
  final int httpPort;
  final int udpPort;
  final String apiKey;
  final InstanceEnvironment environment;
  final bool autoConnect;
  final List<String> tags;

  InstanceConfigEntry({
    required this.id,
    required this.name,
    this.description = '',
    required this.host,
    this.websocketPort = 9001,
    this.adminPort = 9200,
    this.metricsPort = 9100,
    this.httpPort = 9000,
    this.udpPort = 9000,
    required this.apiKey,
    this.environment = InstanceEnvironment.development,
    this.autoConnect = true,
    this.tags = const [],
  });

  ServerInstance toServerInstance() => ServerInstance(
        id: id,
        name: name,
        description: description,
        host: host,
        ports: InstancePorts(
          websocket: websocketPort,
          admin: adminPort,
          metrics: metricsPort,
          http: httpPort,
          udp: udpPort,
        ),
        apiKey: apiKey,
        environment: environment,
        autoConnect: autoConnect,
        tags: tags,
      );

  factory InstanceConfigEntry.fromJson(Map<String, dynamic> json) =>
      _$InstanceConfigEntryFromJson(json);
  Map<String, dynamic> toJson() => _$InstanceConfigEntryToJson(this);
}

class InstanceStatusInfo {
  final InstanceStatus status;
  final String label;
  final String description;
  final String icon;
  final bool canStart;
  final bool canStop;
  final bool canRestart;

  const InstanceStatusInfo({
    required this.status,
    required this.label,
    required this.description,
    required this.icon,
    required this.canStart,
    required this.canStop,
    required this.canRestart,
  });

  static InstanceStatusInfo getInfo(InstanceStatus status) {
    switch (status) {
      case InstanceStatus.starting:
        return const InstanceStatusInfo(
          status: InstanceStatus.starting,
          label: 'Starting',
          description: 'Instance is starting up',
          icon: '🔄',
          canStart: false,
          canStop: true,
          canRestart: false,
        );
      case InstanceStatus.running:
        return const InstanceStatusInfo(
          status: InstanceStatus.running,
          label: 'Running',
          description: 'Instance is operational',
          icon: '✅',
          canStart: false,
          canStop: true,
          canRestart: true,
        );
      case InstanceStatus.stopping:
        return const InstanceStatusInfo(
          status: InstanceStatus.stopping,
          label: 'Stopping',
          description: 'Instance is shutting down',
          icon: '⏳',
          canStart: false,
          canStop: false,
          canRestart: false,
        );
      case InstanceStatus.stopped:
        return const InstanceStatusInfo(
          status: InstanceStatus.stopped,
          label: 'Stopped',
          description: 'Instance is not running',
          icon: '⏹️',
          canStart: true,
          canStop: false,
          canRestart: false,
        );
      case InstanceStatus.error:
        return const InstanceStatusInfo(
          status: InstanceStatus.error,
          label: 'Error',
          description: 'Instance encountered an error',
          icon: '❌',
          canStart: true,
          canStop: true,
          canRestart: true,
        );
      case InstanceStatus.maintenance:
        return const InstanceStatusInfo(
          status: InstanceStatus.maintenance,
          label: 'Maintenance',
          description: 'Instance is in maintenance mode',
          icon: '🔧',
          canStart: false,
          canStop: true,
          canRestart: false,
        );
      case InstanceStatus.unknown:
        return const InstanceStatusInfo(
          status: InstanceStatus.unknown,
          label: 'Unknown',
          description: 'Instance status is unknown',
          icon: '❓',
          canStart: true,
          canStop: true,
          canRestart: true,
        );
      case InstanceStatus.disconnected:
        return const InstanceStatusInfo(
          status: InstanceStatus.disconnected,
          label: 'Disconnected',
          description: 'Lost connection to instance',
          icon: '🔌',
          canStart: true,
          canStop: false,
          canRestart: false,
        );
      case InstanceStatus.discovered:
        return const InstanceStatusInfo(
          status: InstanceStatus.discovered,
          label: 'Discovered',
          description: 'Instance directory found, not yet started',
          icon: '🔍',
          canStart: true,
          canStop: false,
          canRestart: false,
        );
    }
  }
}
