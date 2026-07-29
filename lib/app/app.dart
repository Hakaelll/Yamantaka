import 'dart:async';
import 'dart:io';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:window_manager/window_manager.dart';

import '../core/errors/app_failure.dart';
import '../features/library/domain/library_document.dart';
import '../features/library/presentation/library_screen.dart';
import '../features/reader/application/reader_controller.dart';
import '../features/reader/presentation/reader_screen.dart';
import '../features/settings/presentation/settings_screen.dart';
import '../storage/repositories/app_settings_repository.dart';
import 'app_config.dart';
import 'app_services.dart';
import 'shortcuts/app_commands.dart';
import 'theme/app_theme.dart';
import 'theme/app_tokens.dart';

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
  bool _restorationAttempted = false;

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
    } else {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        unawaited(_restoreLastSession());
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
      builder: (context, _) {
        final settings = widget.services.settings.settings;
        return MaterialApp(
          navigatorKey: _navigatorKey,
          title: AppConfig.productName,
          debugShowCheckedModeBanner: false,
          theme: AppTheme.light(density: settings.interfaceDensity),
          darkTheme: AppTheme.dark(density: settings.interfaceDensity),
          themeMode: settings.darkMode ? ThemeMode.dark : ThemeMode.light,
          themeAnimationDuration:
              settings.reduceMotion || settings.disableAnimations
              ? AppMotion.instant
              : AppMotion.standard,
          home: CallbackShortcuts(
            bindings: AppCommandRegistry.applicationShortcuts(
              openDocument: () => unawaited(_pickAndOpen()),
              overrides: settings.shortcutOverrides,
            ),
            child: Focus(autofocus: true, child: Builder(builder: _buildPage)),
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
      onDocumentInfo: _showDocumentInfo,
      onRelocateDocument: _relocateDocument,
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
      visualProfile: widget.services.settings.settings.visualProfile,
      initialPanelWidth: widget.services.settings.settings.readerPanelWidth,
      initialPanelSection: widget.services.settings.settings.readerPanelSection,
      initialPanelOpen: widget.services.settings.settings.readerPanelOpen,
      initialChromeVisible:
          widget.services.settings.settings.readerChromeVisible,
      showModeControls:
          widget.services.settings.settings.showReaderModeControls,
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
    final selected = await _pickPdf();
    if (selected != null) await _openPath(selected.path);
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
      setState(() => _page = _AppPage.reader);
      unawaited(
        widget.services.settings.update(
          widget.services.settings.settings.copyWith(lastDocumentPath: path),
        ),
      );
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
                  document.title ?? document.fileName,
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                if (document.author case final author?) ...[
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
        final confirmed = await _confirmDocumentAction(
          title: 'Retirar de la biblioteca',
          message:
              'Se eliminarán el progreso, marcadores y sesiones locales. '
              'El archivo PDF no se borrará.',
          confirmLabel: 'Retirar',
        );
        if (!confirmed) return;
        if (isCurrentDocument) await widget.services.reader.closeDocument();
        widget.services.documents.remove(document.id);
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
      _showNotice(failure.userMessage);
      return;
    }
    await _openPath(selected.path);
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
  copyPath,
  reveal,
  resetProgress,
  remove,
  relocate,
  open,
}

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
