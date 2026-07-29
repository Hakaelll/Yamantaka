import '../../reader/domain/reader_models.dart';

final class ReadingProfile {
  const ReadingProfile({
    required this.id,
    required this.name,
    required this.isBuiltin,
    required this.theme,
    required this.margin,
    required this.layout,
    required this.zoomMode,
    required this.denseChrome,
    required this.autoHideChrome,
    required this.reduceMotion,
    required this.electronicInk,
    required this.highContrast,
  });

  final String id;
  final String name;
  final bool isBuiltin;
  final ReaderTheme theme;
  final double margin;
  final PageLayoutMode layout;
  final ZoomMode zoomMode;
  final bool denseChrome;
  final bool autoHideChrome;
  final bool reduceMotion;
  final bool electronicInk;
  final bool highContrast;

  DocumentPreferences apply(DocumentPreferences preferences) =>
      preferences.copyWith(
        readingTheme: theme,
        horizontalMargin: margin,
        verticalMargin: margin,
        pageLayout: electronicInk ? PageLayoutMode.single : layout,
        zoomMode: zoomMode,
        controlsAutoHide: autoHideChrome,
      );
}

abstract final class BuiltinReadingProfiles {
  static const all = <ReadingProfile>[
    ReadingProfile(
      id: 'builtin-reading',
      name: 'Lectura',
      isBuiltin: true,
      theme: ReaderTheme.ivory,
      margin: 32,
      layout: PageLayoutMode.continuous,
      zoomMode: ZoomMode.fitPage,
      denseChrome: false,
      autoHideChrome: true,
      reduceMotion: false,
      electronicInk: false,
      highContrast: false,
    ),
    ReadingProfile(
      id: 'builtin-study',
      name: 'Estudio',
      isBuiltin: true,
      theme: ReaderTheme.light,
      margin: 24,
      layout: PageLayoutMode.continuous,
      zoomMode: ZoomMode.fitWidth,
      denseChrome: true,
      autoHideChrome: false,
      reduceMotion: false,
      electronicInk: false,
      highContrast: false,
    ),
    ReadingProfile(
      id: 'builtin-presentation',
      name: 'Presentación',
      isBuiltin: true,
      theme: ReaderTheme.black,
      margin: 0,
      layout: PageLayoutMode.single,
      zoomMode: ZoomMode.fitPage,
      denseChrome: true,
      autoHideChrome: true,
      reduceMotion: true,
      electronicInk: false,
      highContrast: false,
    ),
    ReadingProfile(
      id: 'builtin-night',
      name: 'Noche',
      isBuiltin: true,
      theme: ReaderTheme.dark,
      margin: 24,
      layout: PageLayoutMode.continuous,
      zoomMode: ZoomMode.fitPage,
      denseChrome: false,
      autoHideChrome: true,
      reduceMotion: true,
      electronicInk: false,
      highContrast: false,
    ),
    ReadingProfile(
      id: 'builtin-eink',
      name: 'Tinta electrónica',
      isBuiltin: true,
      theme: ReaderTheme.light,
      margin: 16,
      layout: PageLayoutMode.single,
      zoomMode: ZoomMode.fitPage,
      denseChrome: true,
      autoHideChrome: true,
      reduceMotion: true,
      electronicInk: true,
      highContrast: true,
    ),
    ReadingProfile(
      id: 'builtin-comparison',
      name: 'Comparación',
      isBuiltin: true,
      theme: ReaderTheme.gray,
      margin: 12,
      layout: PageLayoutMode.single,
      zoomMode: ZoomMode.fitPage,
      denseChrome: true,
      autoHideChrome: false,
      reduceMotion: true,
      electronicInk: false,
      highContrast: false,
    ),
  ];
}
