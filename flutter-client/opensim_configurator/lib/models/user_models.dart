import 'package:json_annotation/json_annotation.dart';

part 'user_models.g.dart';

enum UserStatus {
  @JsonValue('active')
  active,
  @JsonValue('inactive')
  inactive,
  @JsonValue('suspended')
  suspended,
  @JsonValue('pending')
  pending,
}

enum WearableType {
  @JsonValue(0)
  shape,
  @JsonValue(1)
  skin,
  @JsonValue(2)
  hair,
  @JsonValue(3)
  eyes,
  @JsonValue(4)
  shirt,
  @JsonValue(5)
  pants,
  @JsonValue(6)
  shoes,
  @JsonValue(7)
  socks,
  @JsonValue(8)
  jacket,
  @JsonValue(9)
  gloves,
  @JsonValue(10)
  undershirt,
  @JsonValue(11)
  underpants,
  @JsonValue(12)
  skirt,
  @JsonValue(13)
  alpha,
  @JsonValue(14)
  tattoo,
  @JsonValue(15)
  physics,
  @JsonValue(16)
  universal,
}

enum AppearanceStatus {
  @JsonValue('complete')
  complete,
  @JsonValue('incomplete')
  incomplete,
  @JsonValue('missing')
  missing,
  @JsonValue('error')
  error,
}

@JsonSerializable()
class UserAccount {
  final String id;
  final String firstName;
  final String lastName;
  final String? email;
  final int created;
  final int userLevel;
  final int userFlags;
  final String? userTitle;
  final UserStatus status;

  UserAccount({
    required this.id,
    required this.firstName,
    required this.lastName,
    this.email,
    required this.created,
    this.userLevel = 0,
    this.userFlags = 0,
    this.userTitle,
    this.status = UserStatus.active,
  });

  String get fullName => '$firstName $lastName';

  DateTime get createdDate => DateTime.fromMillisecondsSinceEpoch(created * 1000);

  String get createdFormatted {
    final date = createdDate;
    return '${date.year}-${date.month.toString().padLeft(2, '0')}-${date.day.toString().padLeft(2, '0')}';
  }

  bool get isAdmin => userLevel >= 200;
  bool get isGod => userLevel >= 250;

  factory UserAccount.fromJson(Map<String, dynamic> json) =>
      _$UserAccountFromJson(json);
  Map<String, dynamic> toJson() => _$UserAccountToJson(this);
}

@JsonSerializable()
class WearableEntry {
  final WearableType type;
  final String itemId;
  final String assetId;
  final String? name;
  final bool isValid;

  WearableEntry({
    required this.type,
    required this.itemId,
    required this.assetId,
    this.name,
    this.isValid = true,
  });

  String get typeName {
    switch (type) {
      case WearableType.shape:
        return 'Shape';
      case WearableType.skin:
        return 'Skin';
      case WearableType.hair:
        return 'Hair';
      case WearableType.eyes:
        return 'Eyes';
      case WearableType.shirt:
        return 'Shirt';
      case WearableType.pants:
        return 'Pants';
      case WearableType.shoes:
        return 'Shoes';
      case WearableType.socks:
        return 'Socks';
      case WearableType.jacket:
        return 'Jacket';
      case WearableType.gloves:
        return 'Gloves';
      case WearableType.undershirt:
        return 'Undershirt';
      case WearableType.underpants:
        return 'Underpants';
      case WearableType.skirt:
        return 'Skirt';
      case WearableType.alpha:
        return 'Alpha';
      case WearableType.tattoo:
        return 'Tattoo';
      case WearableType.physics:
        return 'Physics';
      case WearableType.universal:
        return 'Universal';
    }
  }

  bool get isBodyPart =>
      type == WearableType.shape ||
      type == WearableType.skin ||
      type == WearableType.hair ||
      type == WearableType.eyes;

  factory WearableEntry.fromJson(Map<String, dynamic> json) =>
      _$WearableEntryFromJson(json);
  Map<String, dynamic> toJson() => _$WearableEntryToJson(this);
}

@JsonSerializable()
class InventoryFolder {
  final String folderId;
  final String parentFolderId;
  final String folderName;
  final int type;
  final int version;
  final int itemCount;

  InventoryFolder({
    required this.folderId,
    required this.parentFolderId,
    required this.folderName,
    required this.type,
    this.version = 1,
    this.itemCount = 0,
  });

