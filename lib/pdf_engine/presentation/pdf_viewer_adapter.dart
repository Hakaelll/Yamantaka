import 'package:flutter/widgets.dart';

import '../models/pdf_models.dart';

enum PdfViewFlow { continuous, single, facing }

enum PdfViewZoomMode { custom, fitWidth, fitPage }

enum PdfViewReadingDirection { leftToRight, rightToLeft }

final class PdfViewConfiguration {
  const PdfViewConfiguration({
    required this.backgroundColor,
    required this.margin,
    required this.flow,
    required this.rotation,
    required this.reduceMotion,
    this.initialZoom = 1,
    this.initialZoomMode = PdfViewZoomMode.fitPage,
    this.initialScrollOffset = 0,
    this.readingDirection = PdfViewReadingDirection.leftToRight,
    this.pageShadow,
  });

  final Color backgroundColor;
  final double margin;
  final PdfViewFlow flow;
  final int rotation;
  final bool reduceMotion;
  final double initialZoom;
  final PdfViewZoomMode initialZoomMode;
  final double initialScrollOffset;
  final PdfViewReadingDirection readingDirection;
  final BoxShadow? pageShadow;
}

final class PdfViewController extends ChangeNotifier {
  int currentPageIndex = 0;
  int pageCount = 0;
  double zoom = 1;
  double scrollOffset = 0;
  bool isReady = false;

  Future<void> Function(int pageIndex)? _goToPage;
  Future<void> Function(double factor)? _zoomBy;
  Future<void> Function()? _fitWidth;
  Future<void> Function()? _fitPage;

  Future<void> goToPage(int pageIndex) async {
    if (_goToPage case final callback?) await callback(pageIndex);
  }

  Future<void> zoomBy(double factor) async {
    if (_zoomBy case final callback?) await callback(factor);
  }

  Future<void> fitWidth() async {
    if (_fitWidth case final callback?) await callback();
  }

  Future<void> fitPage() async {
    if (_fitPage case final callback?) await callback();
  }

  void attach({
    required int pageCount,
    required int currentPageIndex,
    required double zoom,
    required Future<void> Function(int pageIndex) goToPage,
    required Future<void> Function(double factor) zoomBy,
    required Future<void> Function() fitWidth,
    required Future<void> Function() fitPage,
  }) {
    this.pageCount = pageCount;
    this.currentPageIndex = currentPageIndex;
    this.zoom = zoom;
    _goToPage = goToPage;
    _zoomBy = zoomBy;
    _fitWidth = fitWidth;
    _fitPage = fitPage;
    isReady = true;
    notifyListeners();
  }

  void update({int? pageIndex, double? zoom, double? scrollOffset}) {
    if (pageIndex != null) currentPageIndex = pageIndex;
    if (zoom != null) this.zoom = zoom;
    if (scrollOffset != null) this.scrollOffset = scrollOffset;
    notifyListeners();
  }

  void detach() {
    isReady = false;
    scrollOffset = 0;
    _goToPage = null;
    _zoomBy = null;
    _fitWidth = null;
    _fitPage = null;
  }
}

abstract interface class PdfViewerAdapter {
  Widget buildViewer({
    required OpenedPdfDocument document,
    required PdfViewController controller,
    required PdfViewConfiguration configuration,
    required int initialPageIndex,
    required ValueChanged<int> onPageChanged,
    required ValueChanged<Object> onError,
  });
}
