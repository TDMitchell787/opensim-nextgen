// OpenSim Next - Phase 30 Advanced Observability Platform
// Flutter Theme System with Multiple Dark Themes
// Using ELEGANT ARCHIVE SOLUTION methodology

import 'package:flutter/material.dart';

class ObservabilityThemes {
  // Theme Names
  static const String system = 'System';
  static const String light = 'Light';
  static const String dark = 'Dark';
  static const String opensimDark = 'OpenSim Dark';
  static const String matrix = 'Matrix';
  static const String cyberpunk = 'Cyberpunk';
  static const String dracula = 'Dracula';
  static const String monokai = 'Monokai';

  static final Map<String, String> themeNames = {
    system: '🌅 System',
    light: '☀️ Light Theme', 
    dark: '🌙 Dark Theme',
    opensimDark: '🌌 OpenSim Dark',
    matrix: '💚 Matrix Green',
    cyberpunk: '🔮 Cyberpunk',
    dracula: '🧛 Dracula',
    monokai: '🎨 Monokai',
  };

  static final Map<String, String> themeDescriptions = {
    system: 'Follows system dark/light mode',
    light: 'Clean and bright interface for daytime use',
    dark: 'Modern dark interface that\'s easy on the eyes',
    opensimDark: 'Professional dark theme with blue accents',
    matrix: 'Classic terminal-style green on black',
    cyberpunk: 'Futuristic neon theme with magenta and cyan',
    dracula: 'Popular dark theme with purple accents',
    monokai: 'Code editor inspired dark theme',
  };

  // Light Theme
  static ThemeData get lightTheme => ThemeData(
    useMaterial3: true,
    brightness: Brightness.light,
    colorScheme: ColorScheme.fromSeed(
      seedColor: const Color(0xFF2563EB), // Primary blue
      brightness: Brightness.light,
    ),
    visualDensity: VisualDensity.adaptivePlatformDensity,
  );

