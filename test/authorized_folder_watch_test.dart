import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/library/domain/authorized_folder_watch.dart';

void main() {
  test(
    'events outside authorized roots are ignored and accepted events debounce',
    () {
      final root = '${Directory.systemTemp.path}${Platform.pathSeparator}Books';
      final other =
          '${Directory.systemTemp.path}${Platform.pathSeparator}Other';
      final watch = AuthorizedFolderWatch(debounce: const Duration(seconds: 1))
        ..authorize(root);
      expect(
        watch.accept('$other${Platform.pathSeparator}book.pdf'),
        FolderWatchDecision.ignored,
      );
      final at = DateTime.utc(2026, 1, 1);
      expect(
        watch.accept('$root${Platform.pathSeparator}book.pdf', at: at),
        FolderWatchDecision.queuedForRevalidation,
      );
      expect(
        watch.drain(now: at.add(const Duration(milliseconds: 500))),
        isEmpty,
      );
      final ready = watch.drain(now: at.add(const Duration(seconds: 1)));
      expect(ready.single.path.toLowerCase(), contains('book.pdf'));
    },
  );

  test('pausing clears pending work and revoking a root is immediate', () {
    final books = '${Directory.systemTemp.path}${Platform.pathSeparator}Books';
    final other = '${Directory.systemTemp.path}${Platform.pathSeparator}Other';
    final watch = AuthorizedFolderWatch()..authorize(books);
    final at = DateTime.utc(2026, 1, 1);
    watch.accept('$books${Platform.pathSeparator}book.pdf', at: at);
    watch.setPaused(true);
    expect(watch.pendingCount, 0);
    expect(
      watch.accept('$books${Platform.pathSeparator}other.pdf'),
      FolderWatchDecision.paused,
    );
    watch.setPaused(false);
    watch.authorize(other);
    watch.accept('$other${Platform.pathSeparator}a.pdf', at: at);
    watch.revoke(other);
    expect(watch.pendingCount, 0);
  });
}
