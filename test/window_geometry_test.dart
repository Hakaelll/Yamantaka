import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/platform/window_geometry.dart';

void main() {
  test('repositions an off-screen monitor without losing usable size', () {
    final bounds =
        const WindowBounds(
          x: -2400,
          y: -900,
          width: 1200,
          height: 900,
        ).normalized(
          workArea: const WindowWorkArea(
            left: 0,
            top: 0,
            width: 1920,
            height: 1080,
          ),
        );
    expect(bounds.x, greaterThanOrEqualTo(-1200 + 24));
    expect(bounds.y, greaterThanOrEqualTo(-900 + 24));
    expect(bounds.width, 1200);
    expect(bounds.height, 900);
  });

  test('clamps a window larger than the available work area', () {
    final bounds = const WindowBounds(x: 0, y: 0, width: 4000, height: 2000)
        .normalized(
          workArea: const WindowWorkArea(
            left: 0,
            top: 0,
            width: 1920,
            height: 1080,
          ),
        );
    expect(bounds.width, 1920);
    expect(bounds.height, 1080);
  });

  test(
    'does not throw when a remote work area is smaller than the minimum',
    () {
      final bounds = const WindowBounds(x: 0, y: 0, width: 640, height: 480)
          .normalized(
            workArea: const WindowWorkArea(
              left: 0,
              top: 0,
              width: 320,
              height: 240,
            ),
          );
      expect(bounds.width, 320);
      expect(bounds.height, 240);
    },
  );
}
