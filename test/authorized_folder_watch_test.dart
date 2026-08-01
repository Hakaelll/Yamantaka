import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/features/library/domain/authorized_folder_watch.dart';

void main() {
  test(
    'events outside authorized roots are ignored and accepted events debounce',
    () {
      final watch = AuthorizedFolderWatch(debounce: const Duration(seconds: 1))
        ..authorize('C:/Books');
      expect(watch.accept('C:/Other/book.pdf'), FolderWatchDecision.ignored);
      final at = DateTime.utc(2026, 1, 1);
      expect(
        watch.accept('C:/Books/book.pdf', at: at),
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
    final watch = AuthorizedFolderWatch()..authorize('C:/Books');
    final at = DateTime.utc(2026, 1, 1);
    watch.accept('C:/Books/book.pdf', at: at);
    watch.setPaused(true);
    expect(watch.pendingCount, 0);
    expect(watch.accept('C:/Books/other.pdf'), FolderWatchDecision.paused);
    watch.setPaused(false);
    watch.authorize('C:/Other');
    watch.accept('C:/Other/a.pdf', at: at);
    watch.revoke('C:/Other');
    expect(watch.pendingCount, 0);
  });
}
