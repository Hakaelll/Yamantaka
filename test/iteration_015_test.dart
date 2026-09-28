import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/app/theme/app_tokens.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/library_controller.dart';
import 'package:yamantaka/features/library/application/duplicate_resolution_service.dart';
import 'package:yamantaka/features/library/domain/duplicate_resolution.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/library/presentation/duplicate_review_dialog.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';
import 'package:yamantaka/features/organization/domain/organization_models.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';

void main() {
  testWidgets('minimal library keeps navigation and reset actions on demand', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1280, 720);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    final repository = DocumentRepository(database);
    repository.save(_document('reading', progress: .4));
    repository.save(_document('favorite', favorite: true));
    repository.save(_document('finished', progress: 1));
    final controller = LibraryController(
      repository,
      AppLogger(minimumLevel: LogLevel.error),
    );
    var openedSettings = false;
    await controller.load();
    await tester.pumpWidget(
      _app(
        LibraryScreen(
          controller: controller,
          onOpenPath: (_) async {},
          onPickDocument: () async {},
          onOpenSettings: () => openedSettings = true,
          onAnnotations: () {},
          onProblems: () {},
          showRecents: false,
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('library-navigation')), findsNothing);
    expect(find.byTooltip('Ajustes'), findsNothing);
    expect(find.text('Biblioteca'), findsNothing);
    expect(find.text('3 documentos'), findsOneWidget);
    await tester.tap(find.byKey(const Key('library-view-list-button')));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('library-view-grid-button')), findsOneWidget);
    await tester.tap(find.byKey(const Key('library-view-grid-button')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('library-selection-button')));
    await tester.pumpAndSettle();
    expect(find.byTooltip('Salir del modo de selección'), findsOneWidget);
    await tester.tap(find.byKey(const Key('library-selection-button')));
    await tester.pumpAndSettle();
    expect(find.text('Revisión de anotaciones'), findsNothing);
    expect(find.text('Problemas y recuperación'), findsNothing);
    await tester.tap(find.byTooltip('Organizar y más acciones'));
    await tester.pumpAndSettle();
    expect(find.text('Revisión de anotaciones'), findsOneWidget);
    expect(find.text('Problemas y recuperación'), findsOneWidget);
    expect(find.text('Ajustes'), findsOneWidget);
    await tester.tap(find.text('Ajustes'));
    await tester.pumpAndSettle();
    expect(openedSettings, isTrue);
    expect(controller.documents.length, 3);
    controller.query = 'reading';
    controller.selectedSmartCollection = SmartCollection.inProgress;
    await controller.reload();
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('library-reset-filters-button')));
    await tester.pumpAndSettle();
    expect(controller.query, isEmpty);
    expect(controller.selectedSmartCollection, isNull);
    expect(controller.documents.length, 3);
    await tester.tap(find.byKey(const Key('library-search-button')));
    await tester.pumpAndSettle();
    expect(find.byType(TextField), findsOneWidget);
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();
    expect(find.byType(TextField), findsNothing);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox.shrink());
    controller.dispose();
    database.close();
  });

  test('a single document waits for an explicit selection', () async {
    final database = AppDatabase.inMemory(
      AppLogger(minimumLevel: LogLevel.error),
    );
    addTearDown(database.close);
    final repository = DocumentRepository(database)
      ..save(_document('only-document'));
    final controller = LibraryController(
      repository,
      AppLogger(minimumLevel: LogLevel.error),
    );
    addTearDown(controller.dispose);

    await controller.load();

    expect(controller.documents, hasLength(1));
    expect(controller.selectedDocumentId, isNull);
  });

  testWidgets('duplicate decisions remain attached to their own pair', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1280, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final candidates = [
      for (final id in ['first', 'second'])
        DuplicateCandidate(
          existing: _document('kept-$id'),
          incoming: _document(id),
          evidence: const DuplicateEvidence(
            kind: DuplicateMatchKind.exactFingerprint,
            score: 100,
            sameFingerprint: true,
          ),
        ),
    ];
    await tester.pumpWidget(
      _app(
        DuplicateReviewDialog(
          candidates: candidates,
          impactFor: (_) => const DuplicateMergeImpact(
            localRecords: 2,
            conflictFields: ['title', 'library_note'],
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Contenido idéntico'), findsOneWidget);
    expect(find.textContaining('true'), findsNothing);
    final first = find.byKey(const ValueKey('duplicate-conflict-first-title'));
    await tester.tap(first);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Usar valor entrante').last);
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Siguiente'));
    await tester.pumpAndSettle();
    final secondState = tester.state<FormFieldState<MetadataConflictChoice>>(
      find.byKey(const ValueKey('duplicate-conflict-second-title')),
    );
    expect(secondState.value, MetadataConflictChoice.existing);
    await tester.tap(find.byTooltip('Anterior'));
    await tester.pumpAndSettle();
    expect(
      tester.state<FormFieldState<MetadataConflictChoice>>(first).value,
      MetadataConflictChoice.incoming,
    );
    expect(tester.takeException(), isNull);
  });

  test(
    'primary controls provide distinct disabled and focus states in both themes',
    () {
      for (final theme in [AppTheme.light(), AppTheme.dark()]) {
        final style = theme.filledButtonTheme.style!;
        expect(style.side!.resolve({WidgetState.focused})!.width, 2);
        expect(
          style.foregroundColor!.resolve({WidgetState.disabled}),
          isNot(style.foregroundColor!.resolve({})),
        );
        expect(AppSizes.readerRestingDockHeight, greaterThanOrEqualTo(40));
      }
    },
  );
}

Widget _app(Widget child) => MaterialApp(
  theme: AppTheme.light(),
  localizationsDelegates: const [
    YamantakaLocalizations.delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
    GlobalCupertinoLocalizations.delegate,
  ],
  locale: const Locale('es'),
  supportedLocales: const [Locale('es'), Locale('en')],
  home: Scaffold(body: child),
);

LibraryDocument _document(
  String id, {
  double progress = 0,
  bool favorite = false,
}) => LibraryDocument(
  id: id,
  path: 'C:/fixtures/$id.pdf',
  normalizedPath: 'c:/fixtures/$id.pdf',
  fileName: '$id.pdf',
  title: 'Documento $id',
  author: 'Autora local',
  pageCount: 10,
  fileSize: 100,
  fileModifiedAt: DateTime.utc(2026, 8, 26),
  addedAt: DateTime.utc(2026, 8, 26),
  lastOpenedAt: DateTime.utc(2026, 8, 26),
  fingerprint: id,
  readingProgress: progress,
  lastPageIndex: (progress * 9).round(),
  isFavorite: favorite,
);
