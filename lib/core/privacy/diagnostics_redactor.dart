import 'dart:convert';

final class DiagnosticsRedactor {
  static const includedFields = <String>{
    'yamantaka_version',
    'flutter_version',
    'platform',
    'platform_version',
    'architecture',
    'pdf_engine',
    'database_schema',
    'database_bytes',
    'cache_bytes',
    'documents',
    'library_notes',
    'missing_documents',
    'indexed_documents',
    'annotations',
    'collections',
    'tags',
    'workspaces',
    'integrity',
    'safe_mode',
    'storage_mode',
    'engine',
    'jobs',
    'memory',
    'lifecycle',
    'startup',
    'performance',
  };

  static const excludedFields = <String>{
    'path',
    'paths',
    'title',
    'author',
    'query',
    'selected_text',
    'note',
    'body',
    'password',
    'pdf_bytes',
  };

  static Map<String, Object?> redact(Map<String, Object?> values) => {
    for (final entry in values.entries)
      if (includedFields.contains(entry.key))
        entry.key: _safeValue(entry.value),
  };

  static Map<String, Object?> manifest() => {
    'included_fields': includedFields.toList()..sort(),
    'excluded_fields': excludedFields.toList()..sort(),
    'contains_private_content': false,
  };

  static String preview(Map<String, Object?> values) =>
      const JsonEncoder.withIndent(
        '  ',
      ).convert({'manifest': manifest(), 'values': redact(values)});

  static void assertSafe(Map<String, Object?> values) {
    _assertSafeValue(redact(values));
  }

  static void _assertSafeValue(Object? value) {
    if (value is Map) {
      for (final entry in value.entries) {
        if (entry.key is! String ||
            excludedFields.contains(entry.key as String)) {
          throw StateError('Unsafe diagnostics field detected.');
        }
        _assertSafeValue(entry.value);
      }
      return;
    }
    if (value is Iterable) {
      for (final item in value) {
        _assertSafeValue(item);
      }
      return;
    }
    if (value is String &&
        (value.contains('\\') ||
            value.contains('/Users/') ||
            value.toLowerCase().contains('.pdf'))) {
      throw StateError('Unsafe diagnostics content detected.');
    }
  }

  static Object? _safeValue(Object? value) {
    if (value is Map) {
      return {
        for (final entry in value.entries)
          if (entry.key is String && !excludedFields.contains(entry.key))
            entry.key as String: _safeValue(entry.value),
      };
    }
    if (value is Iterable) return [for (final item in value) _safeValue(item)];
    if (value is String && value.length > 120) return value.substring(0, 120);
    return value;
  }
}
