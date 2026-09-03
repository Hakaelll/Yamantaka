import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/app_config.dart';
import 'package:yamantaka/core/errors/app_failure.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/library/application/duplicate_resolution_service.dart';
import 'package:yamantaka/features/library/domain/duplicate_resolution.dart';
import 'package:yamantaka/features/library/domain/library_document.dart';
import 'package:yamantaka/features/reader/domain/document_lifecycle.dart';
import 'package:yamantaka/features/recovery/domain/recovery_models.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/automation_repository.dart';
import 'package:yamantaka/storage/repositories/document_repository.dart';
import 'package:yamantaka/storage/repositories/recovery_repository.dart';

void main() {
  group('0.13 migration and recovery contracts', () {
    test(
      'every supported schema fixture migrates without losing documents',
      () {
        for (var version = 1; version <= AppConfig.databaseVersion; version++) {
          final database = AppDatabase.inMemoryAtVersion(
            AppLogger(minimumLevel: LogLevel.error),
            version,
          );
          try {
            expect(_userVersion(database), version);
            database.database.execute(
              '''
            INSERT INTO documents(
              id, path, normalized_path, file_name, page_count, file_size,
              file_modified_at, added_at, last_opened_at, last_page_index,
              reading_progress, preferred_view_mode, preferred_theme,
              is_missing, fingerprint
            ) VALUES (?, ?, ?, ?, 1, 1, ?, ?, ?, 0, 0, 'document', 'ivory',
              0, ?)
            ''',
              [
                'fixture-$version',
                'fixture-$version.pdf',
                'fixture-$version.pdf',
                'fixture-$version.pdf',
                '2026-08-31T00:00:00.000Z',
                '2026-08-31T00:00:00.000Z',
                '2026-08-31T00:00:00.000Z',
                'fingerprint-$version',
              ],
            );
            database.migrateToCurrentForTesting();
            expect(_userVersion(database), AppConfig.databaseVersion);
            database.migrateToCurrentForTesting();
            expect(_userVersion(database), AppConfig.databaseVersion);
            expect(database.integrityCheck(), 'ok');
            expect(
              database.database.select('SELECT id FROM documents').single['id'],
              'fixture-$version',
            );
          } finally {
            database.close();
          }
        }
      },
    );

    test(
      'atomic session omits paths and passwords and keeps recovery identity',
      () {
        final snapshot = AtomicSessionSnapshot(
          workspaceId: 'workspace',
          activeDocumentId: 'document',
          activeDocumentFingerprint: 'fingerprint',
          pageIndex: 8,
          pageCount: 20,
          quarantinedDocumentId: 'failed-document',
          lifecycleState: DocumentLifecycleState.wrongPassword.name,
          savedAt: DateTime.utc(2026, 8, 31),
          clean: false,
        );
        final json = snapshot.toJson();
        expect(json.values, isNot(contains('secret')));
        expect(json.keys, isNot(contains('path')));
        final restored = AtomicSessionSnapshot.fromJson(json);
        expect(restored.activeDocumentFingerprint, 'fingerprint');
        expect(restored.pageIndex, 8);
        expect(restored.quarantinedDocumentId, 'failed-document');
      },
    );

    test(
      'typed lifecycle distinguishes password and terminal document states',
      () {
        const wrongPassword = AppFailure(FailureKind.wrongPassword, 'wrong');
        final lifecycle = DocumentLifecycleSnapshot.failure(
          generation: 4,
          failure: wrongPassword,
          documentId: 'document',
          passwordAttempt: AppConfig.maxPdfPasswordAttempts,
        );
        expect(lifecycle.state, DocumentLifecycleState.wrongPassword);
        expect(lifecycle.passwordAttempt, 3);
        expect(lifecycle.canRetry, isTrue);
      },
    );

    test('failed documents are quarantined until the warning is discarded', () {
      final database = AppDatabase.inMemory(
        AppLogger(minimumLevel: LogLevel.error),
      );
      addTearDown(database.close);
      final documents = DocumentRepository(database);
      documents.save(_document('failed'));
      documents.markOpenFailure('failed', FailureKind.corruptPdf.name);

      final recovery = RecoveryRepository(database);
      expect(documents.automaticRestoreBlocked('failed'), isTrue);
      expect(recovery.problems().single.code, 'document_restore_blocked');
      recovery.discardProblem('failed');
      expect(documents.automaticRestoreBlocked('failed'), isFalse);
    });

    test('journals persist phase, reason, capability and compatibility', () {
      final database = AppDatabase.inMemory(
        AppLogger(minimumLevel: LogLevel.error),
      );
      addTearDown(database.close);
      final repository = AutomationRepository(database);
      final journal = repository.startJournal(
        kind: 'automation',
        totalItems: 2,
        phase: 'applying',
        resumeCapability: JournalResumeCapability.retry,
        compatibilityHash: 'recipe-v1',
      );
      repository.updateJournal(
        journal.id,
        status: JournalStatus.running,
        completedItems: 1,
        undo: const {
          'first': {'documentId': 'one'},
        },
        phase: 'applying',
        resumeCapability: JournalResumeCapability.rollback,
      );
      repository.markRunningAsInterrupted();
      final interrupted = repository.journal(journal.id)!;
      expect(interrupted.phase, 'applying');
      expect(interrupted.interruptionReason, 'process_interrupted');
      expect(interrupted.resumeCapability, JournalResumeCapability.rollback);
      expect(interrupted.compatibilityHash, 'recipe-v1');
    });
  });

  group('0.13 duplicate consolidation', () {
    test(
      'surfaces evidence, preserves annotation origin and records undo',
      () async {
        final database = AppDatabase.inMemory(
          AppLogger(minimumLevel: LogLevel.error),
        );
        addTearDown(database.close);
        final documents = DocumentRepository(database);
        documents.save(_document('kept', fingerprint: 'same'));
        documents.save(_document('incoming', fingerprint: 'same'));
        final now = DateTime.utc(2026, 8, 31).toIso8601String();
        database.database.execute(
          '''
        INSERT INTO annotations(
          id, document_id, page_index, type, color, rects_json,
          page_rotation, document_fingerprint, status, created_at, updated_at
        ) VALUES ('annotation', 'incoming', 0, 'note', 'yellow', '[]', 0,
          'same', 'attached', ?, ?)
        ''',
          [now, now],
        );
        String? restoredPath;
        final service = DuplicateResolutionService(
          database: database,
          documents: documents,
          createPreBackup: () async => 'pre-merge.json',
          restoreBackup: (path) async {
            restoredPath = path;
            return 'pre-import.json';
          },
        );
        final candidates = service.candidates();
        expect(candidates, hasLength(1));
        expect(
          candidates.single.evidence.kind,
          DuplicateMatchKind.exactFingerprint,
        );
        expect(service.previewImpact('kept', 'incoming').localRecords, 2);

        final result = await service.resolve(
          existingId: 'kept',
          incomingId: 'incoming',
          action: DuplicateResolutionAction.merge,
          confirmed: true,
        );
        expect(documents.byId('incoming'), isNull);
        final annotation = database.database
            .select("SELECT * FROM annotations WHERE id = 'annotation'")
            .single;
        expect(annotation['document_id'], 'kept');
        expect(annotation['origin_document_id'], 'incoming');
        expect(annotation['origin_fingerprint'], 'same');
        expect(result.historyId, isNotNull);

        expect(await service.undoMerge(result.historyId!), isTrue);
        expect(restoredPath, 'pre-merge.json');
        expect(
          database.database.select(
            'SELECT status FROM duplicate_merge_history WHERE id = ?',
            [result.historyId],
          ).single['status'],
          'rolledBack',
        );
      },
    );

    test('collapses relation cycles and duplicate edges safely', () async {
      final database = AppDatabase.inMemory(
        AppLogger(minimumLevel: LogLevel.error),
      );
      addTearDown(database.close);
      final documents = DocumentRepository(database);
      documents.save(_document('kept', fingerprint: 'same'));
      documents.save(_document('incoming', fingerprint: 'same'));
      documents.save(_document('third'));
      final now = DateTime.utc(2026, 9, 1).toIso8601String();
      void relation(String id, String source, String target, String type) {
        database.database.execute(
          '''
          INSERT INTO document_relations(
            id, source_document_id, target_document_id, relation_type,
            created_at
          ) VALUES (?, ?, ?, ?, ?)
          ''',
          [id, source, target, type, now],
        );
      }

      relation('kept-edge', 'kept', 'third', 'related');
      relation('duplicate-edge', 'incoming', 'third', 'related');
      relation('cycle', 'incoming', 'kept', 'replaces');
      relation('backlink', 'third', 'incoming', 'sourceOf');
      final service = DuplicateResolutionService(
        database: database,
        documents: documents,
        createPreBackup: () async => 'pre-merge.json',
      );

      await service.resolve(
        existingId: 'kept',
        incomingId: 'incoming',
        action: DuplicateResolutionAction.merge,
        confirmed: true,
      );

      final relations = database.database.select(
        'SELECT source_document_id, target_document_id, relation_type '
        'FROM document_relations ORDER BY id',
      );
      expect(relations, hasLength(2));
      expect(
        relations.where(
          (row) =>
              row['source_document_id'] == 'incoming' ||
              row['target_document_id'] == 'incoming' ||
              row['source_document_id'] == row['target_document_id'],
        ),
        isEmpty,
      );
      expect(database.database.select('PRAGMA foreign_key_check'), isEmpty);
    });
  });
}

int _userVersion(AppDatabase database) =>
    database.database.select('PRAGMA user_version').first.columnAt(0) as int;

LibraryDocument _document(String id, {String? fingerprint}) {
  final now = DateTime.utc(2026, 8, 31);
  return LibraryDocument(
    id: id,
    path: '$id.pdf',
    normalizedPath: '$id.pdf',
    fileName: 'shared.pdf',
    title: 'Shared',
    pageCount: 10,
    fileSize: 100,
    fileModifiedAt: now,
    addedAt: now,
    lastOpenedAt: now,
    fingerprint: fingerprint ?? id,
  );
}
