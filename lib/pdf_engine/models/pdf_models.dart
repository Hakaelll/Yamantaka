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

final class PdfDocumentMetadata {
  const PdfDocumentMetadata({this.title, this.author});

  final String? title;
  final String? author;
}

final class PdfStructuredTextBlock {
  const PdfStructuredTextBlock({
    required this.text,
    required this.left,
    required this.top,
    required this.width,
    required this.height,
    required this.pageWidth,
    required this.pageHeight,
  });

  final String text;
  final double left;
  final double top;
  final double width;
  final double height;
  final double pageWidth;
  final double pageHeight;

  double get relativeHeight => pageHeight <= 0 ? 0 : height / pageHeight;
  double get relativeTop => pageHeight <= 0 ? 0 : top / pageHeight;
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

/// A rendered page whose image is owned by the PDF engine.
///
/// Consumers may read or encode [image], but must not dispose it. The engine
/// releases the image when its cache entry is evicted or the document closes.
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
  final Set<void Function()> _cancellationListeners = <void Function()>{};

  bool get isCancelled => _cancelled;

  void addCancellationListener(void Function() listener) {
    if (_cancelled) {
      try {
        listener();
      } on Object {
        // Cancellation must remain best-effort for an obsolete job.
      }
      return;
    }
    _cancellationListeners.add(listener);
  }

  void removeCancellationListener(void Function() listener) {
    _cancellationListeners.remove(listener);
  }

  void cancel() {
    if (_cancelled) return;
    _cancelled = true;
    final listeners = _cancellationListeners.toList(growable: false);
    _cancellationListeners.clear();
    for (final listener in listeners) {
      try {
        listener();
      } on Object {
        // A cancellation callback must not prevent the remaining callbacks.
      }
    }
  }
}
