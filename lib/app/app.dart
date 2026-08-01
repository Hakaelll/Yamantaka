import 'dart:async';
import 'dart:io';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter/services.dart';
import 'package:window_manager/window_manager.dart';

import '../core/errors/app_failure.dart';
import '../core/errors/failure_presentation.dart';
import '../core/i18n/app_localizations.dart';
import '../core/performance/startup_timeline.dart';
import '../core/platform/activation_channel.dart';
import '../features/library/domain/library_document.dart';
import '../features/library/domain/library_selection.dart';
import '../features/library/domain/metadata_editor.dart';
import '../features/library/presentation/library_screen.dart';
import '../features/library/presentation/document_cover_picker.dart';
import '../features/library/presentation/folder_import_dialog.dart';
import '../features/help/presentation/help_center_dialog.dart';
import '../features/help/presentation/welcome_dialog.dart';
import '../features/navigation/application/navigation_history_controller.dart';
import '../features/reader/application/reader_controller.dart';
import '../features/reader/domain/reader_models.dart';
import '../features/reader/presentation/reader_screen.dart';
import '../features/settings/presentation/settings_screen.dart';
import '../features/search/domain/global_search_models.dart';
import '../features/search/presentation/global_search_dialog.dart';
import '../features/workspaces/domain/workspace_models.dart';
import '../features/workspaces/presentation/workspace_dialog.dart';
import '../storage/repositories/app_settings_repository.dart';
import 'app_config.dart';
import 'app_services.dart';
import 'shortcuts/app_commands.dart';
import 'theme/app_theme.dart';
import 'theme/app_tokens.dart';

@visibleForTesting
ThemeMode resolveAppThemeMode(AppSettings settings, {required bool safeMode}) {
  // Safe mode protects restoration and indexing; changing palette is a
  // presentation-only operation and remains available.
  return settings.darkMode ? ThemeMode.dark : ThemeMode.light;
}

final class YamantakaApp extends StatefulWidget {
  const YamantakaApp({
    super.key,
    required this.services,
    this.initialPath,
    this.initialPaths = const [],
  });

  final AppServices services;
  final String? initialPath;
  final List<String> initialPaths;

  @override
  State<YamantakaApp> createState() => _YamantakaAppState();
}

