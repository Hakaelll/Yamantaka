import 'dart:async';
import 'dart:io';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:window_manager/window_manager.dart';

import '../core/errors/app_failure.dart';
import '../features/library/presentation/library_screen.dart';
import '../features/reader/application/reader_controller.dart';
import '../features/reader/presentation/reader_screen.dart';
import '../features/settings/presentation/settings_screen.dart';
import 'app_config.dart';
import 'app_services.dart';
import 'shortcuts/app_commands.dart';
import 'theme/app_theme.dart';

final class YamantakaApp extends StatefulWidget {
  const YamantakaApp({super.key, required this.services, this.initialPath});

  final AppServices services;
  final String? initialPath;

  @override
  State<YamantakaApp> createState() => _YamantakaAppState();
}

final class _YamantakaAppState extends State<YamantakaApp>
    with WidgetsBindingObserver, WindowListener {
  final GlobalKey<NavigatorState> _navigatorKey = GlobalKey<NavigatorState>();
  _AppPage _page = _AppPage.library;
  Timer? _windowSaveTimer;

  bool get _isDesktop =>
      Platform.isWindows || Platform.isLinux || Platform.isMacOS;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    if (_isDesktop) windowManager.addListener(this);
    if (widget.initialPath case final path?) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        unawaited(_openPath(path));
      });
    }
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    widget.services.reader.setVisible(
      state == AppLifecycleState.resumed || state == AppLifecycleState.inactive,
    );
  }

  @override
  void dispose() {
    _windowSaveTimer?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    if (_isDesktop) windowManager.removeListener(this);
    unawaited(_saveWindowBounds().whenComplete(widget.services.dispose));
    super.dispose();
  }

  @override
  void onWindowResized() => _scheduleWindowBoundsSave();

  @override
  void onWindowMoved() => _scheduleWindowBoundsSave();

  @override
  void onWindowUnmaximize() => _scheduleWindowBoundsSave();

  @override
  void onWindowClose() => unawaited(_saveWindowBounds());

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.services.settings,
      builder: (context, _) => MaterialApp(
        navigatorKey: _navigatorKey,
        title: AppConfig.productName,
        debugShowCheckedModeBanner: false,
        theme: AppTheme.light(),
        darkTheme: AppTheme.dark(),
        themeMode: widget.services.settings.settings.darkMode
            ? ThemeMode.dark
            : ThemeMode.light,
        home: CallbackShortcuts(
          bindings: AppCommandRegistry.applicationShortcuts(
            openDocument: () => unawaited(_pickAndOpen()),
          ),
          child: Focus(autofocus: true, child: Builder(builder: _buildPage)),
        ),
      ),
    );
  }

  Widget _buildPage(BuildContext context) => switch (_page) {
    _AppPage.library => LibraryScreen(
      controller: widget.services.library,
      onOpenPath: _openPath,
      onPickDocument: _pickAndOpen,
      onOpenSettings: () => setState(() => _page = _AppPage.settings),
      showRecents: widget.services.settings.settings.showRecents,
    ),
    _AppPage.reader => ReaderScreen(
      controller: widget.services.reader,
      engine: widget.services.engine,
      viewerAdapter: widget.services.viewerAdapter,
      onBack: _closeReader,
      onOpenDocument: _pickAndOpen,
      onOpenSettings: () => setState(() => _page = _AppPage.settings),
      reduceMotion:
          widget.services.settings.settings.reduceMotion ||
          widget.services.settings.settings.disableAnimations,
      showProgress: widget.services.settings.settings.showProgress,
      readingDirection: widget.services.settings.settings.readingDirection,
    ),
    _AppPage.settings => SettingsScreen(
      services: widget.services,
      onBack: () => setState(
        () => _page = widget.services.reader.status == ReaderStatus.ready
            ? _AppPage.reader
            : _AppPage.library,
      ),
    ),
  };

  Future<void> _pickAndOpen() async {
    const pdfTypes = XTypeGroup(
      label: 'PDF',
      extensions: ['pdf'],
      mimeTypes: ['application/pdf'],
      uniformTypeIdentifiers: ['com.adobe.pdf'],
    );
    final selected = await openFile(acceptedTypeGroups: const [pdfTypes]);
    if (selected != null) await _openPath(selected.path);
  }

  Future<void> _openPath(String path) async {
    var opened = await widget.services.reader.openPath(path);
    if (!mounted) return;
    final failure = widget.services.reader.error;
    if (!opened &&
        failure != null &&
        (failure.kind == FailureKind.passwordRequired ||
            failure.kind == FailureKind.wrongPassword)) {
      final password = await _askForPassword(
        wrongPassword: failure.kind == FailureKind.wrongPassword,
      );
      if (password != null && password.isNotEmpty) {
        opened = await widget.services.reader.openPath(
          path,
          password: password,
        );
      }
    }
    if (!mounted) return;
    if (opened) {
      setState(() => _page = _AppPage.reader);
      return;
    }
    final message =
        widget.services.reader.error?.userMessage ??
        'No se pudo abrir el documento.';
    ScaffoldMessenger.of(
      _navigatorKey.currentContext ?? context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }

  Future<String?> _askForPassword({required bool wrongPassword}) async {
    final context = _navigatorKey.currentContext;
    if (context == null) return null;
    final textController = TextEditingController();
    final password = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(wrongPassword ? 'Contraseña incorrecta' : 'PDF protegido'),
        content: TextField(
          controller: textController,
          autofocus: true,
          obscureText: true,
          decoration: const InputDecoration(labelText: 'Contraseña'),
          onSubmitted: (value) => Navigator.pop(context, value),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, textController.text),
            child: const Text('Abrir'),
          ),
        ],
      ),
    );
    textController.dispose();
    return password;
  }

  Future<void> _closeReader() async {
    await widget.services.reader.closeDocument();
    await widget.services.library.load();
    if (mounted) setState(() => _page = _AppPage.library);
  }

  void _scheduleWindowBoundsSave() {
    _windowSaveTimer?.cancel();
    _windowSaveTimer = Timer(
      const Duration(milliseconds: 500),
      () => unawaited(_saveWindowBounds()),
    );
  }

  Future<void> _saveWindowBounds() async {
    if (!_isDesktop ||
        await windowManager.isFullScreen() ||
        await windowManager.isMaximized()) {
      return;
    }
    final bounds = await windowManager.getBounds();
    if (bounds.width < 640 || bounds.height < 480) return;
    await widget.services.settings.saveWindowBounds(
      x: bounds.left,
      y: bounds.top,
      width: bounds.width,
      height: bounds.height,
    );
  }
}

enum _AppPage { library, reader, settings }
