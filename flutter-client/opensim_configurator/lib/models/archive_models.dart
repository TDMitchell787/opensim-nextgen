import 'package:json_annotation/json_annotation.dart';

part 'archive_models.g.dart';

enum ArchiveType {
  @JsonValue('iar')
  iar,
  @JsonValue('oar')
  oar,
}

enum JobStatus {
  @JsonValue('queued')
  queued,
  @JsonValue('running')
  running,
  @JsonValue('completed')
  completed,
  @JsonValue('failed')
  failed,
  @JsonValue('cancelled')
  cancelled,
}

enum JobOperation {
  @JsonValue('load')
  load,
  @JsonValue('save')
  save,
}

@JsonSerializable()
class ArchiveJob {
  final String id;
  final ArchiveType archiveType;
  final JobOperation operation;
  final JobStatus status;
  final double progress;
  final String? progressMessage;
  final String? targetId;
  final String? targetName;
  final String? filePath;
  final DateTime createdAt;
  final DateTime? startedAt;
  final DateTime? completedAt;
  final String? error;
  final ArchiveJobResult? result;

  ArchiveJob({
    required this.id,
    required this.archiveType,
    required this.operation,
    required this.status,
    this.progress = 0.0,
    this.progressMessage,
    this.targetId,
    this.targetName,
    this.filePath,
    required this.createdAt,
    this.startedAt,
    this.completedAt,
    this.error,
    this.result,
  });

  String get typeLabel => archiveType == ArchiveType.iar ? 'IAR' : 'OAR';
  String get operationLabel => operation == JobOperation.load ? 'Load' : 'Save';
  String get displayName => '$typeLabel $operationLabel';

  bool get isActive => status == JobStatus.queued || status == JobStatus.running;
  bool get isComplete => status == JobStatus.completed;
  bool get isFailed => status == JobStatus.failed;
  bool get isCancelled => status == JobStatus.cancelled;

  Duration? get elapsed {
    if (startedAt == null) return null;
    final end = completedAt ?? DateTime.now();
    return end.difference(startedAt!);
  }

  String get elapsedFormatted {
    final d = elapsed;
    if (d == null) return '--';
    if (d.inHours > 0) {
      return '${d.inHours}h ${d.inMinutes.remainder(60)}m';
    }
    if (d.inMinutes > 0) {
      return '${d.inMinutes}m ${d.inSeconds.remainder(60)}s';
    }
    return '${d.inSeconds}s';
  }

  factory ArchiveJob.fromJson(Map<String, dynamic> json) =>
      _$ArchiveJobFromJson(json);
  Map<String, dynamic> toJson() => _$ArchiveJobToJson(this);
}

@JsonSerializable()
class ArchiveJobResult {
  final int? assetsProcessed;
  final int? foldersProcessed;
  final int? itemsProcessed;
  final int? objectsProcessed;
  final int? parcelsProcessed;
  final bool? terrainProcessed;
  final int? archiveSizeBytes;
  final String? downloadPath;

  ArchiveJobResult({
    this.assetsProcessed,
    this.foldersProcessed,
    this.itemsProcessed,
    this.objectsProcessed,
    this.parcelsProcessed,
    this.terrainProcessed,
    this.archiveSizeBytes,
    this.downloadPath,
  });

  String get archiveSizeFormatted {
    if (archiveSizeBytes == null) return '--';
    final bytes = archiveSizeBytes!;
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(2)} GB';
  }

  factory ArchiveJobResult.fromJson(Map<String, dynamic> json) =>
      _$ArchiveJobResultFromJson(json);
  Map<String, dynamic> toJson() => _$ArchiveJobResultToJson(this);
}

@JsonSerializable()
class IarLoadRequest {
  final String userId;
  final String? targetFolderId;
  final bool merge;
  final String filePath;

  IarLoadRequest({
    required this.userId,
    this.targetFolderId,
    this.merge = false,
    required this.filePath,
  });

  factory IarLoadRequest.fromJson(Map<String, dynamic> json) =>
      _$IarLoadRequestFromJson(json);
  Map<String, dynamic> toJson() => _$IarLoadRequestToJson(this);
}

@JsonSerializable()
class IarSaveRequest {
  final String userId;
  final String? folderId;
  final bool includeAssets;

  IarSaveRequest({
    required this.userId,
    this.folderId,
    this.includeAssets = true,
  });

  factory IarSaveRequest.fromJson(Map<String, dynamic> json) =>
      _$IarSaveRequestFromJson(json);
  Map<String, dynamic> toJson() => _$IarSaveRequestToJson(this);
}

@JsonSerializable()
class OarLoadRequest {
  final String regionId;
  final String filePath;
  final bool merge;
  final bool loadTerrain;
  final bool loadObjects;
  final bool loadParcels;

  OarLoadRequest({
    required this.regionId,
    required this.filePath,
    this.merge = false,
    this.loadTerrain = true,
    this.loadObjects = true,
    this.loadParcels = true,
  });

  factory OarLoadRequest.fromJson(Map<String, dynamic> json) =>
      _$OarLoadRequestFromJson(json);
  Map<String, dynamic> toJson() => _$OarLoadRequestToJson(this);
}

@JsonSerializable()
class OarSaveRequest {
  final String regionId;
  final bool includeAssets;
  final bool includeTerrain;
  final bool includeObjects;
  final bool includeParcels;

  OarSaveRequest({
    required this.regionId,
    this.includeAssets = true,
    this.includeTerrain = true,
    this.includeObjects = true,
    this.includeParcels = true,
  });

  factory OarSaveRequest.fromJson(Map<String, dynamic> json) =>
      _$OarSaveRequestFromJson(json);
  Map<String, dynamic> toJson() => _$OarSaveRequestToJson(this);
}

@JsonSerializable()
class ArchiveJobResponse {
  final bool success;
  final String? jobId;
  final String? message;
  final String? error;

  ArchiveJobResponse({
    required this.success,
    this.jobId,
    this.message,
    this.error,
  });

  factory ArchiveJobResponse.fromJson(Map<String, dynamic> json) =>
      _$ArchiveJobResponseFromJson(json);
  Map<String, dynamic> toJson() => _$ArchiveJobResponseToJson(this);
}

@JsonSerializable()
class ArchiveJobListResponse {
  final List<ArchiveJob> jobs;
  final int totalCount;
  final int activeCount;

  ArchiveJobListResponse({
    required this.jobs,
    required this.totalCount,
    required this.activeCount,
  });

  factory ArchiveJobListResponse.fromJson(Map<String, dynamic> json) =>
      _$ArchiveJobListResponseFromJson(json);
  Map<String, dynamic> toJson() => _$ArchiveJobListResponseToJson(this);
}

@JsonSerializable()
class ArchiveProgressUpdate {
  final String jobId;
  final String jobType;
  final String status;
  final double progress;
  final String? message;
  final int? itemsProcessed;
  final int? itemsTotal;
  final int elapsedMs;

  ArchiveProgressUpdate({
    required this.jobId,
    required this.jobType,
    required this.status,
    required this.progress,
    this.message,
    this.itemsProcessed,
    this.itemsTotal,
    required this.elapsedMs,
  });

  factory ArchiveProgressUpdate.fromJson(Map<String, dynamic> json) =>
      _$ArchiveProgressUpdateFromJson(json);
  Map<String, dynamic> toJson() => _$ArchiveProgressUpdateToJson(this);
}
