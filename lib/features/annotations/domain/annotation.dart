enum AnnotationType {
  highlight('Resaltado'),
  underline('Subrayado'),
  strikeout('Tachado'),
  textNote('Nota de texto'),
  pageMarker('Marcador visual'),
  pageComment('Comentario de página');

  const AnnotationType(this.label);
  final String label;
}

enum AnnotationColor {
  yellow('Amarillo suave', 0xFFE6C75A),
  green('Verde suave', 0xFF79B58A),
  blue('Azul suave', 0xFF78A9C8),
  rose('Rosa suave', 0xFFC98E9E),
  violet('Violeta suave', 0xFFA897C8);

  const AnnotationColor(this.label, this.argb);
  final String label;
  final int argb;
}

enum AnnotationAttachmentStatus { attached, reviewRequired }

/// Editorial review state. This is deliberately independent from
/// [AnnotationAttachmentStatus], which only describes whether the geometry is
/// still attached to the same document fingerprint.
enum AnnotationReviewState { pending, accepted, resolved }

const Object _keepAnnotationText = Object();

final class NormalizedRect {
  const NormalizedRect({
    required this.left,
    required this.top,
    required this.right,
    required this.bottom,
  }) : assert(left >= 0 && left <= 1),
       assert(top >= 0 && top <= 1),
       assert(right >= 0 && right <= 1),
       assert(bottom >= 0 && bottom <= 1),
       assert(left <= right),
       assert(top <= bottom);

  final double left;
  final double top;
  final double right;
  final double bottom;

  double get width => right - left;
  double get height => bottom - top;

  Map<String, double> toJson() => {
    'left': left,
    'top': top,
    'right': right,
    'bottom': bottom,
  };

  factory NormalizedRect.fromJson(Map<String, Object?> json) => NormalizedRect(
    left: (json['left'] as num).toDouble(),
    top: (json['top'] as num).toDouble(),
    right: (json['right'] as num).toDouble(),
    bottom: (json['bottom'] as num).toDouble(),
  );

  NormalizedRect rotatedClockwise() => NormalizedRect(
    left: 1 - bottom,
    top: left,
    right: 1 - top,
    bottom: right,
  );

  bool get isMeaningful => width > 0.0001 && height > 0.0001;
}

final class PdfAnnotation {
  const PdfAnnotation({
    required this.id,
    required this.documentId,
    required this.pageIndex,
    required this.type,
    required this.color,
    required this.documentFingerprint,
    required this.createdAt,
    required this.updatedAt,
    this.note,
    this.selectedText,
    this.contextBefore,
    this.contextAfter,
    this.rects = const [],
    this.pageRotation = 0,
    this.status = AnnotationAttachmentStatus.attached,
    this.reviewState = AnnotationReviewState.pending,
    this.tags = const [],
  });

  final String id;
  final String documentId;
  final int pageIndex;
  final AnnotationType type;
  final AnnotationColor color;
  final String? note;
  final String? selectedText;
  final String? contextBefore;
  final String? contextAfter;
  final List<NormalizedRect> rects;
  final int pageRotation;
  final String documentFingerprint;
  final AnnotationAttachmentStatus status;
  final AnnotationReviewState reviewState;
  final List<String> tags;
  final DateTime createdAt;
  final DateTime updatedAt;

  PdfAnnotation copyWith({
    AnnotationType? type,
    AnnotationColor? color,
    Object? note = _keepAnnotationText,
    String? selectedText,
    String? contextBefore,
    String? contextAfter,
    List<NormalizedRect>? rects,
    int? pageRotation,
    String? documentFingerprint,
    AnnotationAttachmentStatus? status,
    AnnotationReviewState? reviewState,
    List<String>? tags,
    DateTime? updatedAt,
  }) => PdfAnnotation(
    id: id,
    documentId: documentId,
    pageIndex: pageIndex,
    type: type ?? this.type,
    color: color ?? this.color,
    note: identical(note, _keepAnnotationText) ? this.note : note as String?,
    selectedText: selectedText ?? this.selectedText,
    contextBefore: contextBefore ?? this.contextBefore,
    contextAfter: contextAfter ?? this.contextAfter,
    rects: rects ?? this.rects,
    pageRotation: pageRotation ?? this.pageRotation,
    documentFingerprint: documentFingerprint ?? this.documentFingerprint,
    status: status ?? this.status,
    reviewState: reviewState ?? this.reviewState,
    tags: tags ?? this.tags,
    createdAt: createdAt,
    updatedAt: updatedAt ?? this.updatedAt,
  );
}
