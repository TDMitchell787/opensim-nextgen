// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'user_models.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

UserAccount _$UserAccountFromJson(Map<String, dynamic> json) => UserAccount(
  id: json['id'] as String,
  firstName: json['firstName'] as String,
  lastName: json['lastName'] as String,
  email: json['email'] as String?,
  created: (json['created'] as num).toInt(),
  userLevel: (json['userLevel'] as num?)?.toInt() ?? 0,
  userFlags: (json['userFlags'] as num?)?.toInt() ?? 0,
  userTitle: json['userTitle'] as String?,
  status:
      $enumDecodeNullable(_$UserStatusEnumMap, json['status']) ??
      UserStatus.active,
);

Map<String, dynamic> _$UserAccountToJson(UserAccount instance) =>
    <String, dynamic>{
      'id': instance.id,
      'firstName': instance.firstName,
      'lastName': instance.lastName,
      'email': instance.email,
      'created': instance.created,
      'userLevel': instance.userLevel,
      'userFlags': instance.userFlags,
      'userTitle': instance.userTitle,
      'status': _$UserStatusEnumMap[instance.status]!,
    };

const _$UserStatusEnumMap = {
  UserStatus.active: 'active',
  UserStatus.inactive: 'inactive',
  UserStatus.suspended: 'suspended',
  UserStatus.pending: 'pending',
};

WearableEntry _$WearableEntryFromJson(Map<String, dynamic> json) =>
    WearableEntry(
      type: $enumDecode(_$WearableTypeEnumMap, json['type']),
      itemId: json['itemId'] as String,
      assetId: json['assetId'] as String,
      name: json['name'] as String?,
      isValid: json['isValid'] as bool? ?? true,
    );

Map<String, dynamic> _$WearableEntryToJson(WearableEntry instance) =>
    <String, dynamic>{
      'type': _$WearableTypeEnumMap[instance.type]!,
      'itemId': instance.itemId,
      'assetId': instance.assetId,
      'name': instance.name,
      'isValid': instance.isValid,
    };

const _$WearableTypeEnumMap = {
  WearableType.shape: 0,
  WearableType.skin: 1,
  WearableType.hair: 2,
  WearableType.eyes: 3,
  WearableType.shirt: 4,
  WearableType.pants: 5,
  WearableType.shoes: 6,
  WearableType.socks: 7,
  WearableType.jacket: 8,
  WearableType.gloves: 9,
  WearableType.undershirt: 10,
  WearableType.underpants: 11,
  WearableType.skirt: 12,
  WearableType.alpha: 13,
  WearableType.tattoo: 14,
  WearableType.physics: 15,
  WearableType.universal: 16,
};

InventoryFolder _$InventoryFolderFromJson(Map<String, dynamic> json) =>
    InventoryFolder(
      folderId: json['folderId'] as String,
      parentFolderId: json['parentFolderId'] as String,
      folderName: json['folderName'] as String,
      type: (json['type'] as num).toInt(),
      version: (json['version'] as num?)?.toInt() ?? 1,
      itemCount: (json['itemCount'] as num?)?.toInt() ?? 0,
    );

Map<String, dynamic> _$InventoryFolderToJson(InventoryFolder instance) =>
    <String, dynamic>{
      'folderId': instance.folderId,
      'parentFolderId': instance.parentFolderId,
      'folderName': instance.folderName,
      'type': instance.type,
      'version': instance.version,
      'itemCount': instance.itemCount,
    };

