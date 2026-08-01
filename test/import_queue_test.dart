import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/library/domain/import_queue.dart';

void main() {
  test(
    'deduplicates paths, retries isolated failures and resumes running work',
    () {
      final queue = ImportQueue(maxPending: 2);
      final first = const ImportQueueItem(
        id: 'a',
        path: r'C:\Books\a.pdf',
        normalizedPath: r'c:\books\a.pdf',
        fingerprint: 'a',
      );
      expect(queue.enqueue(first), isTrue);
      expect(queue.enqueue(first), isFalse);
      expect(
        queue.enqueue(
          const ImportQueueItem(
            id: 'same-fingerprint',
            path: r'D:\Books\copy.pdf',
            normalizedPath: r'd:\books\copy.pdf',
            fingerprint: 'a',
          ),
        ),
        isFalse,
      );

      final running = queue.claimNext()!;
      expect(running.state, ImportWorkState.running);
      queue.prepareForResume();
      expect(queue.claimNext()?.id, 'a');
      queue.fail('a', 'Access denied', retryable: true);
      expect(queue.items.single.state, ImportWorkState.pending);
    },
  );

  test('serialized queue never restores running work as running', () {
    final queue = ImportQueue();
    queue.enqueue(
      const ImportQueueItem(
        id: 'a',
        path: r'C:\Books\a.pdf',
        normalizedPath: r'c:\books\a.pdf',
      ),
    );
    queue.claimNext();
    final restored = ImportQueue.fromJson(queue.toJson());
    expect(restored.items.single.state, ImportWorkState.pending);
  });
}
