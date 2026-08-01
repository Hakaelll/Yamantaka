import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/pdf_engine/presentation/viewport_anchor.dart';

void main() {
  test('rotation keeps anchors normalized', () {
    const anchor = ViewportAnchor(
      pageIndex: 2,
      x: 0.2,
      y: 0.3,
      zoom: 1.5,
      rotation: 0,
    );
    final rotated = ViewportAnchorTransform.rotate(anchor, 90);
    expect(rotated.x, closeTo(0.7, 0.0001));
    expect(rotated.y, closeTo(0.2, 0.0001));
    expect(rotated.rotation, 90);
  });

  test('projection clamps an anchor to the page bounds', () {
    const anchor = ViewportAnchor(
      pageIndex: 0,
      x: 1,
      y: 1,
      zoom: 2,
      rotation: 0,
    );
    final region = ViewportAnchorTransform.project(
      anchor,
      width: 1000,
      height: 800,
      viewportWidth: 400,
      viewportHeight: 300,
    );
    expect(region.left, 800);
    expect(region.top, 650);
  });
}
