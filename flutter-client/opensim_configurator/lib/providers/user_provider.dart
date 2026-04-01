import 'package:flutter/foundation.dart';
import '../models/user_models.dart';
import '../services/user_service.dart';

class UserProvider extends ChangeNotifier {
  UserService? _service;
  List<UserAccount> _users = [];
  UserAccount? _selectedUser;
  AppearanceDiagnostics? _selectedUserDiagnostics;
  bool _isLoading = false;
  String? _errorMessage;
  int _totalCount = 0;
  int _currentPage = 1;
  final int _pageSize = 50;

  List<UserAccount> get users => _users;
  UserAccount? get selectedUser => _selectedUser;
  AppearanceDiagnostics? get selectedUserDiagnostics => _selectedUserDiagnostics;
  bool get isLoading => _isLoading;
  String? get errorMessage => _errorMessage;
  int get totalCount => _totalCount;
  int get currentPage => _currentPage;
  int get pageSize => _pageSize;
  int get totalPages => (_totalCount / _pageSize).ceil();
  bool get hasMore => _currentPage < totalPages;

  void configure(String baseUrl, String? apiKey) {
    _service = UserService(baseUrl: baseUrl, apiKey: apiKey);
    notifyListeners();
  }

  bool get isConfigured => _service != null;

  Future<void> loadUsers({bool refresh = false}) async {
    if (_service == null) {
      _errorMessage = 'Service not configured';
      notifyListeners();
      return;
    }

    if (refresh) {
      _currentPage = 1;
    }

    _isLoading = true;
    _errorMessage = null;
    notifyListeners();

    try {
      final response = await _service!.getUsers(
        page: _currentPage,
        pageSize: _pageSize,
      );

      if (refresh) {
        _users = response.users;
      } else {
        _users.addAll(response.users);
      }
      _totalCount = response.totalCount;
      _errorMessage = null;
    } catch (e) {
      _errorMessage = e.toString();
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<void> loadMoreUsers() async {
    if (!hasMore || _isLoading) return;
    _currentPage++;
    await loadUsers();
  }

  Future<void> selectUser(String userId) async {
    if (_service == null) return;

    _isLoading = true;
    notifyListeners();

    try {
      _selectedUser = await _service!.getUser(userId);
      if (_selectedUser != null) {
        await loadUserDiagnostics(userId);
      }
    } catch (e) {
      _errorMessage = e.toString();
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  void clearSelection() {
    _selectedUser = null;
    _selectedUserDiagnostics = null;
    notifyListeners();
  }

  Future<void> loadUserDiagnostics(String userId) async {
    if (_service == null) return;

    try {
      _selectedUserDiagnostics = await _service!.getAppearanceDiagnostics(userId);
    } catch (e) {
      _selectedUserDiagnostics = null;
    }
    notifyListeners();
  }

  Future<UserCreateResponse> createUser(UserCreateRequest request) async {
    if (_service == null) {
      return UserCreateResponse(success: false, error: 'Service not configured');
    }

    _isLoading = true;
    notifyListeners();

    try {
      final response = await _service!.createUser(request);
      if (response.success) {
        await loadUsers(refresh: true);
      }
      return response;
    } catch (e) {
      return UserCreateResponse(success: false, error: e.toString());
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<bool> deleteUser(String userId) async {
    if (_service == null) return false;

    _isLoading = true;
    notifyListeners();

    try {
      final success = await _service!.deleteUser(userId);
      if (success) {
        _users.removeWhere((u) => u.id == userId);
        if (_selectedUser?.id == userId) {
          _selectedUser = null;
          _selectedUserDiagnostics = null;
        }
        _totalCount--;
      }
      return success;
    } catch (e) {
      _errorMessage = e.toString();
      return false;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<bool> repairAppearance(String userId) async {
    if (_service == null) return false;

    _isLoading = true;
    notifyListeners();

    try {
      final success = await _service!.repairAppearance(userId);
      if (success) {
        await loadUserDiagnostics(userId);
      }
      return success;
    } catch (e) {
      _errorMessage = e.toString();
      return false;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<bool> resetAppearance(String userId) async {
    if (_service == null) return false;

    _isLoading = true;
    notifyListeners();

    try {
      final success = await _service!.resetAppearance(userId);
      if (success) {
        await loadUserDiagnostics(userId);
      }
      return success;
    } catch (e) {
      _errorMessage = e.toString();
      return false;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<bool> rebuildInventory(String userId) async {
    if (_service == null) return false;

    _isLoading = true;
    notifyListeners();

    try {
      final success = await _service!.rebuildInventory(userId);
      if (success) {
        await loadUserDiagnostics(userId);
      }
      return success;
    } catch (e) {
      _errorMessage = e.toString();
      return false;
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  Future<bool> resetPassword(String userId, String newPassword) async {
    if (_service == null) return false;

    try {
      return await _service!.resetPassword(userId, newPassword);
    } catch (e) {
      _errorMessage = e.toString();
      return false;
    }
  }

  Color getStatusColor(UserStatus status) {
    switch (status) {
      case UserStatus.active:
        return const Color(0xFF4CAF50);
      case UserStatus.inactive:
        return const Color(0xFF9E9E9E);
      case UserStatus.suspended:
        return const Color(0xFFF44336);
      case UserStatus.pending:
        return const Color(0xFFFF9800);
    }
  }

  Color getAppearanceStatusColor(AppearanceStatus status) {
    switch (status) {
      case AppearanceStatus.complete:
        return const Color(0xFF4CAF50);
      case AppearanceStatus.incomplete:
        return const Color(0xFFFF9800);
      case AppearanceStatus.missing:
        return const Color(0xFFF44336);
      case AppearanceStatus.error:
        return const Color(0xFF9C27B0);
    }
  }

  String getAppearanceStatusLabel(AppearanceStatus status) {
    switch (status) {
      case AppearanceStatus.complete:
        return 'Complete';
      case AppearanceStatus.incomplete:
        return 'Incomplete';
      case AppearanceStatus.missing:
        return 'Missing';
      case AppearanceStatus.error:
        return 'Error';
    }
  }
}