final class _YamantakaAppState extends State<YamantakaApp>
    with WidgetsBindingObserver, WindowListener {
  final GlobalKey<NavigatorState> _navigatorKey = GlobalKey<NavigatorState>();
  _AppPage _page = _AppPage.library;
  Timer? _windowSaveTimer;
  bool _restorationAttempted = false;

  bool get _isDesktop =>
      Platform.isWindows || Platform.isLinux || Platform.isMacOS;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    unawaited(
      ActivationChannel.install((paths) async {
        if (mounted) await _openPaths(paths);
      }),
    );
    widget.services.reader.addListener(_syncActiveTab);
    if (_isDesktop) windowManager.addListener(this);
    final startupPaths = widget.initialPaths.isNotEmpty
        ? widget.initialPaths
        : widget.initialPath == null
        ? const <String>[]
        : [widget.initialPath!];
    if (startupPaths.isNotEmpty) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _completeFirstFrame();
        unawaited(_openPaths(startupPaths));
      });
    } else {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _completeFirstFrame();
        unawaited(_showWelcomeOrRestore());
      });
    }
  }

  Future<void> _showWelcomeOrRestore() async {
    if (!mounted || widget.services.safeMode) {
      await _restoreLastSession();
      return;
    }
    if (widget.services.settings.settings.hasSeenWelcome) {
      await _restoreLastSession();
      return;
    }
    final action = await showDialog<WelcomeAction>(
      context: context,
      barrierDismissible: false,
      builder: (_) => const WelcomeDialog(),
    );
    await widget.services.settings.update(
      widget.services.settings.settings.copyWith(hasSeenWelcome: true),
    );
    if (!mounted) return;
    switch (action) {
      case WelcomeAction.open:
        await _pickAndOpen();
      case WelcomeAction.help:
        await _showHelp();
        await _restoreLastSession();
      case WelcomeAction.dismiss:
      case null:
        await _restoreLastSession();
    }
  }

  Future<void> _showHelp() async {
    await showDialog<void>(
      context: context,
      builder: (_) => HelpCenterDialog(
        language: AppLanguageCode.fromCode(
          widget.services.settings.settings.languageCode,
        ),
      ),
    );
  }

  void _completeFirstFrame() {
    widget.services.startup.complete(StartupPhase.firstFrame);
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    widget.services.reader.setVisible(
      state == AppLifecycleState.resumed || state == AppLifecycleState.inactive,
    );
    widget.services.folderWatch.setPaused(
      state == AppLifecycleState.hidden ||
          state == AppLifecycleState.paused ||
          state == AppLifecycleState.detached,
    );
    widget.services.jobs.setBackgroundPaused(
      state == AppLifecycleState.hidden ||
          state == AppLifecycleState.paused ||
          state == AppLifecycleState.detached,
    );
    widget.services.libraryIndex.setBackgroundPaused(
      state == AppLifecycleState.hidden ||
          state == AppLifecycleState.paused ||
          state == AppLifecycleState.detached,
    );
  }

  @override
  void dispose() {
    _windowSaveTimer?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    widget.services.reader.removeListener(_syncActiveTab);
    if (_isDesktop) windowManager.removeListener(this);
    unawaited(ActivationChannel.uninstall());
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
      builder: (context, _) {
        // Safe mode suppresses risky restoration/indexing paths, but it must
        // never discard presentation preferences such as the selected theme.
        final settings = widget.services.settings.settings;
        final systemHighContrast = WidgetsBinding
            .instance
            .platformDispatcher
            .accessibilityFeatures
            .highContrast;
        final highContrast =
            systemHighContrast ||
            settings.visualProfile == ReaderVisualProfile.electronicInk;
        return MaterialApp(
          navigatorKey: _navigatorKey,
          title: AppConfig.productName,
          locale: Locale(AppLanguageCode.fromCode(settings.languageCode).code),
          supportedLocales: const [Locale('es'), Locale('en')],
          localizationsDelegates: const [
            YamantakaLocalizations.delegate,
            GlobalMaterialLocalizations.delegate,
            GlobalWidgetsLocalizations.delegate,
            GlobalCupertinoLocalizations.delegate,
          ],
          debugShowCheckedModeBanner: false,
          theme: AppTheme.light(
            density: settings.interfaceDensity,
            highContrast: highContrast,
          ),
          darkTheme: AppTheme.dark(
            density: settings.interfaceDensity,
            highContrast: highContrast,
          ),
          themeMode: resolveAppThemeMode(
            settings,
            safeMode: widget.services.safeMode,
          ),
          themeAnimationDuration:
              settings.reduceMotion || settings.disableAnimations
              ? AppMotion.instant
              : AppMotion.standard,
          home: CallbackShortcuts(
            bindings: AppCommandRegistry.applicationShortcuts(
              openDocument: () => unawaited(_pickAndOpen()),
              overrides: settings.shortcutOverrides,
            ),
            child: Focus(
              autofocus: true,
              child: Builder(
                builder: (context) {
                  final reduceMotion =
                      settings.reduceMotion ||
                      settings.disableAnimations ||
                      MediaQuery.disableAnimationsOf(context);
                  return AnimatedSwitcher(
                    duration: reduceMotion
                        ? AppMotion.instant
                        : AppMotion.panel,
                    reverseDuration: reduceMotion
                        ? AppMotion.instant
                        : AppMotion.fast,
                    switchInCurve: Curves.easeOutCubic,
                    switchOutCurve: Curves.easeInCubic,
                    transitionBuilder: (child, animation) {
                      final scale = Tween<double>(
                        begin: 0.985,
                        end: 1,
                      ).animate(animation);
                      return FadeTransition(
                        opacity: animation,
                        child: ScaleTransition(scale: scale, child: child),
                      );
                    },
                    child: KeyedSubtree(
                      key: ValueKey(_page),
                      child: _buildPage(context),
                    ),
                  );
                },
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildPage(BuildContext context) => switch (_page) {
    _AppPage.library => LibraryScreen(
      controller: widget.services.library,
      onOpenPath: _openPath,
      onPickDocument: _pickAndOpen,
      onOpenSettings: () => setState(() => _page = _AppPage.settings),
      showRecents: widget.services.settings.settings.showRecents,
      initialReadingFilter: LibraryReadingState.values.firstWhere(
        (value) =>
            value.name ==
            widget.services.settings.settings.libraryReadingFilter,
        orElse: () => LibraryReadingState.all,
      ),
      onReadingFilterChanged: (value) => unawaited(
        widget.services.settings.update(
          widget.services.settings.settings.copyWith(
            libraryReadingFilter: value.name,
          ),
        ),
      ),
      onBulkOperation: _runBulkOperation,
      onDocumentInfo: _showDocumentInfo,
      onRelocateDocument: _relocateDocument,
      onToggleFavorite: _toggleFavorite,
      onChangeCover: _changeDocumentCover,
      onRevealDocument: (document) =>
          widget.services.revealDocument(document.path),
      onRemoveDocument: _removeDocument,
      onDocumentVisible: widget.services.thumbnails.promoteVisibleCover,
      organization: widget.services.organization,
      onGlobalSearch: _showGlobalSearch,
      onOpenPaths: _openPaths,
      onWorkspaces: _showWorkspaces,
      onImportFolder: _importFolder,
      onRecoverFolder: _recoverMissingFromFolder,
      safeMode: widget.services.safeMode,
    ),
    _AppPage.reader => _buildReaderPage(),
    _AppPage.settings => SettingsScreen(
      services: widget.services,
      onBack: () => setState(
        () => _page = widget.services.reader.status == ReaderStatus.ready
            ? _AppPage.reader
            : _AppPage.library,
      ),
    ),
  };

  Widget _buildReaderPage() => ReaderScreen(
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
    controlsAutoHide: widget.services.settings.settings.autoHideControls,
    readingDirection: widget.services.settings.settings.readingDirection,
    visualProfile: widget.services.settings.settings.visualProfile,
    initialPanelWidth: widget.services.settings.settings.readerPanelWidth,
    initialPanelSection: widget.services.settings.settings.readerPanelSection,
    initialPanelOpen: widget.services.settings.settings.readerPanelOpen,
    initialChromeVisible: widget.services.settings.settings.readerChromeVisible,
    showModeControls: widget.services.settings.settings.showReaderModeControls,
    showPanelControls:
        widget.services.settings.settings.showReaderPanelControls,
    shortcutOverrides: widget.services.settings.settings.shortcutOverrides,
    recentDocuments: widget.services.library.documents
        .take(8)
        .toList(growable: false),
    onOpenPath: _openPath,
    onPanelWidthChanged: (width) => unawaited(
      widget.services.settings.update(
        widget.services.settings.settings.copyWith(readerPanelWidth: width),
      ),
    ),
    onPanelChanged: (section) => unawaited(
      widget.services.settings.update(
        widget.services.settings.settings.copyWith(
          readerPanelOpen: section != null,
          readerPanelSection: section?.name ?? '',
        ),
      ),
    ),
    onChromeVisibilityChanged: (visible) => unawaited(
      widget.services.settings.update(
        widget.services.settings.settings.copyWith(
          readerChromeVisible: visible,
        ),
      ),
    ),
    onDocumentInfo: _showCurrentDocumentInfo,
    onExportAnnotations: _exportAnnotations,
  );

  Future<void> _pickAndOpen() async {
    final selected = await _pickPdf();
    if (selected != null) await _openPath(selected.path);
  }

  Future<void> _openPaths(List<String> paths) async {
    for (final path in paths.take(AppConfig.maxOpenTabs)) {
      await _openPath(path);
    }
    if (paths.length > AppConfig.maxOpenTabs) {
      _showNotice(
        'Se abrieron ${AppConfig.maxOpenTabs} archivos; el resto se omitió '
        'para respetar el límite de recursos.',
      );
    }
  }

  Future<void> _showGlobalSearch() async {
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    await showDialog<void>(
      context: context,
      builder: (context) => GlobalSearchDialog(
        controller: widget.services.globalSearch,
        indexController: widget.services.libraryIndex,
        onOpen: (result) => unawaited(_openGlobalSearchResult(result)),
      ),
    );
  }

  Future<void> _showWorkspaces() async {
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    final settings = widget.services.settings.settings;
    await showDialog<void>(
      context: context,
      builder: (context) => WorkspaceDialog(
        controller: widget.services.workspaces,
        panelSection: settings.readerPanelSection,
        panelWidth: settings.readerPanelWidth,
        onRestore: (workspace) => unawaited(_restoreWorkspace(workspace)),
      ),
    );
  }

  Future<void> _importFolder() async {
    final directory = await getDirectoryPath(
      confirmButtonText: 'Analizar carpeta',
    );
    if (directory == null) return;
    if (!mounted) return;
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    if (!context.mounted) return;
    unawaited(
      widget.services.folderImport.scan(directory, includeSubdirectories: true),
    );
    await showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (context) => FolderImportDialog(
        controller: widget.services.folderImport,
        directoryPath: directory,
        onImport: (watchRoot) async {
          if (watchRoot) widget.services.folderWatch.authorize(directory);
          final summary = await widget.services.folderImport.importNew();
          await widget.services.library.load();
          if (context.mounted) Navigator.pop(context);
          _showNotice(
            'Importados ${summary.imported}; '
            '${summary.duplicates} duplicados; '
            '${summary.inaccessible + summary.invalid} con error.',
          );
        },
      ),
    );
  }

  Future<void> _recoverMissingFromFolder() async {
    final directory = await getDirectoryPath(
      confirmButtonText: 'Buscar documentos',
    );
    if (directory == null) return;
    _showNotice('Buscando coincidencias locales…');
    final matches = await widget.services.documentRecovery.findMatches(
      directory,
    );
    final summary = await widget.services.documentRecovery.recoverExact(
      matches,
    );
    await widget.services.library.load();
    _showNotice(
      'Recuperados ${summary.recovered}; '
      '${summary.ambiguous} ambiguos; '
      '${summary.modified} modificados; '
      '${summary.notFound} no encontrados.',
    );
  }

  Future<void> _restoreWorkspace(WorkspaceSnapshot workspace) async {
    widget.services.workspaces.restore(workspace.id);
    final available = workspace.tabs
        .where((tab) => !tab.isMissing && File(tab.path).existsSync())
        .toList(growable: false);
    final missingCount = workspace.tabs.length - available.length;
    final active = available
        .where((tab) => tab.id == workspace.activeTabId)
        .firstOrNull;
    final target = active ?? available.firstOrNull;
    if (target != null) await _activateTab(target);
    if (missingCount > 0) {
      _showNotice(
        'Espacio restaurado; $missingCount documentos no están disponibles.',
      );
    }
  }

  Future<void> _openGlobalSearchResult(GlobalSearchResult result) async {
    final document = widget.services.documents.byId(result.documentId);
    if (document == null || document.isMissing) {
      _showNotice('El documento del resultado no está disponible.');
      return;
    }
    await _openPath(document.path);
    final pageIndex = result.pageIndex;
    if (pageIndex != null &&
        widget.services.reader.status == ReaderStatus.ready) {
      await widget.services.reader.navigateTo(
        pageIndex,
        reason: NavigationReason.searchResult,
        label: 'Resultado de búsqueda global',
      );
    }
  }

  Future<void> _exportAnnotations() async {
    final context = _navigatorKey.currentContext;
    final document = widget.services.reader.document;
    if (context == null || document == null) return;
    final format = await showDialog<_AnnotationExportFormat>(
      context: context,
      builder: (context) => SimpleDialog(
        title: const Text('Exportar anotaciones'),
        children: [
          SimpleDialogOption(
            onPressed: () =>
                Navigator.pop(context, _AnnotationExportFormat.markdown),
            child: const ListTile(
              leading: Icon(Icons.description_outlined),
              title: Text('Markdown'),
              subtitle: Text('Documento legible y ordenado por página'),
            ),
          ),
          SimpleDialogOption(
            onPressed: () =>
                Navigator.pop(context, _AnnotationExportFormat.json),
            child: const ListTile(
              leading: Icon(Icons.data_object_rounded),
              title: Text('JSON versionado'),
              subtitle: Text('Datos estructurados para futura importación'),
            ),
          ),
        ],
      ),
    );
    if (format == null) return;
    final extension = format == _AnnotationExportFormat.markdown
        ? 'md'
        : 'json';
    final location = await getSaveLocation(
      suggestedName: 'yamantaka-anotaciones.$extension',
      acceptedTypeGroups: [
        XTypeGroup(label: extension.toUpperCase(), extensions: [extension]),
      ],
    );
    if (location == null) return;
    final annotations = widget.services.annotations.annotations;
    final content = format == _AnnotationExportFormat.markdown
        ? widget.services.annotationExporter.toMarkdown(
            title: document.effectiveTitle,
            author: document.effectiveAuthor,
            progress: document.readingProgress,
            annotations: annotations,
          )
        : widget.services.annotationExporter.toVersionedJson(
            documentId: document.id,
            title: document.effectiveTitle,
            author: document.effectiveAuthor,
            progress: document.readingProgress,
            annotations: annotations,
          );
    try {
      await File(location.path).writeAsString(content, flush: true);
      _showNotice('Anotaciones exportadas.');
    } on Object {
      _showNotice('No se pudo exportar. Comprueba permisos y espacio libre.');
    }
  }

  Future<XFile?> _pickPdf() async {
    const pdfTypes = XTypeGroup(
      label: 'PDF',
      extensions: ['pdf'],
      mimeTypes: ['application/pdf'],
      uniformTypeIdentifiers: ['com.adobe.pdf'],
    );
    return openFile(acceptedTypeGroups: const [pdfTypes]);
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
      final document = widget.services.reader.document;
      if (document != null) widget.services.tabs.open(document);
      setState(() => _page = _AppPage.reader);
      unawaited(
        widget.services.settings.update(
          widget.services.settings.settings.copyWith(lastDocumentPath: path),
        ),
      );
      return;
    }
    final presentation = FailurePresentations.forFailure(
      widget.services.reader.error,
    );
    ScaffoldMessenger.of(_navigatorKey.currentContext ?? context).showSnackBar(
      SnackBar(content: Text('${presentation.title}: ${presentation.message}')),
    );
  }

  Future<void> _activateTab(DocumentTab tab) async {
    if (tab.isMissing) {
      _showNotice('Este documento guardado ya no está disponible.');
      return;
    }
    widget.services.tabs.activate(tab.id);
    await _openPath(tab.path);
  }

  void _syncActiveTab() {
    final reader = widget.services.reader;
    final preferences = reader.preferences;
    if (reader.status != ReaderStatus.ready || preferences == null) return;
    widget.services.tabs.updateActiveNavigation(
      pageIndex: reader.currentPageIndex,
      zoomLevel: preferences.zoomLevel,
      pageLayout: preferences.pageLayout.name,
      readingMode: preferences.viewMode.name,
      scrollOffset: preferences.scrollOffset,
      readingProgress: reader.pageCount == 0
          ? 0
          : (reader.currentPageIndex + 1) / reader.pageCount,
    );
  }

  Future<String?> _askForPassword({required bool wrongPassword}) async {
    final context = _navigatorKey.currentContext;
    if (context == null) return null;
    final textController = TextEditingController();
    final password = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        icon: const Icon(Icons.lock_outline_rounded),
        title: Text(wrongPassword ? 'Contraseña incorrecta' : 'PDF protegido'),
        content: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 400),
          child: TextField(
            controller: textController,
            autofocus: true,
            obscureText: true,
            decoration: InputDecoration(
              labelText: 'Contraseña',
              errorText: wrongPassword
                  ? 'La contraseña no es correcta. Inténtalo de nuevo.'
                  : null,
              helperText: 'La contraseña no se guarda.',
            ),
            onSubmitted: (value) => Navigator.pop(context, value),
          ),
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
    await widget.services.settings.update(
      widget.services.settings.settings.copyWith(
        lastDocumentPath: '',
        readerPanelOpen: false,
      ),
    );
    await widget.services.library.load();
    if (mounted) setState(() => _page = _AppPage.library);
  }

  Future<void> _restoreLastSession() async {
    if (_restorationAttempted) return;
    _restorationAttempted = true;
    if (widget.services.safeMode) return;
    final settings = widget.services.settings.settings;
    final path = settings.lastDocumentPath;
    final pathExists = path.isNotEmpty && File(path).existsSync();
    if (!shouldRestoreReaderSession(settings, pathExists: pathExists)) {
      if (path.isNotEmpty && !pathExists) {
        await widget.services.settings.update(
          settings.copyWith(lastDocumentPath: '', readerPanelOpen: false),
        );
      }
      return;
    }
    await _openPath(path);
    if (widget.services.reader.status != ReaderStatus.ready) {
      await widget.services.settings.update(
        widget.services.settings.settings.copyWith(
          lastDocumentPath: '',
          readerPanelOpen: false,
        ),
      );
    }
  }

  Future<void> _showCurrentDocumentInfo() async {
    final document = widget.services.reader.document;
    if (document == null) return;
    await _showDocumentInfo(document);
  }

  Future<void> _showDocumentInfo(LibraryDocument document) async {
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    final isCurrentDocument =
        widget.services.reader.document?.id == document.id &&
        widget.services.reader.status == ReaderStatus.ready;
    final isAvailable = !document.isMissing && File(document.path).existsSync();
    final readingState = document.readingProgress >= 1
        ? 'Terminado'
        : document.readingProgress > 0
        ? 'En curso'
        : 'Sin empezar';
    final presence = !isAvailable
        ? 'No disponible en la ruta guardada'
        : document.hasFileChanged
        ? 'Disponible; cambió desde la última apertura'
        : 'Disponible';
    final action = await showDialog<_DocumentInfoAction>(
      context: context,
      builder: (context) => AlertDialog(
        icon: const Icon(Icons.info_outline_rounded),
        title: const Text('Información del documento'),
        content: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 560),
          child: SelectionArea(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  document.effectiveTitle,
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                if (document.effectiveAuthor case final author?) ...[
                  const SizedBox(height: AppSpacing.space4),
                  Text(author),
                ],
                const SizedBox(height: AppSpacing.space16),
                _DocumentMetadataLine(
                  label: 'Archivo',
                  value: document.fileName,
                ),
                _DocumentMetadataLine(
                  label: 'Páginas',
                  value: '${document.pageCount}',
                ),
                _DocumentMetadataLine(
                  label: 'Tamaño',
                  value: _formatBytes(document.fileSize),
                ),
                _DocumentMetadataLine(
                  label: 'Última apertura',
                  value: _formatLocalDate(document.lastOpenedAt),
                ),
                _DocumentMetadataLine(
                  label: 'Progreso',
                  value:
                      '${(document.readingProgress * 100).round()} % · '
                      'página ${document.lastPageIndex + 1}',
                ),
                _DocumentMetadataLine(label: 'Estado', value: readingState),
                if (document.libraryNote case final note?)
                  _DocumentMetadataLine(label: 'Nota', value: note),
                _DocumentMetadataLine(
                  label: 'Modificado',
                  value: _formatLocalDate(document.fileModifiedAt),
                ),
                _DocumentMetadataLine(label: 'Presencia', value: presence),
                const SizedBox(height: AppSpacing.space8),
                Text('Ruta', style: Theme.of(context).textTheme.labelMedium),
                const SizedBox(height: AppSpacing.space4),
                SelectableText(
                  document.path,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
          ),
        ),
        actions: [
          TextButton.icon(
            onPressed: () =>
                Navigator.pop(context, _DocumentInfoAction.editMetadata),
            icon: const Icon(Icons.edit_note_rounded, size: 18),
            label: const Text('Editar metadata'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, _DocumentInfoAction.remove),
            child: const Text('Retirar'),
          ),
          TextButton(
            onPressed: () =>
                Navigator.pop(context, _DocumentInfoAction.resetProgress),
            child: const Text('Reiniciar progreso'),
          ),
          TextButton.icon(
            onPressed: () =>
                Navigator.pop(context, _DocumentInfoAction.copyPath),
            icon: const Icon(Icons.copy_rounded, size: 18),
            label: const Text('Copiar ruta'),
          ),
          if (isAvailable)
            TextButton.icon(
              onPressed: () =>
                  Navigator.pop(context, _DocumentInfoAction.reveal),
              icon: const Icon(Icons.folder_open_rounded, size: 18),
              label: const Text('Mostrar'),
            )
          else
            TextButton.icon(
              onPressed: () =>
                  Navigator.pop(context, _DocumentInfoAction.relocate),
              icon: const Icon(Icons.drive_file_move_outline, size: 18),
              label: const Text('Localizar'),
            ),
          FilledButton(
            onPressed: () => Navigator.pop(
              context,
              isCurrentDocument ? null : _DocumentInfoAction.open,
            ),
            child: Text(
              isCurrentDocument
                  ? 'Cerrar'
                  : document.readingProgress > 0
                  ? 'Continuar'
                  : 'Abrir',
            ),
          ),
        ],
      ),
    );
    if (!mounted || action == null) return;
    switch (action) {
      case _DocumentInfoAction.copyPath:
        await Clipboard.setData(ClipboardData(text: document.path));
        _showNotice('Ruta copiada.');
      case _DocumentInfoAction.editMetadata:
        await _editDocumentMetadata(document);
      case _DocumentInfoAction.reveal:
        await widget.services.revealDocument(document.path);
      case _DocumentInfoAction.resetProgress:
        final confirmed = await _confirmDocumentAction(
          title: 'Reiniciar progreso',
          message:
              'La página guardada volverá al inicio. El PDF y sus marcadores '
              'seguirán intactos.',
          confirmLabel: 'Reiniciar',
        );
        if (!confirmed) return;
        if (isCurrentDocument) {
          await widget.services.reader.resetProgress();
        } else {
          widget.services.documents.resetProgress(document.id);
        }
        await widget.services.library.load();
        _showNotice('Progreso reiniciado.');
      case _DocumentInfoAction.remove:
        await _removeDocument(document);
      case _DocumentInfoAction.relocate:
        await _relocateDocument(document);
      case _DocumentInfoAction.open:
        if (isAvailable) {
          await _openPath(document.path);
        } else {
          await _relocateDocument(document);
        }
    }
  }

  Future<void> _editDocumentMetadata(LibraryDocument document) async {
    final title = TextEditingController(text: document.titleOverride ?? '');
    final author = TextEditingController(text: document.authorOverride ?? '');
    final note = TextEditingController(text: document.libraryNote ?? '');
    final edited = await showDialog<MetadataEditState>(
      context: _navigatorKey.currentContext!,
      builder: (context) => AlertDialog(
        icon: const Icon(Icons.edit_note_rounded),
        title: const Text('Editar metadata local'),
        content: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 520),
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                TextField(
                  controller: title,
                  autofocus: true,
                  maxLength: 512,
                  decoration: const InputDecoration(
                    labelText: 'Título',
                    helperText: 'Vacío restaura el título original.',
                  ),
                ),
                TextField(
                  controller: author,
                  maxLength: 512,
                  decoration: const InputDecoration(labelText: 'Autor'),
                ),
                TextField(
                  controller: note,
                  maxLength: 4000,
                  minLines: 3,
                  maxLines: 6,
                  decoration: const InputDecoration(
                    labelText: 'Nota de biblioteca',
                    helperText: 'Se guarda fuera del PDF.',
                  ),
                ),
              ],
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(
              context,
              MetadataEditState(
                sourceTitle: document.sourceTitle ?? document.title,
                sourceAuthor: document.sourceAuthor ?? document.author,
                titleOverride: title.text,
                authorOverride: author.text,
                libraryNote: note.text,
              ),
            ),
            child: const Text('Guardar'),
          ),
        ],
      ),
    );
    title.dispose();
    author.dispose();
    note.dispose();
    if (!mounted || edited == null) return;
    try {
      final value = edited.normalized();
      widget.services.documents.updateLocalMetadata(
        document.id,
        title: value.titleOverride,
        author: value.authorOverride,
        note: value.libraryNote,
      );
      widget.services.libraryIndex.refreshMetadata(document.id);
      await widget.services.library.reload();
      _showNotice('Metadata local guardada.');
    } on ArgumentError catch (error) {
      _showNotice(error.message?.toString() ?? 'La metadata no es válida.');
    }
  }

  Future<void> _relocateDocument(LibraryDocument document) async {
    final selected = await _pickPdf();
    if (selected == null) return;
    try {
      final inspected = await widget.services.documentService.inspect(
        selected.path,
      );
      if (inspected.fingerprint != document.fingerprint) {
        _showNotice(
          'El PDF seleccionado no coincide con el documento ausente. '
          'No se ha modificado la biblioteca.',
        );
        return;
      }
    } on AppFailure catch (failure) {
      final presentation = FailurePresentations.forFailure(failure);
      _showNotice('${presentation.title}: ${presentation.message}');
      return;
    }
    await _openPath(selected.path);
  }

  Future<void> _toggleFavorite(LibraryDocument document) async {
    await widget.services.library.toggleFavorite(document.id);
    _showNotice(
      document.isFavorite ? 'Quitado de favoritos.' : 'Añadido a favoritos.',
    );
  }

  Future<BulkOperationSummary> _runBulkOperation(
    BulkLibraryAction action,
    Iterable<String> documentIds, {
    String? targetId,
  }) async {
    final ids = documentIds.toList(growable: false);
    final currentId = widget.services.reader.document?.id;
    final removesCurrent =
        action == BulkLibraryAction.removeFromLibrary &&
        currentId != null &&
        ids.contains(currentId) &&
        widget.services.reader.status == ReaderStatus.ready;
    if (removesCurrent) await widget.services.reader.closeDocument();
    final summary = await widget.services.bulkOperations.execute(
      action,
      ids,
      targetId: targetId,
    );
    widget.services.organization.reload();
    await widget.services.library.load();
    if (removesCurrent) {
      await widget.services.settings.update(
        widget.services.settings.settings.copyWith(
          lastDocumentPath: '',
          readerPanelOpen: false,
        ),
      );
      if (mounted) setState(() => _page = _AppPage.library);
    }
    return summary;
  }

  Future<void> _changeDocumentCover(LibraryDocument document) async {
    final context = _navigatorKey.currentContext;
    if (context == null || document.isMissing) return;
    final changed = await showDialog<bool>(
      context: context,
      barrierDismissible: false,
      builder: (context) => DocumentCoverPickerDialog(
        document: document,
        loadPreviews: () => widget.services.thumbnails.previewPages(document),
        onUsePage: (pageIndex) =>
            widget.services.thumbnails.setManualCover(document, pageIndex),
        onRestoreAutomatic: () =>
            widget.services.thumbnails.restoreAutomaticCover(document),
      ),
    );
    if (changed == true) {
      await widget.services.library.load();
      _showNotice('Portada actualizada.');
    }
  }

  Future<void> _removeDocument(LibraryDocument document) async {
    final isCurrentDocument =
        widget.services.reader.document?.id == document.id &&
        widget.services.reader.status == ReaderStatus.ready;
    final confirmed = await _confirmDocumentAction(
      title: 'Retirar de la biblioteca',
      message:
          'Se eliminarán el progreso, marcadores y sesiones locales. '
          'El archivo PDF no se borrará.',
      confirmLabel: 'Retirar',
    );
    if (!confirmed) return;
    if (isCurrentDocument) await widget.services.reader.closeDocument();
    await widget.services.thumbnails.removeForDocument(document);
    widget.services.documents.remove(document.id);
    widget.services.organization.reload();
    if (isCurrentDocument) {
      await widget.services.settings.update(
        widget.services.settings.settings.copyWith(
          lastDocumentPath: '',
          readerPanelOpen: false,
        ),
      );
    }
    await widget.services.library.load();
    if (mounted && isCurrentDocument) {
      setState(() => _page = _AppPage.library);
    }
    _showNotice('Documento retirado. El PDF no se ha borrado.');
  }

  Future<bool> _confirmDocumentAction({
    required String title,
    required String message,
    required String confirmLabel,
  }) async {
    final context = _navigatorKey.currentContext;
    if (context == null) return false;
    return await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            title: Text(title),
            content: Text(message),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context, false),
                child: const Text('Cancelar'),
              ),
              FilledButton(
                onPressed: () => Navigator.pop(context, true),
                child: Text(confirmLabel),
              ),
            ],
          ),
        ) ??
        false;
  }

  void _showNotice(String message) {
    final context = _navigatorKey.currentContext;
    if (context != null) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(message)));
    }
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

