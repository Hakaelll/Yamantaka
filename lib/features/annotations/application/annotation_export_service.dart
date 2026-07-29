import 'dart:convert';

import '../domain/annotation.dart';

final class AnnotationExportService {
  const AnnotationExportService();

  String toMarkdown({
    required String title,
    String? author,
    required double progress,
    required List<PdfAnnotation> annotations,
    DateTime? exportedAt,
  }) {
    final timestamp = (exportedAt ?? DateTime.now()).toUtc();
    final buffer = StringBuffer()
      ..writeln('# ${_markdownText(title)}')
      ..writeln()
      ..writeln('- Exportado: ${timestamp.toIso8601String()}')
      ..writeln('- Progreso: ${(progress * 100).clamp(0, 100).round()} %');
    if (author != null && author.trim().isNotEmpty) {
      buffer.writeln('- Autor: ${_markdownText(author)}');
    }
    final sorted = [...annotations]
      ..sort((a, b) => a.pageIndex.compareTo(b.pageIndex));
    for (final annotation in sorted) {
      buffer
        ..writeln()
        ..writeln(
          '## Página ${annotation.pageIndex + 1} — ${annotation.type.label}',
        )
        ..writeln()
        ..writeln('- Color: ${annotation.color.label}');
      if (annotation.tags.isNotEmpty) {
        buffer.writeln(
          '- Etiquetas: ${annotation.tags.map(_markdownText).join(', ')}',
        );
      }
      if (annotation.selectedText case final text?) {
        buffer
          ..writeln()
          ..writeln('> ${_markdownText(text).replaceAll('\n', '\n> ')}');
      }
      if (annotation.note case final note?) {
        buffer
          ..writeln()
          ..writeln(_markdownText(note));
      }
      buffer
        ..writeln()
        ..writeln(
          '[Abrir página ${annotation.pageIndex + 1}](#page-${annotation.pageIndex + 1})',
        );
    }
    return buffer.toString();
  }

  String toVersionedJson({
    required String documentId,
    required String title,
    String? author,
    required double progress,
    required List<PdfAnnotation> annotations,
    DateTime? exportedAt,
  }) => const JsonEncoder.withIndent('  ').convert({
    'format': 'yamantaka-annotations',
    'version': 1,
    'exportedAt': (exportedAt ?? DateTime.now()).toUtc().toIso8601String(),
    'metadata': {
      'documentId': documentId,
      'title': title,
      'author': ?author,
      'progress': progress.clamp(0, 1),
    },
    'configuration': {'coordinates': 'normalized-page-v1'},
    'annotations': annotations
        .map(
          (item) => {
            'id': item.id,
            'pageIndex': item.pageIndex,
            'type': item.type.name,
            'color': item.color.name,
            if (item.note != null) 'note': item.note,
            if (item.selectedText != null) 'selectedText': item.selectedText,
            if (item.contextBefore != null) 'contextBefore': item.contextBefore,
            if (item.contextAfter != null) 'contextAfter': item.contextAfter,
            'rects': item.rects.map((rect) => rect.toJson()).toList(),
            'pageRotation': item.pageRotation,
            'documentFingerprint': item.documentFingerprint,
            'status': item.status.name,
            'tags': item.tags,
            'createdAt': item.createdAt.toUtc().toIso8601String(),
            'updatedAt': item.updatedAt.toUtc().toIso8601String(),
          },
        )
        .toList(),
  });

  static String _markdownText(String value) =>
      value.replaceAll(r'\', r'\\').replaceAll(RegExp(r'([#*_[\]`])'), r'\$1');
}
