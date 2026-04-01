// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'archive_models.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

ArchiveJob _$ArchiveJobFromJson(Map<String, dynamic> json) => ArchiveJob(
      id: json['id'] as String,
      archiveType: $enumDecode(_$ArchiveTypeEnumMap, json['archiveType']),
      operation: $enumDecode(_$JobOperationEnumMap, json['operation']),
      status: $enumDecode(_$JobStatusEnumMap, json['status']),
      progress: (json['progress'] as num?)?.toDouble() ?? 0.0,
      progressMessage: json['progressMessage'] as String?,
      targetId: json['targetId'] as String?,
      targetName: json['targetName'] as String?,
      filePath: json['filePath'] as String?,
      createdAt: DateTime.parse(json['createdAt'] as String),
      startedAt: json['startedAt'] == null
          ? null
          : DateTime.parse(json['startedAt'] as String),
      completedAt: json['completedAt'] == null
          ? null
          : DateTime.parse(json['completedAt'] as String),
      error: json['error'] as String?,
      result: json['result'] == null
          ? null
          : ArchiveJobResult.fromJson(json['result'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$ArchiveJobToJson(ArchiveJob instance) =>
    <String, dynamic>{
      'id': instance.id,
      'archiveType': _$ArchiveTypeEnumMap[instance.archiveType]!,
      'operation': _$JobOperationEnumMap[instance.operation]!,
      'status': _$JobStatusEnumMap[instance.status]!,
      'progress': instance.progress,
      'progressMessage': instance.progressMessage,
      'targetId': instance.targetId,
      'targetName': instance.targetName,
      'filePath': instance.filePath,
      'createdAt': instance.createdAt.toIso8601String(),
      'startedAt': instance.startedAt?.toIso8601String(),
      'completedAt': instance.completedAt?.toIso8601String(),
      'error': instance.error,
      'result': instance.result?.toJson(),
    };

const _$ArchiveTypeEnumMap = {
  ArchiveType.iar: 'iar',
  ArchiveType.oar: 'oar',
};

const _$JobOperationEnumMap = {
  JobOperation.load: 'load',
  JobOperation.save: 'save',
};

const _$JobStatusEnumMap = {
  JobStatus.queued: 'queued',
  JobStatus.running: 'running',
  JobStatus.completed: 'completed',
  JobStatus.failed: 'failed',
  JobStatus.cancelled: 'cancelled',
};

ArchiveJobResult _$ArchiveJobResultFromJson(Map<String, dynamic> json) =>
    ArchiveJobResult(
      assetsProcessed: (json['assetsProcessed'] as num?)?.toInt(),
      foldersProcessed: (json['foldersProcessed'] as num?)?.toInt(),
      itemsProcessed: (json['itemsProcessed'] as num?)?.toInt(),
      objectsProcessed: (json['objectsProcessed'] as num?)?.toInt(),
      parcelsProcessed: (json['parcelsProcessed'] as num?)?.toInt(),
      terrainProcessed: json['terrainProcessed'] as bool?,
      archiveSizeBytes: (json['archiveSizeBytes'] as num?)?.toInt(),
      downloadPath: json['downloadPath'] as String?,
    );

Map<String, dynamic> _$ArchiveJobResultToJson(ArchiveJobResult instance) =>
    <String, dynamic>{
      'assetsProcessed': instance.assetsProcessed,
      'foldersProcessed': instance.foldersProcessed,
      'itemsProcessed': instance.itemsProcessed,
      'objectsProcessed': instance.objectsProcessed,
      'parcelsProcessed': instance.parcelsProcessed,
      'terrainProcessed': instance.terrainProcessed,
      'archiveSizeBytes': instance.archiveSizeBytes,
      'downloadPath': instance.downloadPath,
    };

IarLoadRequest _$IarLoadRequestFromJson(Map<String, dynamic> json) =>
    IarLoadRequest(
      userId: json['userId'] as String,
      targetFolderId: json['targetFolderId'] as String?,
      merge: json['merge'] as bool? ?? false,
      filePath: json['filePath'] as String,
    );

Map<String, dynamic> _$IarLoadRequestToJson(IarLoadRequest instance) =>
    <String, dynamic>{
      'userId': instance.userId,
      'targetFolderId': instance.targetFolderId,
      'merge': instance.merge,
      'filePath': instance.filePath,
    };

IarSaveRequest _$IarSaveRequestFromJson(Map<String, dynamic> json) =>
    IarSaveRequest(
      userId: json['userId'] as String,
      folderId: json['folderId'] as String?,
      includeAssets: json['includeAssets'] as bool? ?? true,
    );

Map<String, dynamic> _$IarSaveRequestToJson(IarSaveRequest instance) =>
    <String, dynamic>{
      'userId': instance.userId,
      'folderId': instance.folderId,
      'includeAssets': instance.includeAssets,
    };

OarLoadRequest _$OarLoadRequestFromJson(Map<String, dynamic> json) =>
    OarLoadRequest(
      regionId: json['regionId'] as String,
      filePath: json['filePath'] as String,
      merge: json['merge'] as bool? ?? false,
      loadTerrain: json['loadTerrain'] as bool? ?? true,
      loadObjects: json['loadObjects'] as bool? ?? true,
      loadParcels: json['loadParcels'] as bool? ?? true,
    );

Map<String, dynamic> _$OarLoadRequestToJson(OarLoadRequest instance) =>
    <String, dynamic>{
      'regionId': instance.regionId,
      'filePath': instance.filePath,
      'merge': instance.merge,
      'loadTerrain': instance.loadTerrain,
      'loadObjects': instance.loadObjects,
      'loadParcels': instance.loadParcels,
    };

OarSaveRequest _$OarSaveRequestFromJson(Map<String, dynamic> json) =>
    OarSaveRequest(
      regionId: json['regionId'] as String,
      includeAssets: json['includeAssets'] as bool? ?? true,
      includeTerrain: json['includeTerrain'] as bool? ?? true,
      includeObjects: json['includeObjects'] as bool? ?? true,
      includeParcels: json['includeParcels'] as bool? ?? true,
    );

Map<String, dynamic> _$OarSaveRequestToJson(OarSaveRequest instance) =>
    <String, dynamic>{
      'regionId': instance.regionId,
      'includeAssets': instance.includeAssets,
      'includeTerrain': instance.includeTerrain,
      'includeObjects': instance.includeObjects,
      'includeParcels': instance.includeParcels,
    };

ArchiveJobResponse _$ArchiveJobResponseFromJson(Map<String, dynamic> json) =>
    ArchiveJobResponse(
      success: json['success'] as bool,
      jobId: json['jobId'] as String?,
      message: json['message'] as String?,
      error: json['error'] as String?,
    );

Map<String, dynamic> _$ArchiveJobResponseToJson(ArchiveJobResponse instance) =>
    <String, dynamic>{
      'success': instance.success,
      'jobId': instance.jobId,
      'message': instance.message,
      'error': instance.error,
    };

ArchiveJobListResponse _$ArchiveJobListResponseFromJson(
        Map<String, dynamic> json) =>
    ArchiveJobListResponse(
      jobs: (json['jobs'] as List<dynamic>)
          .map((e) => ArchiveJob.fromJson(e as Map<String, dynamic>))
          .toList(),
      totalCount: (json['totalCount'] as num).toInt(),
      activeCount: (json['activeCount'] as num).toInt(),
    );

Map<String, dynamic> _$ArchiveJobListResponseToJson(
        ArchiveJobListResponse instance) =>
    <String, dynamic>{
      'jobs': instance.jobs.map((e) => e.toJson()).toList(),
      'totalCount': instance.totalCount,
      'activeCount': instance.activeCount,
    };

ArchiveProgressUpdate _$ArchiveProgressUpdateFromJson(
        Map<String, dynamic> json) =>
    ArchiveProgressUpdate(
      jobId: json['jobId'] as String,
      jobType: json['jobType'] as String,
      status: json['status'] as String,
      progress: (json['progress'] as num).toDouble(),
      message: json['message'] as String?,
      itemsProcessed: (json['itemsProcessed'] as num?)?.toInt(),
      itemsTotal: (json['itemsTotal'] as num?)?.toInt(),
      elapsedMs: (json['elapsedMs'] as num).toInt(),
    );

Map<String, dynamic> _$ArchiveProgressUpdateToJson(
        ArchiveProgressUpdate instance) =>
    <String, dynamic>{
      'jobId': instance.jobId,
      'jobType': instance.jobType,
      'status': instance.status,
      'progress': instance.progress,
      'message': instance.message,
      'itemsProcessed': instance.itemsProcessed,
      'itemsTotal': instance.itemsTotal,
      'elapsedMs': instance.elapsedMs,
    };