  // Dark Theme
  static ThemeData get darkTheme => ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: ColorScheme.fromSeed(
      seedColor: const Color(0xFF3B82F6), // Lighter blue
      brightness: Brightness.dark,
    ),
    visualDensity: VisualDensity.adaptivePlatformDensity,
  );

  // OpenSim Dark Theme
  static ThemeData get opensimDarkTheme => ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: const ColorScheme.dark(
      primary: Color(0xFF3B82F6),
      primaryContainer: Color(0xFF1E40AF),
      secondary: Color(0xFF10B981),
      secondaryContainer: Color(0xFF047857),
      surface: Color(0xFF1A2332),
      background: Color(0xFF0C1426),
      error: Color(0xFFEF4444),
      onPrimary: Color(0xFFFFFFFF),
      onSecondary: Color(0xFFFFFFFF),
      onSurface: Color(0xFFE2E8F0),
      onBackground: Color(0xFFE2E8F0),
      onError: Color(0xFFFFFFFF),
      brightness: Brightness.dark,
    ),
    visualDensity: VisualDensity.adaptivePlatformDensity,
    scaffoldBackgroundColor: const Color(0xFF0C1426),
    cardColor: const Color(0xFF1A2332),
    appBarTheme: const AppBarTheme(
      backgroundColor: Color(0xFF1A2332),
      foregroundColor: Color(0xFFE2E8F0),
    ),
  );

  // Matrix Green Theme
  static ThemeData get matrixTheme => ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: const ColorScheme.dark(
      primary: Color(0xFF00FF41),
      primaryContainer: Color(0xFF00CC33),
      secondary: Color(0xFF33FF66),
      secondaryContainer: Color(0xFF00CC33),
      surface: Color(0xFF0D1117),
      background: Color(0xFF000000),
      error: Color(0xFFFF5555),
      onPrimary: Color(0xFF000000),
      onSecondary: Color(0xFF000000),
      onSurface: Color(0xFF00FF41),
      onBackground: Color(0xFF00FF41),
      onError: Color(0xFF000000),
      brightness: Brightness.dark,
    ),
    visualDensity: VisualDensity.adaptivePlatformDensity,
    scaffoldBackgroundColor: const Color(0xFF000000),
    cardColor: const Color(0xFF0D1117),
    appBarTheme: const AppBarTheme(
      backgroundColor: Color(0xFF0D1117),
      foregroundColor: Color(0x00FF41),
    ),
  );

  // Cyberpunk Theme
  static ThemeData get cyberpunkTheme => ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: const ColorScheme.dark(
      primary: Color(0xFFFF00FF),
      primaryContainer: Color(0xFFCC00CC),
      secondary: Color(0xFF00FFFF),
      secondaryContainer: Color(0xFF00CCCC),
      surface: Color(0xFF1A0A1A),
      background: Color(0xFF0A0A0A),
      error: Color(0xFFFF0080),
      onPrimary: Color(0xFF000000),
      onSecondary: Color(0xFF000000),
      onSurface: Color(0xFFFF00FF),
      onBackground: Color(0xFFFF00FF),
      onError: Color(0xFF000000),
      brightness: Brightness.dark,
    ),
    visualDensity: VisualDensity.adaptivePlatformDensity,
    scaffoldBackgroundColor: const Color(0xFF0A0A0A),
    cardColor: const Color(0xFF1A0A1A),
    appBarTheme: const AppBarTheme(
      backgroundColor: Color(0xFF1A0A1A),
      foregroundColor: Color(0xFFFF00FF),
    ),
  );

  // Dracula Theme
  static ThemeData get draculaTheme => ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: const ColorScheme.dark(
      primary: Color(0xFFBD93F9),
      primaryContainer: Color(0xFF8B5CF6),
      secondary: Color(0xFF50FA7B),
      secondaryContainer: Color(0xFF22C55E),
      surface: Color(0xFF44475A),
      background: Color(0xFF282A36),
      error: Color(0xFFFF5555),
      onPrimary: Color(0xFF000000),
      onSecondary: Color(0xFF000000),
      onSurface: Color(0xFFF8F8F2),
      onBackground: Color(0xFFF8F8F2),
      onError: Color(0xFF000000),
      brightness: Brightness.dark,
    ),
    visualDensity: VisualDensity.adaptivePlatformDensity,
    scaffoldBackgroundColor: const Color(0xFF282A36),
    cardColor: const Color(0xFF44475A),
    appBarTheme: const AppBarTheme(
      backgroundColor: Color(0xFF44475A),
      foregroundColor: Color(0xFFF8F8F2),
    ),
  );

  // Monokai Theme
  static ThemeData get monokaiTheme => ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: const ColorScheme.dark(
      primary: Color(0xFF66D9EF),
      primaryContainer: Color(0xFF0EA5E9),
      secondary: Color(0xFFA6E22E),
      secondaryContainer: Color(0xFF65A30D),
      surface: Color(0xFF3E3D32),
      background: Color(0xFF272822),
      error: Color(0xFFF92672),
      onPrimary: Color(0xFF000000),
      onSecondary: Color(0xFF000000),
      onSurface: Color(0xFFF8F8F2),
      onBackground: Color(0xFFF8F8F2),
      onError: Color(0xFF000000),
      brightness: Brightness.dark,
    ),
    visualDensity: VisualDensity.adaptivePlatformDensity,
    scaffoldBackgroundColor: const Color(0xFF272822),
    cardColor: const Color(0xFF3E3D32),
    appBarTheme: const AppBarTheme(
      backgroundColor: Color(0xFF3E3D32),
      foregroundColor: Color(0xFFF8F8F2),
    ),
  );

  // Get theme by name
  static ThemeData getTheme(String themeName) {
    switch (themeName) {
      case light:
        return lightTheme;
      case dark:
        return darkTheme;
      case opensimDark:
        return opensimDarkTheme;
      case matrix:
        return matrixTheme;
      case cyberpunk:
        return cyberpunkTheme;
      case dracula:
        return draculaTheme;
      case monokai:
        return monokaiTheme;
      default:
        return lightTheme;
    }
  }

  // Get theme mode by name
  static ThemeMode getThemeMode(String themeName) {
    switch (themeName) {
      case system:
        return ThemeMode.system;
      case light:
        return ThemeMode.light;
      default:
        return ThemeMode.dark;
    }
  }

  // Get list of all theme names
  static List<String> get allThemes => [
    system,
    light,
    dark,
    opensimDark,
    matrix,
    cyberpunk,
    dracula,
    monokai,
  ];

  // Check if theme is dark
  static bool isDarkTheme(String themeName) {
    return themeName != light && themeName != system;
  }

  // Get chart colors for theme
  static Map<String, Color> getChartColors(String themeName) {
    switch (themeName) {
      case matrix:
        return {
          'primary': const Color(0xFF00FF41),
          'secondary': const Color(0xFF33FF66),
          'background': const Color(0xFF000000),
          'text': const Color(0xFF00FF41),
        };
      case cyberpunk:
        return {
          'primary': const Color(0xFFFF00FF),
          'secondary': const Color(0xFF00FFFF),
          'background': const Color(0xFF0A0A0A),
          'text': const Color(0xFFFF00FF),
        };
      case dracula:
        return {
          'primary': const Color(0xFFBD93F9),
          'secondary': const Color(0xFF50FA7B),
          'background': const Color(0xFF282A36),
          'text': const Color(0xFFF8F8F2),
        };
      case monokai:
        return {
          'primary': const Color(0xFF66D9EF),
          'secondary': const Color(0xFFA6E22E),
          'background': const Color(0xFF272822),
          'text': const Color(0xFFF8F8F2),
        };
      case opensimDark:
        return {
          'primary': const Color(0xFF3B82F6),
          'secondary': const Color(0xFF10B981),
          'background': const Color(0xFF0C1426),
          'text': const Color(0xFFE2E8F0),
        };
      default:
        return {
          'primary': const Color(0xFF2563EB),
          'secondary': const Color(0xFF10B981),
          'background': const Color(0xFFFFFFFF),
          'text': const Color(0xFF1E293B),
        };
    }
  }
}