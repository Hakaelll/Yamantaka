part of 'app.dart';

final class _YamantakaAppState extends State<YamantakaApp>
    with WidgetsBindingObserver, WindowListener {
  final GlobalKey<NavigatorState> _navigatorKey = GlobalKey<NavigatorState>();
  _AppPage _page = _AppPage.library;
  Timer? _windowSaveTimer;
  Timer? _sessionSaveTimer;
  bool _restorationAttempted = false;
  bool _tabSwitcherOpen = false;
  bool _readerFocusMode = false;

  bool get _isDesktop =>
      Platform.isWindows || Platform.isLinux || Platform.isMacOS;

  YamantakaLocalizations get _strings =>
      YamantakaLocalizations.of(_navigatorKey.currentContext ?? context);

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
    if (await _offerInterruptedSessionRecovery()) return;
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
    if (state == AppLifecycleState.hidden ||
        state == AppLifecycleState.paused ||
        state == AppLifecycleState.detached) {
      _persistAtomicSession(clean: false);
    }
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
    widget.services.fileValidation.setBackgroundPaused(
      state == AppLifecycleState.hidden ||
          state == AppLifecycleState.paused ||
          state == AppLifecycleState.detached,
    );
  }

  @override
  void dispose() {
    _windowSaveTimer?.cancel();
    _sessionSaveTimer?.cancel();
    _persistAtomicSession(clean: true);
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
  void onWindowClose() {
    _persistAtomicSession(clean: true);
    unawaited(_saveWindowBounds());
  }

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
              settings.resolvedMotionPreference != AppMotionPreference.full
              ? AppMotion.instant
              : AppMotion.standard,
          home: CallbackShortcuts(
            bindings: {
              for (final choice in AppCommandRegistry.shortcutChoices)
                if (choice.token ==
                    AppCommandRegistry.resolvedTokens(
                      settings.shortcutOverrides,
                    )[AppCommandId.fullScreen])
                  choice.activator: () => unawaited(_toggleWindowFullScreen()),
              ...AppCommandRegistry.applicationShortcuts(
                openDocument: () => unawaited(_pickAndOpen()),
                overrides: settings.shortcutOverrides,
              ),
              const SingleActivator(
                LogicalKeyboardKey.tab,
                control: true,
              ): () =>
                  unawaited(_showTabSwitcher(reverse: false)),
              const SingleActivator(
                LogicalKeyboardKey.tab,
                control: true,
                shift: true,
              ): () =>
                  unawaited(_showTabSwitcher(reverse: true)),
            },
            child: Focus(
              autofocus: true,
              child: Builder(
                builder: (context) {
                  final reduceMotion =
                      settings.resolvedMotionPreference !=
                          AppMotionPreference.full ||
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
      onToggleFullScreen: () => unawaited(_toggleWindowFullScreen()),
      isFullScreen: widget.services.reader.isFullScreen,
      controller: widget.services.library,
      onOpenPath: _openPath,
      onPickDocument: _pickAndOpen,
      onOpenSettings: () => setState(() => _page = _AppPage.settings),
      showRecents: widget.services.settings.settings.showRecents,
      reduceMotion:
          widget.services.settings.settings.resolvedMotionPreference !=
          AppMotionPreference.full,
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
      onDocumentVisible: (document) {
        widget.services.thumbnails.promoteVisibleCover(document);
        widget.services.libraryIndex.prioritizeDocument(document.id);
      },
      onDocumentHidden: (document) =>
          widget.services.thumbnails.cancelOffscreen(document.id),
      organization: widget.services.organization,
      onGlobalSearch: _showGlobalSearch,
      onOpenPaths: _openPaths,
      onWorkspaces: _showWorkspaces,
      onImportFolder: _importFolder,
      onRecoverFolder: _recoverMissingFromFolder,
      safeMode: widget.services.safeMode,
      enrichment: widget.services.enrichment,
      fileValidation: widget.services.fileValidation,
      recentlyClosed: widget.services.readingSessions.recentlyClosed(),
      onRestoreClosed: (entry) => unawaited(_restoreRecentlyClosed(entry)),
      onDismissClosed: (entry) {
        widget.services.readingSessions.removeRecentlyClosed(entry.documentId);
        setState(() {});
      },
      onReread: (document) async {
        await widget.services.library.reread(document.id);
        await _openPath(document.path);
      },
      onMarkFinished: (document) async {
        await widget.services.library.markFinished(document.id);
        if (mounted) setState(() {});
      },
      readingGoal: switch (widget.services.library.continueReading) {
        final document? => widget.services.readingSessions.goalFor(document.id),
        _ => null,
      },
      onSetReadingGoal: (document) => unawaited(_showReadingGoal(document)),
      continuity: ContinuitySummary(
        openDocuments: widget.services.tabs.tabs.length,
        pinnedDocuments: widget.services.tabs.tabs
            .where((tab) => tab.isPinned)
            .length,
        pendingReadings:
            (widget.services.library.readingStateCounts[LibraryQueryReadingState
                    .all] ??
                widget.services.library.libraryDocumentCount) -
            (widget.services.library.readingStateCounts[LibraryQueryReadingState
                    .completed] ??
                0),
        workspaceCount: widget.services.workspaces.workspaces.length,
        problemCount: widget.services.recovery.problems().length,
      ),
      onOpenNext: () => unawaited(_openNextReading()),
      onResumeSession: () => unawaited(_resumeWorkspaceSession()),
      onDiscardContext: () => unawaited(_discardContinuityContext()),
      onProblems: () => unawaited(_showProblems()),
      onAnnotations: () => unawaited(_showGlobalAnnotations()),
      onAutomations: () => unawaited(_showAutomations()),
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
    _AppPage.problems => ProblemsScreen(
      problems: widget.services.recovery.problems(),
      onBack: () => setState(() => _page = _AppPage.library),
      onCheckIntegrity: () {
        final result = widget.services.recovery.integrityCheck();
        _showNotice(
          result.toLowerCase() == 'ok'
              ? _strings.get('problems.integrityOk')
              : _strings.get('problems.integrityFailed'),
        );
      },
      onDiscard: (problem) {
        widget.services.recovery.discardProblem(problem.id);
        setState(() {});
      },
      onResume: (problem) {
        setState(() => _page = _AppPage.library);
        unawaited(_resumeInterruptedAutomation(problem.id));
      },
      onRollback: (problem) {
        setState(() => _page = _AppPage.library);
        unawaited(_rollbackInterruptedAutomation(problem.id));
      },
      onReviewDuplicates: () => unawaited(_showDuplicateReview()),
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
        widget.services.settings.settings.resolvedMotionPreference !=
        AppMotionPreference.full,
    showProgress: widget.services.settings.settings.showProgress,
    controlsAutoHide: widget.services.settings.settings.autoHideControls,
    readingDirection: widget.services.settings.settings.readingDirection,
    visualProfile: widget.services.settings.settings.visualProfile,
    initialPanelWidth: widget.services.settings.settings.readerPanelWidth,
    initialPanelSection: widget.services.settings.settings.readerPanelSection,
    initialPanelOpen: widget.services.settings.settings.readerPanelOpen,
    initialChromeVisible: widget.services.settings.settings.readerChromeVisible,
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
    onPanelChanged: (section) {
      widget.services.tabs.updateActivePanel(section?.name);
      widget.services.reader.setSessionPanelSection(section?.name);
      unawaited(
        widget.services.settings.update(
          widget.services.settings.settings.copyWith(
            readerPanelOpen: section != null,
            readerPanelSection: section?.name ?? '',
          ),
        ),
      );
    },
    onChromeVisibilityChanged: (visible) => unawaited(
      widget.services.settings.update(
        widget.services.settings.settings.copyWith(
          readerChromeVisible: visible,
        ),
      ),
    ),
    onDocumentInfo: _showCurrentDocumentInfo,
    onExportAnnotations: _exportAnnotations,
    onViewportInteractionChanged: (isActive) {
      widget.services.libraryIndex.setInteractionPaused(isActive);
      widget.services.enrichment.setInteractionPaused(isActive);
    },
    onCompare: () => unawaited(_showComparison()),
    onFailureAction: _handleReaderFailure,
    initialFocusMode: _readerFocusMode,
    onFocusModeChanged: (enabled) {
      _readerFocusMode = enabled;
      widget.services.workspaces.updateFocusMode(enabled);
    },
  );

  Future<void> _handleReaderFailure(FailureAction action) async {
    final reader = widget.services.reader;
    switch (action) {
      case FailureAction.retry:
      case FailureAction.enterPassword:
        final path = reader.pendingPath;
        if (path == null || path.isEmpty) {
          await _pickAndOpen();
        } else {
          await _openPath(path);
        }
        return;
      case FailureAction.chooseAnotherFile:
        final failedId = reader.failedDocumentId;
        final failed = failedId == null
            ? null
            : widget.services.documents.byId(failedId);
        if (failed != null && reader.error?.kind == FailureKind.fileNotFound) {
          await _relocateDocument(failed);
        } else {
          await _pickAndOpen();
        }
        return;
      case FailureAction.openDiagnostics:
        final context = _navigatorKey.currentContext;
        if (context == null || !context.mounted) return;
        await showDialog<void>(
          context: context,
          builder: (_) => DiagnosticsDialog(services: widget.services),
        );
        return;
      case FailureAction.goBack:
      case FailureAction.dismiss:
        await _closeReader();
        return;
    }
  }

  Future<void> _showComparison() async {
    final context = _navigatorKey.currentContext;
    final primary = widget.services.reader.openedDocument;
    final primaryDocument = widget.services.reader.document;
    if (context == null || primary == null || primaryDocument == null) return;
    final candidates = widget.services.library.documents
        .where(
          (document) =>
              document.id != primaryDocument.id && !document.isMissing,
        )
        .take(80)
        .toList(growable: false);
    if (candidates.isEmpty) {
      _showNotice(_strings.get('comparison.noCandidates'));
      return;
    }
    final selected = await showDialog<LibraryDocument>(
      context: context,
      builder: (context) => SimpleDialog(
        title: Text(_strings.get('comparison.chooseDocument')),
        children: [
          for (final document in candidates)
            SimpleDialogOption(
              onPressed: () => Navigator.pop(context, document),
              child: ListTile(
                leading: const Icon(Icons.picture_as_pdf_outlined),
                title: Text(
                  document.effectiveTitle,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                subtitle: Text(
                  _strings.get(
                    'comparison.pages',
                    parameters: {'count': document.pageCount},
                  ),
                ),
              ),
            ),
        ],
      ),
    );
    if (selected == null || !context.mounted) return;
    await showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (context) => DocumentComparisonDialog(
        engine: widget.services.engine,
        primary: primary,
        primaryTitle: primaryDocument.effectiveTitle,
        secondaryPath: selected.path,
        secondaryTitle: selected.effectiveTitle,
        initialPage: widget.services.reader.currentPageIndex,
      ),
    );
  }

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
        _strings.get(
          'app.openLimit',
          parameters: {'count': AppConfig.maxOpenTabs},
        ),
      );
    }
  }

  Future<void> _showGlobalSearch() async {
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    widget.services.globalSearch.invalidateDerivedEntries();
    await showDialog<void>(
      context: context,
      builder: (context) => GlobalSearchDialog(
        controller: widget.services.globalSearch,
        indexController: widget.services.libraryIndex,
        onOpen: (result) => unawaited(_openGlobalSearchResult(result)),
        onOpenInNewTab: (result) =>
            unawaited(_openGlobalSearchResult(result, forceNewTab: true)),
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
        focusMode: _readerFocusMode,
        onRestore: (workspace) => unawaited(_restoreWorkspace(workspace)),
        onOpenQueued: (tab) => unawaited(_activateTab(tab)),
      ),
    );
  }

  Future<void> _showTabSwitcher({required bool reverse}) async {
    if (_page != _AppPage.reader ||
        _tabSwitcherOpen ||
        widget.services.tabs.tabs.length < 2) {
      return;
    }
    final primaryContext = FocusManager.instance.primaryFocus?.context;
    if (primaryContext?.findAncestorWidgetOfExactType<EditableText>() != null) {
      return;
    }
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    _tabSwitcherOpen = true;
    final selectedId = await showDialog<String>(
      context: context,
      barrierColor: Colors.black26,
      builder: (context) => DocumentTabSwitcherDialog(
        tabs: widget.services.tabs.tabs,
        activeTabId: widget.services.tabs.activeTabId,
        reverse: reverse,
      ),
    );
    _tabSwitcherOpen = false;
    if (!mounted || selectedId == null) return;
    final tab = widget.services.tabs.tabs
        .where((candidate) => candidate.id == selectedId)
        .firstOrNull;
    if (tab != null && tab.id != widget.services.tabs.activeTabId) {
      await _activateTab(tab);
    }
  }

  Future<void> _importFolder() async {
    final directory = await getDirectoryPath(
      confirmButtonText: _strings.get('library.analyzeFolder'),
    );
    if (directory == null) return;
    if (!mounted) return;
    final context = _navigatorKey.currentContext;
    if (context == null || !context.mounted) return;
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
        onOpenDuplicate: (candidate) async {
          final existing = widget.services.folderImport.duplicateDocumentFor(
            candidate,
          );
          if (existing != null) await _openPath(existing.path);
        },
        onImport: (watchRoot) async {
          if (watchRoot) widget.services.folderWatch.authorize(directory);
          final summary = await widget.services.folderImport.importNew();
          await widget.services.library.load();
          _showNotice(
            _strings.get(
              'library.importSummary',
              parameters: {
                'imported': summary.imported,
                'duplicates': summary.duplicates,
                'failed': summary.inaccessible + summary.invalid,
              },
            ),
          );
        },
      ),
    );
  }

  Future<void> _recoverMissingFromFolder() async {
    final directory = await getDirectoryPath(
      confirmButtonText: _strings.get('library.searchDocuments'),
    );
    if (directory == null) return;
    _showNotice(_strings.get('library.searchingLocalMatches'));
    final matches = await widget.services.documentRecovery.findMatches(
      directory,
    );
    final summary = await widget.services.documentRecovery.recoverExact(
      matches,
    );
    await widget.services.library.load();
    _showNotice(
      _strings.get(
        'library.recoverySummary',
        parameters: {
          'recovered': summary.recovered,
          'ambiguous': summary.ambiguous,
          'modified': summary.modified,
          'notFound': summary.notFound,
        },
      ),
    );
  }

  Future<void> _restoreWorkspace(
    WorkspaceSnapshot workspace, {
    bool automatic = false,
    String? expectedDocumentId,
    String? expectedFingerprint,
    int? expectedPageIndex,
    int? expectedPageCount,
  }) async {
    _readerFocusMode = workspace.focusMode;
    widget.services.workspaces.restore(workspace.id);
    final available = workspace.tabs
        .where(
          (tab) =>
              !tab.isMissing &&
              File(tab.path).existsSync() &&
              (!automatic ||
                  !widget.services.documents.automaticRestoreBlocked(
                    tab.documentId,
                  )),
        )
        .toList(growable: false);
    final missingCount = workspace.tabs.length - available.length;
    final active = available
        .where((tab) => tab.id == workspace.activeTabId)
        .firstOrNull;
    final target = active ?? available.firstOrNull;
    if (target != null) {
      if (automatic &&
          expectedDocumentId == target.documentId &&
          !await _matchesRecoveryIdentity(
            target.path,
            expectedFingerprint: expectedFingerprint,
            expectedPageCount: expectedPageCount,
          )) {
        _showNotice(_strings.get('recovery.sessionUnavailable'));
      } else {
        await _activateTab(target);
        await _restoreRecoveredPage(expectedDocumentId, expectedPageIndex);
      }
    }
    if (missingCount > 0) {
      _showNotice(
        _strings.get(
          'workspace.restoredMissing',
          parameters: {'count': missingCount},
        ),
      );
    }
  }

  Future<void> _openNextReading() async {
    final queued = widget.services.workspaces.openNext();
    if (queued != null && !queued.isMissing && File(queued.path).existsSync()) {
      await _activateTab(queued);
      return;
    }
    final next = widget.services.library.documents
        .where(
          (document) => !document.isMissing && document.readingProgress < 1,
        )
        .firstOrNull;
    if (next != null) {
      await _openPath(next.path);
      return;
    }
    _showNotice(_strings.get('continuity.noNext'));
  }

  Future<void> _resumeWorkspaceSession() async {
    final workspaces = widget.services.workspaces.workspaces;
    final activeId = widget.services.workspaces.activeWorkspaceId;
    final target =
        workspaces.where((workspace) => workspace.id == activeId).firstOrNull ??
        workspaces.where((workspace) => workspace.isDefault).firstOrNull ??
        workspaces.firstOrNull;
    if (target == null) {
      final tab =
          widget.services.tabs.activeTab ??
          widget.services.tabs.tabs.firstOrNull;
      if (tab != null) {
        await _activateTab(tab);
      } else {
        _showNotice(_strings.get('continuity.noSession'));
      }
      return;
    }
    await _restoreWorkspace(target);
  }

  Future<void> _discardContinuityContext() async {
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(_strings.get('continuity.discardTitle')),
        content: Text(_strings.get('continuity.discardBody')),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(_strings.get('action.cancel')),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(_strings.get('continuity.discard')),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    final closed = widget.services.readingSessions.recentlyClosed(limit: 50);
    for (final entry in closed) {
      widget.services.readingSessions.removeRecentlyClosed(entry.documentId);
    }
    widget.services.recovery.discardSession();
    if (mounted) setState(() {});
  }

  Future<void> _showProblems() async {
    if (!mounted) return;
    setState(() => _page = _AppPage.problems);
  }

  Future<void> _resumeInterruptedAutomation(String journalId) async {
    final journal = widget.services.automationRepository.journal(journalId);
    final ruleId = journal?.summary['ruleId'] as String?;
    final rule = widget.services.automationRepository
        .all()
        .where((item) => item.id == ruleId)
        .firstOrNull;
    if (journal == null ||
        rule == null ||
        journal.resumeCapability == JournalResumeCapability.none ||
        journal.resumeCapability == JournalResumeCapability.rollback ||
        journal.compatibilityHash !=
            LocalAutomationEngine.compatibilityHash(rule)) {
      _showNotice(_strings.get('problems.resumeUnavailable'));
      return;
    }
    final matchContext = AutomationMatchContext(
      tagIds: widget.services.organization.tagMemberships,
      collectionIds: widget.services.organization.collectionMemberships,
      duplicateFingerprints: widget.services.documents
          .exactDuplicateFingerprints(),
    );
    final simulation = await widget.services.automations.simulateAsync(
      rule,
      widget.services.documents.scan(),
      context: matchContext,
    );
    if (!mounted) return;
    final context = _navigatorKey.currentContext;
    if (context == null || !context.mounted) return;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(_strings.get('problems.resumeTitle')),
        content: Text(
          _strings.get(
            'problems.resumeBody',
            parameters: {'count': simulation.affected},
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(_strings.get('action.cancel')),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(_strings.get('problems.resume')),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted || !context.mounted) return;
    final cancellation = AutomationCancellationToken();
    final operation = widget.services.automations.apply(
      rule: rule,
      documents: widget.services.documents.scan(),
      context: matchContext,
      cancellationToken: cancellation,
      applyAction: _applyAutomationAction,
      rollbackAction: _undoAutomationAction,
    );
    void closeProgress() {
      if (!context.mounted) return;
      final navigator = Navigator.of(context);
      if (navigator.canPop()) navigator.pop();
    }

    unawaited(
      operation.then<void>(
        (_) {
          closeProgress();
        },
        onError: (Object _, StackTrace _) {
          closeProgress();
        },
      ),
    );
    await showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (context) => AppDialogFrame(
        icon: Icons.sync_rounded,
        title: _strings.get('problems.resuming'),
        description: _strings.get('automation.localOnly'),
        maxWidth: AppSizes.dialogCompactWidth,
        actions: const [],
        content: AppOperationStatus(
          label: _strings.get('automation.title'),
          phase: _strings.get('problems.resuming'),
          onCancel: cancellation.cancel,
          cancelLabel: _strings.get('action.cancel'),
        ),
      ),
    );
    late final AutomationApplyResult result;
    try {
      result = await operation;
    } on Object {
      _showNotice(_strings.get('problems.resumeFailed'));
      return;
    }
    widget.services.recovery.discardProblem(journalId);
    widget.services.organization.reload();
    widget.services.workspaces.reload();
    await widget.services.library.reload();
    _showNotice(
      _strings.get(
        'problems.resumeResult',
        parameters: {'completed': result.completed, 'total': result.total},
      ),
    );
    if (mounted) setState(() {});
  }

  Future<void> _rollbackInterruptedAutomation(String journalId) async {
    try {
      final rolledBack = await widget.services.automations.undo(
        journalId,
        _undoAutomationAction,
      );
      if (!rolledBack) {
        _showNotice(_strings.get('problems.rollbackUnavailable'));
        return;
      }
      widget.services.organization.reload();
      widget.services.workspaces.reload();
      await widget.services.library.reload();
      _showNotice(_strings.get('problems.rollbackComplete'));
      if (mounted) setState(() {});
    } on Object {
      _showNotice(_strings.get('problems.rollbackFailed'));
    }
  }

  Future<void> _showDuplicateReview() async {
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    final candidates = widget.services.duplicateResolution.candidates();
    final result = await showDialog<DuplicateReviewResult>(
      context: context,
      builder: (context) => DuplicateReviewDialog(
        candidates: candidates,
        impactFor: (candidate) => widget.services.duplicateResolution
            .previewImpact(candidate.existing.id, candidate.incoming.id),
      ),
    );
    if (result == null || !mounted || !context.mounted) return;
    if (result.action == DuplicateResolutionAction.keepBoth) {
      _showNotice(_strings.get('duplicates.keptBoth'));
      return;
    }
    final activeId = widget.services.reader.document?.id;
    if (activeId == result.candidate.existing.id ||
        activeId == result.candidate.incoming.id) {
      await widget.services.reader.closeDocument();
    }
    try {
      final merged = await widget.services.duplicateResolution.resolve(
        existingId: result.candidate.existing.id,
        incomingId: result.candidate.incoming.id,
        action: result.action,
        confirmed: true,
        conflictChoices: result.conflictChoices,
      );
      widget.services.organization.reload();
      widget.services.workspaces.reload();
      await widget.services.library.reload();
      if (!context.mounted) return;
      final messenger = ScaffoldMessenger.of(context);
      messenger.showSnackBar(
        SnackBar(
          content: Text(
            _strings.get(
              'duplicates.merged',
              parameters: {'count': merged.conflictFields.length},
            ),
          ),
          action: merged.historyId == null
              ? null
              : SnackBarAction(
                  label: _strings.get('action.undo'),
                  onPressed: () =>
                      unawaited(_undoDuplicateMerge(merged.historyId!)),
                ),
        ),
      );
    } on Object {
      _showNotice(_strings.get('duplicates.mergeFailed'));
    }
  }

  Future<void> _undoDuplicateMerge(String historyId) async {
    try {
      final undone = await widget.services.duplicateResolution.undoMerge(
        historyId,
      );
      if (!undone) {
        _showNotice(_strings.get('duplicates.undoUnavailable'));
        return;
      }
      widget.services.organization.reload();
      widget.services.workspaces.reload();
      await widget.services.library.reload();
      _showNotice(_strings.get('duplicates.undoComplete'));
    } on Object {
      _showNotice(_strings.get('duplicates.undoFailed'));
    }
  }

  Future<void> _showGlobalAnnotations() async {
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    await showDialog<void>(
      context: context,
      builder: (dialogContext) => GlobalAnnotationsDialog(
        repository: widget.services.annotationRepository,
        documentTitle: (documentId) =>
            widget.services.documents.byId(documentId)?.effectiveTitle ??
            _strings.get('annotations.unknownDocument'),
        onOpen: (annotation) {
          Navigator.pop(dialogContext);
          unawaited(_openAnnotation(annotation));
        },
        onExport: _exportGlobalAnnotations,
      ),
    );
  }

  Future<void> _exportGlobalAnnotations(List<PdfAnnotation> annotations) async {
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    final format = await showDialog<_AnnotationExportFormat>(
      context: context,
      builder: (context) => SimpleDialog(
        title: Text(_strings.get('reader.exportAnnotations')),
        children: [
          SimpleDialogOption(
            onPressed: () =>
                Navigator.pop(context, _AnnotationExportFormat.markdown),
            child: const ListTile(
              leading: Icon(Icons.description_outlined),
              title: Text('Markdown'),
            ),
          ),
          SimpleDialogOption(
            onPressed: () =>
                Navigator.pop(context, _AnnotationExportFormat.json),
            child: const ListTile(
              leading: Icon(Icons.data_object_rounded),
              title: Text('JSON'),
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
      suggestedName: 'yamantaka-revision-anotaciones.$extension',
      acceptedTypeGroups: [
        XTypeGroup(label: extension.toUpperCase(), extensions: [extension]),
      ],
    );
    if (location == null) return;
    final byDocument = <String, List<PdfAnnotation>>{};
    for (final annotation in annotations.take(80)) {
      (byDocument[annotation.documentId] ??= []).add(annotation);
    }
    final content = switch (format) {
      _AnnotationExportFormat.markdown =>
        byDocument.entries
            .map((entry) {
              final document = widget.services.documents.byId(entry.key);
              return widget.services.annotationExporter.toMarkdown(
                title:
                    document?.effectiveTitle ??
                    _strings.get('annotations.unknownDocument'),
                author: document?.effectiveAuthor,
                progress: document?.readingProgress ?? 0,
                annotations: entry.value,
              );
            })
            .join('\n---\n\n'),
      _AnnotationExportFormat.json => const JsonEncoder.withIndent(' ').convert(
        {
          'format': 'yamantaka-global-annotations',
          'version': 1,
          'exportedAt': DateTime.now().toUtc().toIso8601String(),
          'documents': [
            for (final entry in byDocument.entries)
              jsonDecode(
                widget.services.annotationExporter.toVersionedJson(
                  documentId: entry.key,
                  title:
                      widget.services.documents
                          .byId(entry.key)
                          ?.effectiveTitle ??
                      _strings.get('annotations.unknownDocument'),
                  author: widget.services.documents
                      .byId(entry.key)
                      ?.effectiveAuthor,
                  progress:
                      widget.services.documents
                          .byId(entry.key)
                          ?.readingProgress ??
                      0,
                  annotations: entry.value,
                ),
              ),
          ],
        },
      ),
    };
    try {
      await File(location.path).writeAsString(content, flush: true);
      _showNotice(_strings.get('annotations.exported'));
    } on Object {
      _showNotice(_strings.get('annotations.exportFailed'));
    }
  }

  Future<void> _openAnnotation(PdfAnnotation annotation) async {
    final document = widget.services.documents.byId(annotation.documentId);
    if (document == null || document.isMissing) {
      _showNotice(_strings.get('search.resultUnavailable'));
      return;
    }
    await _openPath(document.path);
    if (widget.services.reader.status == ReaderStatus.ready) {
      await widget.services.reader.navigateTo(
        annotation.pageIndex,
        reason: NavigationReason.annotation,
        label: _strings.get('annotations.globalTitle'),
      );
    }
  }

  Future<void> _showAutomations() async {
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    final duplicateFingerprints = widget.services.documents
        .exactDuplicateFingerprints();
    await showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (context) => AutomationManagerDialog(
        repository: widget.services.automationRepository,
        engine: widget.services.automations,
        documents: widget.services.documents.scan(),
        matchContext: AutomationMatchContext(
          tagIds: widget.services.organization.tagMemberships,
          collectionIds: widget.services.organization.collectionMemberships,
          duplicateFingerprints: duplicateFingerprints,
        ),
        applyAction: _applyAutomationAction,
        applyUndo: _undoAutomationAction,
      ),
    );
    widget.services.organization.reload();
    widget.services.workspaces.reload();
    await widget.services.library.reload();
    if (mounted) setState(() {});
  }

  Future<Map<String, Object?>> _applyAutomationAction(
    LibraryDocument document,
    AutomationAction action,
  ) async {
    switch (action.kind) {
      case AutomationActionKind.addTag:
        final tag =
            widget.services.organization.tags
                .where(
                  (item) =>
                      item.id == action.value ||
                      item.name.toLowerCase() == action.value.toLowerCase(),
                )
                .firstOrNull ??
            widget.services.organization.ensureTag(action.value);
        final alreadyAssigned =
            widget.services.organization.tagMemberships[tag.id]?.contains(
              document.id,
            ) ??
            false;
        widget.services.organization.assignTagSilently(tag.id, document.id);
        return {
          'kind': 'removeTag',
          'tagId': tag.id,
          'documentId': document.id,
          'changed': !alreadyAssigned,
        };
      case AutomationActionKind.addCollection:
        final collection =
            widget.services.organization.collections
                .where(
                  (item) =>
                      item.id == action.value ||
                      item.name.toLowerCase() == action.value.toLowerCase(),
                )
                .firstOrNull ??
            widget.services.organization.createCollection(action.value);
        final alreadyAssigned =
            widget.services.organization.collectionMemberships[collection.id]
                ?.contains(document.id) ??
            false;
        widget.services.organization.addDocumentSilently(
          collection.id,
          document.id,
        );
        return {
          'kind': 'removeCollection',
          'collectionId': collection.id,
          'documentId': document.id,
          'changed': !alreadyAssigned,
        };
      case AutomationActionKind.addWorkspace:
      case AutomationActionKind.addReadingQueue:
        final workspace = widget.services.workspaces.workspaces
            .where(
              (item) =>
                  item.id == action.value ||
                  item.name.toLowerCase() == action.value.toLowerCase(),
            )
            .firstOrNull;
        if (workspace == null) throw StateError('workspace_not_found');
        final changed = widget.services.workspaces.addDocumentToQueue(
          workspace.id,
          document,
        );
        return {
          'kind': 'removeQueue',
          'workspaceId': workspace.id,
          'documentId': document.id,
          'changed': changed,
        };
      case AutomationActionKind.setReadingState:
        final state = action.value.trim().toLowerCase();
        if (state == 'finished' || state == 'terminado') {
          widget.services.documents.markFinished(document.id);
        } else if (state == 'pending' ||
            state == 'pendiente' ||
            state == 'notstarted') {
          widget.services.documents.resetProgress(document.id);
        } else {
          throw StateError('unsupported_reading_state');
        }
        return {
          'kind': 'restoreProgress',
          'documentId': document.id,
          'pageIndex': document.lastPageIndex,
          'pageCount': document.pageCount,
          'viewMode': document.preferredViewMode.name,
          'theme': document.preferredTheme.name,
        };
      case AutomationActionKind.setPriority:
        final oldValue = document.isFavorite;
        final value = !{
          '0',
          'false',
          'normal',
        }.contains(action.value.trim().toLowerCase());
        widget.services.documents.setFavorite(document.id, value);
        return {
          'kind': 'restoreFavorite',
          'documentId': document.id,
          'value': oldValue,
        };
    }
  }

  Future<void> _undoAutomationAction(Map<String, Object?> undo) async {
    if (undo['changed'] == false) return;
    switch (undo['kind']) {
      case 'removeTag':
        widget.services.organization.removeTagSilently(
          undo['tagId']! as String,
          undo['documentId']! as String,
        );
      case 'removeCollection':
        widget.services.organization.removeDocumentSilently(
          undo['collectionId']! as String,
          undo['documentId']! as String,
        );
      case 'removeQueue':
        widget.services.workspaces.removeDocumentFromQueue(
          undo['workspaceId']! as String,
          undo['documentId']! as String,
        );
      case 'restoreProgress':
        widget.services.documents.updateProgress(
          documentId: undo['documentId']! as String,
          pageIndex: undo['pageIndex']! as int,
          pageCount: undo['pageCount']! as int,
          viewMode: ReaderMode.values.firstWhere(
            (item) => item.name == undo['viewMode'],
          ),
          theme: ReaderTheme.values.firstWhere(
            (item) => item.name == undo['theme'],
          ),
        );
      case 'restoreFavorite':
        widget.services.documents.setFavorite(
          undo['documentId']! as String,
          undo['value']! as bool,
        );
    }
  }

  Future<void> _openGlobalSearchResult(
    GlobalSearchResult result, {
    bool forceNewTab = false,
  }) async {
    final document = widget.services.documents.byId(result.documentId);
    if (document == null || document.isMissing) {
      _showNotice(_strings.get('search.resultUnavailable'));
      return;
    }
    await _openPath(document.path, forceNewTab: forceNewTab);
    final pageIndex = result.pageIndex;
    if (pageIndex != null &&
        widget.services.reader.status == ReaderStatus.ready) {
      await widget.services.reader.navigateTo(
        pageIndex,
        reason: NavigationReason.searchResult,
        label: _strings.get('search.globalResult'),
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
        title: Text(_strings.get('reader.exportAnnotations')),
        children: [
          SimpleDialogOption(
            onPressed: () =>
                Navigator.pop(context, _AnnotationExportFormat.markdown),
            child: ListTile(
              leading: const Icon(Icons.description_outlined),
              title: const Text('Markdown'),
              subtitle: Text(_strings.get('annotation.exportMarkdown')),
            ),
          ),
          SimpleDialogOption(
            onPressed: () =>
                Navigator.pop(context, _AnnotationExportFormat.json),
            child: ListTile(
              leading: const Icon(Icons.data_object_rounded),
              title: Text(_strings.get('annotation.versionedJson')),
              subtitle: Text(_strings.get('annotation.exportJson')),
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

  Future<void> _openPath(
    String path, {
    bool forceNewTab = false,
    bool registerTab = true,
  }) async {
    widget.services.startup.startIfAbsent(StartupPhase.firstPdf);
    var passwordAttempt = 0;
    var opened = await widget.services.reader.openPath(path);
    if (!mounted) return;
    while (!opened && passwordAttempt < AppConfig.maxPdfPasswordAttempts) {
      final failure = widget.services.reader.error;
      if (failure == null ||
          (failure.kind != FailureKind.passwordRequired &&
              failure.kind != FailureKind.wrongPassword)) {
        break;
      }
      final password = await _askForPassword(
        wrongPassword: failure.kind == FailureKind.wrongPassword,
      );
      if (password == null || password.isEmpty) break;
      passwordAttempt++;
      opened = await widget.services.reader.openPath(
        path,
        password: password,
        passwordAttempt: passwordAttempt,
      );
      if (!mounted) return;
    }
    if (!opened &&
        passwordAttempt >= AppConfig.maxPdfPasswordAttempts &&
        widget.services.reader.error?.kind == FailureKind.wrongPassword) {
      _showNotice(_strings.get('reader.passwordAttemptsExceeded'));
    }
    if (!mounted) return;
    if (opened) {
      widget.services.startup.complete(StartupPhase.firstPdf);
      final document = widget.services.reader.document;
      if (document != null) {
        widget.services.readingSessions.removeRecentlyClosed(document.id);
      }
      if (document != null && registerTab) {
        widget.services.tabs.open(document, forceNew: forceNewTab);
      }
      widget.services.tabs.updateActivePanel(
        widget.services.settings.settings.readerPanelOpen
            ? widget.services.settings.settings.readerPanelSection
            : null,
      );
      _syncActiveTab();
      widget.services.libraryIndex.prioritizeSecondary([
        ...widget.services.tabs.tabs
            .where((tab) => tab.isPinned)
            .map((tab) => tab.documentId),
        ...widget.services.readingSessions.recentlyClosed().map(
          (entry) => entry.documentId,
        ),
      ]);
      await widget.services.reader.retainWarmDocuments(
        widget.services.tabs.tabs
            .where((tab) => tab.resourceState == TabResourceState.warm)
            .map((tab) => tab.documentId),
      );
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
      strings: YamantakaLocalizations.of(context),
    );
    ScaffoldMessenger.of(_navigatorKey.currentContext ?? context).showSnackBar(
      SnackBar(content: Text('${presentation.title}: ${presentation.message}')),
    );
  }

  Future<void> _activateTab(DocumentTab tab) async {
    if (tab.isMissing) {
      _showNotice(_strings.get('reader.savedDocumentUnavailable'));
      return;
    }
    widget.services.tabs.activate(tab.id);
    await widget.services.settings.update(
      widget.services.settings.settings.copyWith(
        readerPanelOpen: tab.panelSection != null,
        readerPanelSection: tab.panelSection ?? '',
      ),
    );
    await _openPath(tab.path, registerTab: false);
    if (widget.services.reader.status != ReaderStatus.ready) return;
    widget.services.reader.restoreNavigationSnapshot(
      pageIndex: tab.pageIndex,
      zoomLevel: tab.zoomLevel,
      pageLayout: PageLayoutMode.values.firstWhere(
        (value) => value.name == tab.pageLayout,
        orElse: () => PageLayoutMode.single,
      ),
      readingMode: ReaderMode.values.firstWhere(
        (value) => value.name == tab.readingMode,
        orElse: () => ReaderMode.document,
      ),
      scrollOffset: tab.scrollOffset,
      readingTheme: ReaderTheme.values.firstWhere(
        (value) => value.name == tab.readingTheme,
        orElse: () => ReaderTheme.ivory,
      ),
    );
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
      readingTheme: preferences.readingTheme.name,
      readingProgress: reader.pageCount == 0
          ? 0
          : (reader.currentPageIndex + 1) / reader.pageCount,
    );
    final document = reader.document;
    if (document != null) {
      widget.services.libraryIndex.prioritizeDocument(
        document.id,
        pageIndex: reader.currentPageIndex,
      );
    }
    _scheduleAtomicSessionSave();
  }

  void _scheduleAtomicSessionSave() {
    _sessionSaveTimer?.cancel();
    _sessionSaveTimer = Timer(
      const Duration(milliseconds: 500),
      () => _persistAtomicSession(clean: false),
    );
  }

  void _persistAtomicSession({required bool clean}) {
    if (widget.services.safeMode) return;
    final reader = widget.services.reader;
    final document = reader.document;
    widget.services.recovery.saveSession(
      AtomicSessionSnapshot(
        workspaceId: widget.services.workspaces.activeWorkspaceId,
        activeDocumentId:
            document?.id ?? widget.services.tabs.activeTab?.documentId,
        activeDocumentFingerprint: document?.fingerprint,
        pageIndex: document == null ? null : reader.currentPageIndex,
        pageCount: document?.pageCount,
        quarantinedDocumentId: reader.failedDocumentId,
        lifecycleState: reader.lifecycle.state.name,
        savedAt: DateTime.now().toUtc(),
        clean: clean,
      ),
    );
  }

  Future<bool> _offerInterruptedSessionRecovery() async {
    if (!mounted || widget.services.safeMode) return false;
    final snapshot = widget.services.recovery.session();
    if (snapshot == null || snapshot.clean) return false;
    final action = await showDialog<SessionRecoveryChoice>(
      context: context,
      barrierDismissible: false,
      builder: (context) => const SessionRecoveryDialog(),
    );
    if (action == SessionRecoveryChoice.startClean) {
      widget.services.recovery.discardSession();
      _restorationAttempted = true;
      return true;
    }
    if (action != SessionRecoveryChoice.restore) return false;
    _restorationAttempted = true;
    await _restoreAtomicSnapshot(snapshot);
    return true;
  }

  Future<void> _restoreAtomicSnapshot(AtomicSessionSnapshot snapshot) async {
    if (snapshot.activeDocumentId != null &&
        snapshot.activeDocumentId == snapshot.quarantinedDocumentId) {
      _showNotice(_strings.get('recovery.sessionUnavailable'));
      return;
    }
    final workspace = widget.services.workspaces.workspaces
        .where((item) => item.id == snapshot.workspaceId)
        .firstOrNull;
    if (workspace != null) {
      await _restoreWorkspace(
        workspace,
        automatic: true,
        expectedDocumentId: snapshot.activeDocumentId,
        expectedFingerprint: snapshot.activeDocumentFingerprint,
        expectedPageIndex: snapshot.pageIndex,
        expectedPageCount: snapshot.pageCount,
      );
      return;
    }
    final document = widget.services.documents.byId(
      snapshot.activeDocumentId ?? '',
    );
    if (document != null &&
        document.id != snapshot.quarantinedDocumentId &&
        !widget.services.documents.automaticRestoreBlocked(document.id) &&
        !document.isMissing &&
        File(document.path).existsSync() &&
        await _matchesRecoveryIdentity(
          document.path,
          expectedFingerprint: snapshot.activeDocumentFingerprint,
          expectedPageCount: snapshot.pageCount,
        )) {
      await _openPath(document.path);
      await _restoreRecoveredPage(document.id, snapshot.pageIndex);
      return;
    }
    _showNotice(_strings.get('recovery.sessionUnavailable'));
  }

  Future<bool> _matchesRecoveryIdentity(
    String path, {
    String? expectedFingerprint,
    int? expectedPageCount,
  }) async {
    if (expectedFingerprint == null && expectedPageCount == null) return true;
    try {
      final inspected = await widget.services.documentService.inspect(path);
      if (expectedFingerprint != null &&
          inspected.fingerprint != expectedFingerprint) {
        return false;
      }
      final registered = widget.services.documents.byNormalizedPath(
        inspected.normalizedPath,
      );
      return expectedPageCount == null ||
          registered == null ||
          registered.pageCount == expectedPageCount;
    } on Object {
      return false;
    }
  }

  Future<void> _restoreRecoveredPage(
    String? expectedDocumentId,
    int? pageIndex,
  ) async {
    final reader = widget.services.reader;
    if (pageIndex == null ||
        reader.document?.id != expectedDocumentId ||
        reader.pageCount == 0) {
      return;
    }
    await reader.navigateTo(
      pageIndex,
      reason: NavigationReason.history,
      label: _strings.get('recovery.restoreSession'),
    );
  }

  Future<String?> _askForPassword({required bool wrongPassword}) async {
    final context = _navigatorKey.currentContext;
    if (context == null) return null;
    final password = await showDialog<String>(
      context: context,
      builder: (context) => PasswordRequestDialog(wrongPassword: wrongPassword),
    );
    return password;
  }

  Future<void> _closeReader() async {
    _syncActiveTab();
    final tab = widget.services.tabs.activeTab;
    if (tab != null) {
      widget.services.readingSessions.rememberClosed(
        RecentlyClosedDocument(
          documentId: tab.documentId,
          path: tab.path,
          title: tab.title,
          pageIndex: tab.pageIndex,
          zoomLevel: tab.zoomLevel,
          pageLayout: tab.pageLayout,
          readingMode: tab.readingMode,
          readingTheme: tab.readingTheme,
          scrollOffset: tab.scrollOffset,
          panelSection: tab.panelSection,
          closedAt: DateTime.now().toUtc(),
          isMissing: tab.isMissing,
          isModified: tab.isExternallyModified,
        ),
      );
      widget.services.tabs.close(tab.id);
    }
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

  Future<void> _restoreRecentlyClosed(RecentlyClosedDocument entry) async {
    if (entry.isMissing || !File(entry.path).existsSync()) {
      _showNotice(_strings.get('reader.savedDocumentUnavailable'));
      return;
    }
    await _openPath(entry.path);
    if (widget.services.reader.status != ReaderStatus.ready) return;
    widget.services.reader.restoreNavigationSnapshot(
      pageIndex: entry.pageIndex,
      zoomLevel: entry.zoomLevel,
      pageLayout: PageLayoutMode.values.firstWhere(
        (value) => value.name == entry.pageLayout,
        orElse: () => PageLayoutMode.single,
      ),
      readingMode: ReaderMode.values.firstWhere(
        (value) => value.name == entry.readingMode,
        orElse: () => ReaderMode.document,
      ),
      scrollOffset: entry.scrollOffset,
      readingTheme: ReaderTheme.values.firstWhere(
        (value) => value.name == entry.readingTheme,
        orElse: () => ReaderTheme.ivory,
      ),
    );
    widget.services.tabs.updateActivePanel(entry.panelSection);
    widget.services.reader.setSessionPanelSection(entry.panelSection);
    await widget.services.settings.update(
      widget.services.settings.settings.copyWith(
        readerPanelOpen: entry.panelSection != null,
        readerPanelSection: entry.panelSection ?? '',
      ),
    );
  }

  Future<void> _showReadingGoal(LibraryDocument document) async {
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    final current = widget.services.readingSessions.goalFor(document.id);
    final pages = TextEditingController(text: current?.pages?.toString() ?? '');
    final minutes = TextEditingController(
      text: current?.minutes?.toString() ?? '',
    );
    final goal = await showDialog<ReadingGoal?>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(_strings.get('library.localGoal')),
        content: SizedBox(
          width: 380,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: pages,
                keyboardType: TextInputType.number,
                inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                decoration: InputDecoration(
                  labelText: _strings.get('library.goalPages'),
                ),
              ),
              const SizedBox(height: AppSpacing.space12),
              TextField(
                controller: minutes,
                keyboardType: TextInputType.number,
                inputFormatters: [FilteringTextInputFormatter.digitsOnly],
                decoration: InputDecoration(
                  labelText: _strings.get('library.goalMinutes'),
                ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: Text(_strings.get('action.cancel')),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, const ReadingGoal()),
            child: Text(_strings.get('action.clear')),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(
              context,
              ReadingGoal(
                pages: int.tryParse(pages.text),
                minutes: int.tryParse(minutes.text),
              ),
            ),
            child: Text(_strings.get('action.save')),
          ),
        ],
      ),
    );
    pages.dispose();
    minutes.dispose();
    if (goal == null) return;
    widget.services.readingSessions.setGoal(document.id, goal);
    if (mounted) setState(() {});
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
        ? _strings.get('library.finished')
        : document.readingProgress > 0
        ? _strings.get('library.readingState.inProgress')
        : _strings.get('library.notStarted');
    final presence = !isAvailable
        ? _strings.get('reader.unavailableAtPath')
        : document.hasFileChanged
        ? _strings.get('reader.availableChanged')
        : _strings.get('reader.available');
    final relationCount = widget.services.relations
        .forDocument(document.id)
        .length;
    final action = await showDialog<_DocumentInfoAction>(
      context: context,
      builder: (context) => AlertDialog(
        icon: const Icon(Icons.info_outline_rounded),
        title: Text(_strings.get('command.documentInfo')),
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
                  label: _strings.get('reader.file'),
                  value: document.fileName,
                ),
                _DocumentMetadataLine(
                  label: _strings.get('reader.documentPages'),
                  value: '${document.pageCount}',
                ),
                _DocumentMetadataLine(
                  label: _strings.get('reader.documentSize'),
                  value: _formatBytes(document.fileSize),
                ),
                _DocumentMetadataLine(
                  label: _strings.get('reader.lastOpened'),
                  value: _formatLocalDate(document.lastOpenedAt),
                ),
                _DocumentMetadataLine(
                  label: _strings.get('reader.progress'),
                  value:
                      '${(document.readingProgress * 100).round()} % · '
                      '${_strings.get('reader.pageLower', parameters: {'page': document.lastPageIndex + 1})}',
                ),
                _DocumentMetadataLine(
                  label: _strings.get('library.state'),
                  value: readingState,
                ),
                if (document.libraryNote case final note?)
                  _DocumentMetadataLine(
                    label: _strings.get('reader.note'),
                    value: note,
                  ),
                if (document.topics.isNotEmpty)
                  _DocumentMetadataLine(
                    label: _strings.get('library.topics'),
                    value: document.topics
                        .map(
                          (assignment) =>
                              _strings.get('topic.${assignment.topic.name}'),
                        )
                        .join(', '),
                  ),
                if (document.enrichment?.suggestedTitle case final title?)
                  _DocumentMetadataLine(
                    label: _strings.get('reader.suggestedTitle'),
                    value: title,
                  ),
                if (document.enrichment?.suggestedAuthor case final author?)
                  _DocumentMetadataLine(
                    label: _strings.get('reader.suggestedAuthor'),
                    value: author,
                  ),
                _DocumentMetadataLine(
                  label: _strings.get('reader.documentModified'),
                  value: _formatLocalDate(document.fileModifiedAt),
                ),
                _DocumentMetadataLine(
                  label: _strings.get('reader.presence'),
                  value: presence,
                ),
                _DocumentMetadataLine(
                  label: _strings.get('relations.title'),
                  value: _strings.get(
                    'relations.count',
                    parameters: {'count': relationCount},
                  ),
                ),
                const SizedBox(height: AppSpacing.space8),
                Text(
                  _strings.get('reader.path'),
                  style: Theme.of(context).textTheme.labelMedium,
                ),
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
          if (document.enrichment?.needsReview == true) ...[
            TextButton(
              onPressed: () =>
                  Navigator.pop(context, _DocumentInfoAction.dismissSuggestion),
              child: Text(_strings.get('reader.dismissSuggestion')),
            ),
            FilledButton.tonal(
              onPressed: () =>
                  Navigator.pop(context, _DocumentInfoAction.acceptSuggestion),
              child: Text(_strings.get('reader.acceptSuggestion')),
            ),
          ],
          TextButton.icon(
            onPressed: () =>
                Navigator.pop(context, _DocumentInfoAction.editMetadata),
            icon: const Icon(Icons.edit_note_rounded, size: 18),
            label: Text(_strings.get('reader.editMetadata')),
          ),
          TextButton.icon(
            onPressed: () =>
                Navigator.pop(context, _DocumentInfoAction.relations),
            icon: const Icon(Icons.hub_outlined, size: 18),
            label: Text(_strings.get('relations.title')),
          ),
          TextButton(
            style: TextButton.styleFrom(
              foregroundColor: context.appColors.danger,
            ),
            onPressed: () => Navigator.pop(context, _DocumentInfoAction.remove),
            child: Text(_strings.get('library.remove')),
          ),
          IconButton(
            tooltip: _strings.get('library.reanalyzeMetadata'),
            onPressed: () =>
                Navigator.pop(context, _DocumentInfoAction.reanalyze),
            icon: const Icon(Icons.refresh_rounded),
          ),
          TextButton(
            onPressed: () =>
                Navigator.pop(context, _DocumentInfoAction.resetProgress),
            child: Text(_strings.get('reader.resetProgress')),
          ),
          TextButton.icon(
            onPressed: () =>
                Navigator.pop(context, _DocumentInfoAction.copyPath),
            icon: const Icon(Icons.copy_rounded, size: 18),
            label: Text(_strings.get('reader.copyPath')),
          ),
          if (isAvailable)
            TextButton.icon(
              onPressed: () =>
                  Navigator.pop(context, _DocumentInfoAction.reveal),
              icon: const Icon(Icons.folder_open_rounded, size: 18),
              label: Text(_strings.get('reader.show')),
            ),
          FilledButton(
            onPressed: () => Navigator.pop(
              context,
              isCurrentDocument
                  ? null
                  : isAvailable
                  ? _DocumentInfoAction.open
                  : _DocumentInfoAction.relocate,
            ),
            child: Text(
              isCurrentDocument
                  ? _strings.get('action.close')
                  : !isAvailable
                  ? _strings.get('library.locate')
                  : document.readingProgress > 0
                  ? _strings.get('reader.continue')
                  : _strings.get('action.open'),
            ),
          ),
        ],
      ),
    );
    if (!mounted || action == null) return;
    switch (action) {
      case _DocumentInfoAction.copyPath:
        await Clipboard.setData(ClipboardData(text: document.path));
        _showNotice(_strings.get('reader.pathCopied'));
      case _DocumentInfoAction.editMetadata:
        await _editDocumentMetadata(document);
      case _DocumentInfoAction.acceptSuggestion:
        widget.services.enrichment.acceptSuggestions(document.id);
        await widget.services.library.reload();
        _showNotice(_strings.get('reader.metadataApplied'));
      case _DocumentInfoAction.dismissSuggestion:
        widget.services.enrichment.dismissSuggestions(document.id);
        await widget.services.library.reload();
        _showNotice(_strings.get('reader.suggestionDismissed'));
      case _DocumentInfoAction.reanalyze:
        await widget.services.enrichment.analyzeDocument(document);
        await widget.services.library.reload();
        _showNotice(_strings.get('reader.analysisUpdated'));
      case _DocumentInfoAction.reveal:
        await widget.services.revealDocument(document.path);
      case _DocumentInfoAction.resetProgress:
        final confirmed = await _confirmDocumentAction(
          title: _strings.get('reader.resetProgress'),
          message: _strings.get('reader.resetProgress.description'),
          confirmLabel: _strings.get('reader.reset'),
        );
        if (!confirmed) return;
        if (isCurrentDocument) {
          await widget.services.reader.resetProgress();
        } else {
          widget.services.documents.resetProgress(document.id);
        }
        await widget.services.library.load();
        _showNotice(_strings.get('reader.progressReset'));
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
      case _DocumentInfoAction.relations:
        await _showDocumentRelations(document);
    }
  }

  Future<void> _showDocumentRelations(LibraryDocument document) async {
    final context = _navigatorKey.currentContext;
    if (context == null) return;
    await showDialog<void>(
      context: context,
      builder: (dialogContext) => DocumentRelationsDialog(
        repository: widget.services.relations,
        document: document,
        candidates: widget.services.library.documents
            .where((candidate) => candidate.id != document.id)
            .take(80)
            .toList(growable: false),
        onOpenDocument: (documentId) {
          final target = widget.services.documents.byId(documentId);
          if (target == null || target.isMissing) {
            _showNotice(_strings.get('search.resultUnavailable'));
            return;
          }
          Navigator.pop(dialogContext);
          unawaited(_openPath(target.path));
        },
      ),
    );
  }

  Future<void> _editDocumentMetadata(LibraryDocument document) async {
    final title = TextEditingController(text: document.titleOverride ?? '');
    final author = TextEditingController(text: document.authorOverride ?? '');
    final note = TextEditingController(text: document.libraryNote ?? '');
    final initialTitle = title.text;
    final initialAuthor = author.text;
    final initialNote = note.text;
    final sourceTitle = document.sourceTitle ?? document.title;
    final sourceAuthor = document.sourceAuthor ?? document.author;
    final initialTopics = document.topics
        .map((assignment) => assignment.topic)
        .toSet();
    var selectedTopics = Set<LibraryTopic>.from(initialTopics);
    final edited = await showDialog<_MetadataEditResult>(
      context: _navigatorKey.currentContext!,
      builder: (context) => StatefulBuilder(
        builder: (context, setDialogState) {
          final dirty =
              title.text != initialTitle ||
              author.text != initialAuthor ||
              note.text != initialNote ||
              !setEquals(initialTopics, selectedTopics);
          return AlertDialog(
            title: AppDialogHeader(
              icon: Icons.edit_note_rounded,
              title: _strings.get('reader.editLocalMetadata'),
              description: _strings.get('reader.metadataLocal.description'),
              trailing: dirty
                  ? AppStatusLabel(
                      icon: Icons.edit_outlined,
                      label: _strings.get('reader.unsavedChanges'),
                      tone: AppStatusTone.warning,
                    )
                  : null,
            ),
            content: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 560, maxHeight: 660),
              child: SingleChildScrollView(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      _strings.get('reader.metadataIdentity'),
                      style: Theme.of(context).textTheme.titleSmall,
                    ),
                    const SizedBox(height: AppSpacing.space12),
                    TextField(
                      controller: title,
                      autofocus: true,
                      maxLength: 512,
                      decoration: InputDecoration(
                        labelText: _strings.get('reader.documentTitle'),
                        helperText: _strings.get(
                          'reader.detectedValue',
                          parameters: {
                            'value':
                                sourceTitle ??
                                _strings.get('reader.noDetectedValue'),
                          },
                        ),
                        helperMaxLines: 2,
                        suffixIcon: title.text.isEmpty
                            ? null
                            : IconButton(
                                tooltip: _strings.get('reader.restoreDetected'),
                                onPressed: () => setDialogState(title.clear),
                                icon: const Icon(Icons.restore_rounded),
                              ),
                      ),
                      onChanged: (_) => setDialogState(() {}),
                    ),
                    TextField(
                      controller: author,
                      maxLength: 512,
                      decoration: InputDecoration(
                        labelText: _strings.get('reader.documentAuthor'),
                        helperText: _strings.get(
                          'reader.detectedValue',
                          parameters: {
                            'value':
                                sourceAuthor ??
                                _strings.get('reader.noDetectedValue'),
                          },
                        ),
                        helperMaxLines: 2,
                        suffixIcon: author.text.isEmpty
                            ? null
                            : IconButton(
                                tooltip: _strings.get('reader.restoreDetected'),
                                onPressed: () => setDialogState(author.clear),
                                icon: const Icon(Icons.restore_rounded),
                              ),
                      ),
                      onChanged: (_) => setDialogState(() {}),
                    ),
                    const SizedBox(height: AppSpacing.space16),
                    Text(
                      _strings.get('reader.metadataOrganization'),
                      style: Theme.of(context).textTheme.titleSmall,
                    ),
                    const SizedBox(height: AppSpacing.space8),
                    Wrap(
                      spacing: AppSpacing.space8,
                      runSpacing: AppSpacing.space8,
                      children: [
                        for (final topic in LibraryTopic.values)
                          FilterChip(
                            label: Text(_strings.get('topic.${topic.name}')),
                            selected: selectedTopics.contains(topic),
                            onSelected: (selected) => setDialogState(() {
                              selected
                                  ? selectedTopics.add(topic)
                                  : selectedTopics.remove(topic);
                            }),
                          ),
                      ],
                    ),
                    const SizedBox(height: AppSpacing.space20),
                    Text(
                      _strings.get('reader.metadataNotes'),
                      style: Theme.of(context).textTheme.titleSmall,
                    ),
                    const SizedBox(height: AppSpacing.space8),
                    TextField(
                      controller: note,
                      maxLength: 4000,
                      minLines: 3,
                      maxLines: 6,
                      decoration: InputDecoration(
                        labelText: _strings.get('reader.libraryNote'),
                        helperText: _strings.get('reader.storedOutsidePdf'),
                      ),
                      onChanged: (_) => setDialogState(() {}),
                    ),
                  ],
                ),
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: Text(_strings.get('action.cancel')),
              ),
              FilledButton.icon(
                onPressed: dirty
                    ? () => Navigator.pop(
                        context,
                        _MetadataEditResult(
                          metadata: MetadataEditState(
                            sourceTitle: document.sourceTitle ?? document.title,
                            sourceAuthor:
                                document.sourceAuthor ?? document.author,
                            titleOverride: title.text,
                            authorOverride: author.text,
                            libraryNote: note.text,
                          ),
                          topics: Set.unmodifiable(selectedTopics),
                        ),
                      )
                    : null,
                icon: const Icon(Icons.save_outlined, size: 18),
                label: Text(_strings.get('action.save')),
              ),
            ],
          );
        },
      ),
    );
    title.dispose();
    author.dispose();
    note.dispose();
    if (!mounted || edited == null) return;
    try {
      final value = edited.metadata.normalized();
      widget.services.documents.updateLocalMetadata(
        document.id,
        title: value.titleOverride,
        author: value.authorOverride,
        note: value.libraryNote,
      );
      if (!setEquals(initialTopics, edited.topics)) {
        widget.services.enrichment.setManualTopics(document.id, edited.topics);
      }
      widget.services.libraryIndex.refreshMetadata(document.id);
      await widget.services.library.reload();
      _showNotice(_strings.get('reader.metadataSaved'));
    } on ArgumentError {
      _showNotice(_strings.get('reader.invalidMetadata'));
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
        _showNotice(_strings.get('library.relocationMismatch'));
        return;
      }
    } on AppFailure catch (failure) {
      final presentation = FailurePresentations.forFailure(
        failure,
        strings: _strings,
      );
      _showNotice('${presentation.title}: ${presentation.message}');
      return;
    }
    await _openPath(selected.path);
  }

  Future<void> _toggleFavorite(LibraryDocument document) async {
    await widget.services.library.toggleFavorite(document.id);
    _showNotice(
      _strings.get(
        document.isFavorite
            ? 'library.removedFromFavorites'
            : 'library.addedToFavorites',
      ),
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
      _showNotice(_strings.get('library.coverUpdated'));
    }
  }

  Future<void> _removeDocument(LibraryDocument document) async {
    final isCurrentDocument =
        widget.services.reader.document?.id == document.id &&
        widget.services.reader.status == ReaderStatus.ready;
    final confirmed = await _confirmDocumentAction(
      title: _strings.get('library.removeFromLibrary'),
      message: _strings.get('library.removeFromLibrary.description'),
      confirmLabel: _strings.get('library.remove'),
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
    _showNotice(_strings.get('library.documentRemoved'));
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
                child: Text(_strings.get('action.cancel')),
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
      final messenger = ScaffoldMessenger.of(context);
      messenger
        ..hideCurrentSnackBar()
        ..showSnackBar(
          SnackBar(
            content: Semantics(liveRegion: true, child: Text(message)),
            duration: const Duration(milliseconds: 2400),
          ),
        );
    }
  }

  void _scheduleWindowBoundsSave() {
    _windowSaveTimer?.cancel();
    _windowSaveTimer = Timer(
      const Duration(milliseconds: 500),
      () => unawaited(_saveWindowBounds()),
    );
  }

  Future<void> _toggleWindowFullScreen() async {
    try {
      await widget.services.reader.toggleFullScreen();
      if (mounted) setState(() {});
    } catch (_) {
      if (!mounted) return;
      ScaffoldMessenger.of(
        _navigatorKey.currentContext ?? context,
      ).showSnackBar(
        SnackBar(content: Text(_strings.get('reader.enterFullScreenFailed'))),
      );
    }
  }

  Future<void> _saveWindowBounds() async {
    if (!_isDesktop ||
        widget.services.reader.isFullScreen ||
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