AppearanceDiagnostics _$AppearanceDiagnosticsFromJson(
  Map<String, dynamic> json,
) => AppearanceDiagnostics(
  userId: json['userId'] as String,
  status: $enumDecode(_$AppearanceStatusEnumMap, json['status']),
  wearables:
      (json['wearables'] as List<dynamic>)
          .map((e) => WearableEntry.fromJson(e as Map<String, dynamic>))
          .toList(),
  folders:
      (json['folders'] as List<dynamic>)
          .map((e) => InventoryFolder.fromJson(e as Map<String, dynamic>))
          .toList(),
  expectedFolderCount: (json['expectedFolderCount'] as num?)?.toInt() ?? 21,
  actualFolderCount: (json['actualFolderCount'] as num).toInt(),
  expectedWearableCount: (json['expectedWearableCount'] as num?)?.toInt() ?? 6,
  actualWearableCount: (json['actualWearableCount'] as num).toInt(),
  missingFolders:
      (json['missingFolders'] as List<dynamic>?)
          ?.map((e) => e as String)
          .toList() ??
      const [],
  missingWearables:
      (json['missingWearables'] as List<dynamic>?)
          ?.map((e) => e as String)
          .toList() ??
      const [],
  invalidItems:
      (json['invalidItems'] as List<dynamic>?)
          ?.map((e) => e as String)
          .toList() ??
      const [],
  checkedAt: DateTime.parse(json['checkedAt'] as String),
);

Map<String, dynamic> _$AppearanceDiagnosticsToJson(
  AppearanceDiagnostics instance,
) => <String, dynamic>{
  'userId': instance.userId,
  'status': _$AppearanceStatusEnumMap[instance.status]!,
  'wearables': instance.wearables,
  'folders': instance.folders,
  'expectedFolderCount': instance.expectedFolderCount,
  'actualFolderCount': instance.actualFolderCount,
  'expectedWearableCount': instance.expectedWearableCount,
  'actualWearableCount': instance.actualWearableCount,
  'missingFolders': instance.missingFolders,
  'missingWearables': instance.missingWearables,
  'invalidItems': instance.invalidItems,
  'checkedAt': instance.checkedAt.toIso8601String(),
};

const _$AppearanceStatusEnumMap = {
  AppearanceStatus.complete: 'complete',
  AppearanceStatus.incomplete: 'incomplete',
  AppearanceStatus.missing: 'missing',
  AppearanceStatus.error: 'error',
};

UserCreateRequest _$UserCreateRequestFromJson(Map<String, dynamic> json) =>
    UserCreateRequest(
      firstName: json['firstName'] as String,
      lastName: json['lastName'] as String,
      password: json['password'] as String,
      email: json['email'] as String?,
      userLevel: (json['userLevel'] as num?)?.toInt() ?? 0,
    );

Map<String, dynamic> _$UserCreateRequestToJson(UserCreateRequest instance) =>
    <String, dynamic>{
      'firstName': instance.firstName,
      'lastName': instance.lastName,
      'password': instance.password,
      'email': instance.email,
      'userLevel': instance.userLevel,
    };

UserCreateResponse _$UserCreateResponseFromJson(Map<String, dynamic> json) =>
    UserCreateResponse(
      success: json['success'] as bool,
      userId: json['userId'] as String?,
      message: json['message'] as String?,
      error: json['error'] as String?,
    );

Map<String, dynamic> _$UserCreateResponseToJson(UserCreateResponse instance) =>
    <String, dynamic>{
      'success': instance.success,
      'userId': instance.userId,
      'message': instance.message,
      'error': instance.error,
    };

UserListResponse _$UserListResponseFromJson(Map<String, dynamic> json) =>
    UserListResponse(
      users:
          (json['users'] as List<dynamic>)
              .map((e) => UserAccount.fromJson(e as Map<String, dynamic>))
              .toList(),
      totalCount: (json['totalCount'] as num).toInt(),
      page: (json['page'] as num?)?.toInt() ?? 1,
      pageSize: (json['pageSize'] as num?)?.toInt() ?? 50,
    );

Map<String, dynamic> _$UserListResponseToJson(UserListResponse instance) =>
    <String, dynamic>{
      'users': instance.users,
      'totalCount': instance.totalCount,
      'page': instance.page,
      'pageSize': instance.pageSize,
    };