  String get typeName {
    switch (type) {
      case 0:
        return 'Texture';
      case 1:
        return 'Sound';
      case 2:
        return 'Calling Card';
      case 3:
        return 'Landmark';
      case 5:
        return 'Clothing';
      case 6:
        return 'Object';
      case 7:
        return 'Notecard';
      case 8:
        return 'Root';
      case 10:
        return 'Script';
      case 13:
        return 'Body Part';
      case 14:
        return 'Trash';
      case 15:
        return 'Snapshot';
      case 16:
        return 'Lost And Found';
      case 20:
        return 'Animation';
      case 21:
        return 'Gesture';
      case 23:
        return 'Favorites';
      case 47:
        return 'Current Outfit';
      case 48:
        return 'Outfit';
      case 49:
        return 'My Outfits';
      case 50:
        return 'Mesh';
      case 56:
        return 'Settings';
      case 57:
        return 'Material';
      default:
        return 'Unknown ($type)';
    }
  }

  factory InventoryFolder.fromJson(Map<String, dynamic> json) =>
      _$InventoryFolderFromJson(json);
  Map<String, dynamic> toJson() => _$InventoryFolderToJson(this);
}

@JsonSerializable()
class AppearanceDiagnostics {
  final String userId;
  final AppearanceStatus status;
  final List<WearableEntry> wearables;
  final List<InventoryFolder> folders;
  final int expectedFolderCount;
  final int actualFolderCount;
  final int expectedWearableCount;
  final int actualWearableCount;
  final List<String> missingFolders;
  final List<String> missingWearables;
  final List<String> invalidItems;
  final DateTime checkedAt;

  AppearanceDiagnostics({
    required this.userId,
    required this.status,
    required this.wearables,
    required this.folders,
    this.expectedFolderCount = 21,
    required this.actualFolderCount,
    this.expectedWearableCount = 6,
    required this.actualWearableCount,
    this.missingFolders = const [],
    this.missingWearables = const [],
    this.invalidItems = const [],
    required this.checkedAt,
  });

  bool get isHealthy => status == AppearanceStatus.complete;

  double get folderCompleteness =>
      expectedFolderCount > 0 ? actualFolderCount / expectedFolderCount : 0;

  double get wearableCompleteness =>
      expectedWearableCount > 0 ? actualWearableCount / expectedWearableCount : 0;

  bool get hasBodyParts {
    final bodyPartTypes = [
      WearableType.shape,
      WearableType.skin,
      WearableType.hair,
      WearableType.eyes,
    ];
    return bodyPartTypes.every(
        (type) => wearables.any((w) => w.type == type && w.isValid));
  }

  bool get hasClothing {
    return wearables.any((w) =>
        (w.type == WearableType.shirt || w.type == WearableType.pants) &&
        w.isValid);
  }

  factory AppearanceDiagnostics.fromJson(Map<String, dynamic> json) =>
      _$AppearanceDiagnosticsFromJson(json);
  Map<String, dynamic> toJson() => _$AppearanceDiagnosticsToJson(this);
}

@JsonSerializable()
class UserCreateRequest {
  final String firstName;
  final String lastName;
  final String password;
  final String? email;
  final int userLevel;

  UserCreateRequest({
    required this.firstName,
    required this.lastName,
    required this.password,
    this.email,
    this.userLevel = 0,
  });

  factory UserCreateRequest.fromJson(Map<String, dynamic> json) =>
      _$UserCreateRequestFromJson(json);
  Map<String, dynamic> toJson() => _$UserCreateRequestToJson(this);
}

@JsonSerializable()
class UserCreateResponse {
  final bool success;
  final String? userId;
  final String? message;
  final String? error;

  UserCreateResponse({
    required this.success,
    this.userId,
    this.message,
    this.error,
  });

  factory UserCreateResponse.fromJson(Map<String, dynamic> json) =>
      _$UserCreateResponseFromJson(json);
  Map<String, dynamic> toJson() => _$UserCreateResponseToJson(this);
}

@JsonSerializable()
class UserListResponse {
  final List<UserAccount> users;
  final int totalCount;
  final int page;
  final int pageSize;

  UserListResponse({
    required this.users,
    required this.totalCount,
    this.page = 1,
    this.pageSize = 50,
  });

  int get totalPages => (totalCount / pageSize).ceil();
  bool get hasMore => page < totalPages;

  factory UserListResponse.fromJson(Map<String, dynamic> json) =>
      _$UserListResponseFromJson(json);
  Map<String, dynamic> toJson() => _$UserListResponseToJson(this);
}