enum _DocumentInfoAction {
  editMetadata,
  copyPath,
  reveal,
  resetProgress,
  remove,
  relocate,
  open,
}

enum _AnnotationExportFormat { markdown, json }

final class _DocumentMetadataLine extends StatelessWidget {
  const _DocumentMetadataLine({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: AppSpacing.space6),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 132,
          child: Text(label, style: Theme.of(context).textTheme.labelMedium),
        ),
        Expanded(child: Text(value)),
      ],
    ),
  );
}

String _formatBytes(int bytes) {
  if (bytes < 1024) return '$bytes B';
  final kib = bytes / 1024;
  if (kib < 1024) return '${kib.toStringAsFixed(1)} KiB';
  return '${(kib / 1024).toStringAsFixed(1)} MiB';
}

String _formatLocalDate(DateTime value) {
  final local = value.toLocal();
  String two(int number) => number.toString().padLeft(2, '0');
  return '${two(local.day)}/${two(local.month)}/${local.year} '
      '${two(local.hour)}:${two(local.minute)}';
}

bool shouldRestoreReaderSession(
  AppSettings settings, {
  required bool pathExists,
  bool initialPathProvided = false,
}) {
  return !initialPathProvided &&
      settings.restoreLastSession &&
      !settings.startInLibrary &&
      settings.lastDocumentPath.isNotEmpty &&
      pathExists;
}
