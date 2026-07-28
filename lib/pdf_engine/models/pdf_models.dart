import 'dart:ui' as ui;

final class PdfPageSize {
  const PdfPageSize(this.width, this.height);

  final double width;
  final double height;
}

final class OpenedPdfDocument {
  const OpenedPdfDocument({
    required this.documentId,
    required this.path,
    required this.pageCount,
    required this.pageSizes,
    required this.isEncrypted,
  });

  final String documentId;
  final String path;
  final int pageCount;
  final List<PdfPageSize> pageSizes;
  final bool isEncrypted;
}

final class PdfRenderRegion {
  const PdfRenderRegion({
    required this.left,
    required this.top,
    required this.width,
    required this.height,
  });

  final double left;
  final double top;
  final double width;
  final double height;
}

final class RenderedPdfPage {
  const RenderedPdfPage({
    required this.image,
    required this.pageIndex,
    required this.scale,
  });

  final ui.Image image;
  final int pageIndex;
  final double scale;

  int get estimatedBytes => image.width * image.height * 4;

  void dispose() => image.dispose();
}

final class PdfSearchMatch {
  const PdfSearchMatch({
    required this.pageIndex,
    required this.start,
    required this.length,
    required this.snippet,
  });

  final int pageIndex;
  final int start;
  final int length;
  final String snippet;
}

final class PdfOutlineItem {
  const PdfOutlineItem({
    required this.title,
    this.pageIndex,
    this.children = const [],
  });

  final String title;
  final int? pageIndex;
  final List<PdfOutlineItem> children;
}

final class PdfLink {
  const PdfLink({
    this.uri,
    this.destinationPageIndex,
    required this.rectangles,
  });

  final Uri? uri;
  final int? destinationPageIndex;
  final List<PdfRenderRegion> rectangles;
}

final class PdfOperationToken {
  bool _cancelled = false;

  bool get isCancelled => _cancelled;

  void cancel() => _cancelled = true;
}
