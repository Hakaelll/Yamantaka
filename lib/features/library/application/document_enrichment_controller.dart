import 'dart:async';
import 'dart:convert';
import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as p;

import '../../../pdf_engine/models/pdf_models.dart';
import '../../../pdf_engine/pdf_document_engine.dart';
import '../../../storage/repositories/document_repository.dart';
import '../domain/document_enrichment.dart';
import '../domain/library_document.dart';

final class EnrichmentCandidate {
  const EnrichmentCandidate(this.value, this.confidence);

  final String value;
  final double confidence;
}

final class LocalEnrichmentResult {
  const LocalEnrichmentResult({
    this.title,
    this.author,
    this.topics = const [],
    this.hasExtractableText = false,
  });

  final EnrichmentCandidate? title;
  final EnrichmentCandidate? author;
  final List<DocumentTopicAssignment> topics;
  final bool hasExtractableText;
}

final class EnrichmentProgress {
  const EnrichmentProgress({
    this.completed = 0,
    this.total = 0,
    this.currentTitle,
    this.isRunning = false,
  });

  final int completed;
  final int total;
  final String? currentTitle;
  final bool isRunning;
}

final class DocumentEnrichmentController extends ChangeNotifier {
  DocumentEnrichmentController(this._documents, this._engine);

  static const analyzerVersion = 2;
  static const maximumPages = 12;
  static const maximumTextBytes = 64 * 1024;
  static const automaticConfidence = 0.80;
  static const suggestionConfidence = 0.55;

  final DocumentRepository _documents;
  final PdfDocumentEngine _engine;
  PdfOperationToken? _operationToken;
  bool _cancelled = false;
  bool _paused = false;
  bool _readerPaused = false;
  bool _interactionPaused = false;

  bool get isPaused => _paused || _readerPaused || _interactionPaused;

  EnrichmentProgress progress = const EnrichmentProgress();

  Future<void> analyzePending() async {
    if (progress.isRunning) return;
    final candidates = _documents.needingEnrichment(analyzerVersion);
    await _analyzeCandidates(candidates);
  }

  Future<void> reanalyzeAll() => _analyzeCandidates(
    _documents.all().where((document) => !document.isMissing).toList(),
  );

  Future<void> reanalyzeDocuments(Iterable<LibraryDocument> documents) async {
    final candidates = <LibraryDocument>[];
    for (final requested in documents) {
      final current = _documents.byId(requested.id);
      if (current != null && !current.isMissing) candidates.add(current);
    }
    await _analyzeCandidates(candidates);
  }

  Future<void> _analyzeCandidates(List<LibraryDocument> candidates) async {
    if (progress.isRunning || candidates.isEmpty) return;
    _cancelled = false;
    progress = EnrichmentProgress(total: candidates.length, isRunning: true);
    notifyListeners();
    var completed = 0;
    for (final document in candidates) {
      if (_cancelled) break;
      while (isPaused && !_cancelled) {
        await Future<void>.delayed(const Duration(milliseconds: 80));
      }
      if (_cancelled) break;
      progress = EnrichmentProgress(
        completed: completed,
        total: candidates.length,
        currentTitle: document.effectiveTitle,
        isRunning: true,
      );
      notifyListeners();
      await analyzeDocument(document);
      completed++;
    }
    progress = EnrichmentProgress(
      completed: completed,
      total: candidates.length,
      isRunning: false,
    );
    notifyListeners();
  }

  Future<void> analyzeDocument(LibraryDocument document) async {
    if (document.isMissing) return;
    if (!progress.isRunning) _cancelled = false;
    _operationToken?.cancel();
    final token = PdfOperationToken();
    _operationToken = token;
    _documents.beginEnrichment(document, analyzerVersion);
    String? engineDocumentId;
    try {
      final opened = await _engine.openDocument(document.path);
      engineDocumentId = opened.documentId;
      if (!await _waitUntilAvailable(token)) return;
      final metadata = await _engine.readDocumentMetadata(opened.documentId);
      final coverIndex = (document.coverMetadata?.sourcePageIndex ?? 0).clamp(
        0,
        opened.pageCount - 1,
      );
      final frontMatterPages = <List<PdfStructuredTextBlock>>[];
      final structuredIndexes = <int>{
        coverIndex,
        for (var index = 0; index < math.min(4, opened.pageCount); index++)
          index,
      };
      for (final pageIndex in structuredIndexes.take(4)) {
        if (!await _waitUntilAvailable(token)) return;
        try {
          frontMatterPages.add(
            await _engine.extractStructuredPageText(
              documentId: opened.documentId,
              pageIndex: pageIndex,
              cancellationToken: token,
            ),
          );
        } on Object {
          // Metadata and sampled plain text remain valid fallbacks.
        }
      }
      final blocks = frontMatterPages.isEmpty
          ? const <PdfStructuredTextBlock>[]
          : frontMatterPages.first;
      final sampledText = BytesBuilder(copy: false);
      for (final pageIndex in _samplePageIndexes(
        opened.pageCount,
        coverIndex,
      )) {
        if (!await _waitUntilAvailable(token)) return;
        final text = await _engine.extractPageText(
          documentId: opened.documentId,
          pageIndex: pageIndex,
          cancellationToken: token,
        );
        if (text.trim().isEmpty) continue;
        final remaining = maximumTextBytes - sampledText.length;
        if (remaining <= 0) break;
        final encoded = utf8.encode(text);
        sampledText.add(encoded.take(remaining).toList(growable: false));
        if (sampledText.length < maximumTextBytes) {
          sampledText.addByte(0x0A);
        }
      }
      final result = LocalDocumentEnrichment.analyze(
        metadata: metadata,
        coverBlocks: blocks,
        sampledText: utf8.decode(sampledText.takeBytes(), allowMalformed: true),
        fileName: document.fileName,
        frontMatterPages: frontMatterPages,
      );
      final title = _resolvedCandidate(
        result.title,
        current: document.sourceTitle,
      );
      final author = _resolvedCandidate(
        result.author,
        current: document.sourceAuthor,
      );
      final review = title.suggestion != null || author.suggestion != null;
      _documents.saveEnrichment(
        document: document,
        analyzerVersion: analyzerVersion,
        status: review
            ? EnrichmentStatus.review
            : result.hasExtractableText ||
                  title.detected != null ||
                  author.detected != null
            ? EnrichmentStatus.completed
            : EnrichmentStatus.noText,
        detectedTitle: title.detected,
        detectedAuthor: author.detected,
        suggestedTitle: title.suggestion,
        suggestedAuthor: author.suggestion,
        titleConfidence: result.title?.confidence,
        authorConfidence: result.author?.confidence,
        automaticTopics: result.topics,
      );
    } on Object catch (error) {
      if (!token.isCancelled) {
        _documents.saveEnrichment(
          document: document,
          analyzerVersion: analyzerVersion,
          status: EnrichmentStatus.failed,
          errorCode: error.runtimeType.toString(),
        );
      }
    } finally {
      if (engineDocumentId != null) {
        await _engine.closeDocument(engineDocumentId);
      }
      if (identical(_operationToken, token)) _operationToken = null;
      notifyListeners();
    }
  }

  void pause(bool value) => _paused = value;

  /// Keeps enrichment suspended for the whole lifetime of an open reader.
  void setReaderPaused(bool value) => _readerPaused = value;

  /// Suspends enrichment during high-frequency viewport interaction without
  /// clearing the independent reader or manual pause reasons.
  void setInteractionPaused(bool value) => _interactionPaused = value;

  void cancel() {
    _cancelled = true;
    _operationToken?.cancel();
  }

  Future<bool> _waitUntilAvailable(PdfOperationToken token) async {
    while (isPaused && !token.isCancelled && !_cancelled) {
      await Future<void>.delayed(const Duration(milliseconds: 80));
    }
    return !token.isCancelled && !_cancelled;
  }

  @override
  void dispose() {
    cancel();
    super.dispose();
  }

  LibraryDocument? acceptSuggestions(String documentId) {
    final value = _documents.acceptEnrichmentSuggestions(documentId);
    notifyListeners();
    return value;
  }

  LibraryDocument? dismissSuggestions(String documentId) {
    final value = _documents.dismissEnrichmentSuggestions(documentId);
    notifyListeners();
    return value;
  }

  LibraryDocument? setManualTopics(
    String documentId,
    Set<LibraryTopic> topics,
  ) {
    final value = _documents.setManualTopics(
      documentId,
      topics,
      analyzerVersion: analyzerVersion,
    );
    notifyListeners();
    return value;
  }

  static ({String? detected, String? suggestion}) _resolvedCandidate(
    EnrichmentCandidate? candidate, {
    String? current,
  }) {
    if (candidate == null || _sameText(candidate.value, current)) {
      return (detected: null, suggestion: null);
    }
    if (candidate.confidence >= automaticConfidence) {
      return (detected: candidate.value, suggestion: null);
    }
    if (candidate.confidence >= suggestionConfidence) {
      return (detected: null, suggestion: candidate.value);
    }
    return (detected: null, suggestion: null);
  }

  static bool _sameText(String value, String? other) =>
      other != null && value.trim().toLowerCase() == other.trim().toLowerCase();

  static List<int> _samplePageIndexes(int pageCount, int coverIndex) {
    final indexes = <int>{coverIndex};
    for (var index = 0; index < math.min(8, pageCount); index++) {
      indexes.add(index);
    }
    if (pageCount > 8) {
      final remaining = maximumPages - indexes.length;
      for (var step = 1; step <= remaining; step++) {
        indexes.add(((pageCount - 1) * step / (remaining + 1)).round());
      }
    }
    return indexes.take(maximumPages).toList(growable: false)..sort();
  }
}

abstract final class LocalDocumentEnrichment {
  static LocalEnrichmentResult analyze({
    required PdfDocumentMetadata metadata,
    required List<PdfStructuredTextBlock> coverBlocks,
    required String sampledText,
    required String fileName,
    List<List<PdfStructuredTextBlock>> frontMatterPages = const [],
  }) {
    final cleanTitle = _plausibleMetadata(metadata.title, fileName: fileName);
    final cleanAuthor = _plausibleMetadata(metadata.author);
    final pages = frontMatterPages.isEmpty
        ? <List<PdfStructuredTextBlock>>[coverBlocks]
        : frontMatterPages;
    final visualTitle = _bestFrontMatterCandidate(pages, _coverTitle);
    final fileTitle = _filenameTitle(fileName);
    final title = cleanTitle == null
        ? visualTitle ?? fileTitle
        : EnrichmentCandidate(cleanTitle, 0.98);
    final visualAuthor = _bestFrontMatterCandidate(
      pages,
      (blocks) => _coverAuthor(blocks, title?.value),
    );
    final author = cleanAuthor == null
        ? visualAuthor
        : EnrichmentCandidate(cleanAuthor, 0.98);
    final combined = [
      if (title != null) '${title.value} ${title.value} ${title.value}',
      if (author != null) author.value,
      for (final page in pages)
        for (final block in page) block.text,
      sampledText,
    ].join('\n');
    return LocalEnrichmentResult(
      title: title,
      author: author,
      topics: _classifyTopics(combined),
      hasExtractableText:
          pages.any((page) => page.isNotEmpty) || sampledText.trim().isNotEmpty,
    );
  }

  static EnrichmentCandidate? _bestFrontMatterCandidate(
    List<List<PdfStructuredTextBlock>> pages,
    EnrichmentCandidate? Function(List<PdfStructuredTextBlock>) detect,
  ) {
    EnrichmentCandidate? best;
    for (var index = 0; index < pages.length; index++) {
      final candidate = detect(pages[index]);
      if (candidate == null) continue;
      final adjusted = EnrichmentCandidate(
        candidate.value,
        (candidate.confidence - index * 0.025).clamp(0, 1),
      );
      if (best == null || adjusted.confidence > best.confidence) {
        best = adjusted;
      }
    }
    return best;
  }

  static String? _plausibleMetadata(String? value, {String? fileName}) {
    final clean = _cleanLine(value);
    if (clean == null || clean.length < 3 || clean.length > 180) return null;
    final normalized = _normalize(clean);
    if (_noise.any(normalized.contains)) return null;
    if (fileName != null) {
      final stem = p.basenameWithoutExtension(fileName);
      if (_normalize(stem) == normalized && _looksGeneratedFileName(stem)) {
        return null;
      }
    }
    return clean;
  }

  static EnrichmentCandidate? _filenameTitle(String fileName) {
    final stem = p.basenameWithoutExtension(fileName).trim();
    if (_looksGeneratedFileName(stem)) return null;
    final clean = _cleanLine(
      stem
          .replaceAll(RegExp(r'[_-]+'), ' ')
          .replaceAll(RegExp(r'\s+\(\d+\)$'), ''),
    );
    if (clean == null || clean.length < 4 || clean.length > 140) return null;
    final normalized = _normalize(clean);
    if (_genericFileTitles.contains(normalized)) return null;
    return EnrichmentCandidate(clean, 0.58);
  }

  static EnrichmentCandidate? _coverTitle(List<PdfStructuredTextBlock> blocks) {
    if (blocks.isEmpty) return null;
    final candidates = blocks.where((block) {
      final text = _cleanLine(block.text);
      if (text == null || text.length < 3 || text.length > 180) return false;
      final normalized = _normalize(text);
      return !_noise.any(normalized.contains) &&
          !RegExp(r'^\d+$').hasMatch(normalized) &&
          !RegExp(
            r'^(?:por|by|autor|autora|written by)\b',
          ).hasMatch(normalized) &&
          !normalized.contains('http') &&
          !normalized.contains('www ');
    }).toList();
    if (candidates.isEmpty) return null;
    final maxHeight = candidates.map((block) => block.height).reduce(math.max);
    candidates.sort((left, right) {
      double score(PdfStructuredTextBlock block) {
        final height = maxHeight <= 0 ? 0 : block.height / maxHeight;
        final upperPage = (1 - block.relativeTop).clamp(0, 1);
        final blockCenter = block.left + block.width / 2;
        final centered = block.pageWidth <= 0
            ? 0.5
            : (1 - ((blockCenter / block.pageWidth) - 0.5).abs() * 2).clamp(
                0,
                1,
              );
        final length = block.text.trim().length;
        final usefulLength = length <= 100 ? 1.0 : 0.65;
        return height * 0.54 +
            upperPage * 0.18 +
            centered * 0.14 +
            usefulLength * 0.14;
      }

      return score(right).compareTo(score(left));
    });
    final selected = candidates.first;
    final heightRatio = maxHeight <= 0 ? 0.0 : selected.height / maxHeight;
    final companions = candidates.where((block) {
      if (identical(block, selected) || block.top <= selected.top) return false;
      final gap = block.top - (selected.top + selected.height);
      final clean = _cleanLine(block.text);
      return gap >= -2 &&
          gap <= selected.height * 1.4 &&
          block.height >= selected.height * 0.62 &&
          clean != null &&
          !_looksLikePerson(clean) &&
          selected.text.length + clean.length <= 180;
    }).toList()..sort((left, right) => left.top.compareTo(right.top));
    final title = [
      _cleanLine(selected.text)!,
      if (companions.isNotEmpty) _cleanLine(companions.first.text)!,
    ].join(' ');
    final confidence =
        (0.68 +
                heightRatio * 0.18 +
                ((1 - selected.relativeTop).clamp(0, 1)) * 0.08 +
                (companions.isNotEmpty ? 0.03 : 0))
            .clamp(0, 0.94)
            .toDouble();
    return EnrichmentCandidate(title, confidence);
  }

  static EnrichmentCandidate? _coverAuthor(
    List<PdfStructuredTextBlock> blocks,
    String? title,
  ) {
    for (final block in blocks) {
      final clean = _cleanLine(block.text);
      if (clean == null || clean == title) continue;
      final explicit = RegExp(
        r'^(?:by|por|de|autor(?:a)?|written by|edited by|editado por)\s*[:\-]?\s+(.+)$',
        caseSensitive: false,
      ).firstMatch(clean);
      if (explicit != null && _looksLikePerson(explicit.group(1)!)) {
        return EnrichmentCandidate(explicit.group(1)!.trim(), 0.88);
      }
    }
    final plausible =
        blocks.where((block) {
          final clean = _cleanLine(block.text);
          return clean != null &&
              _normalize(clean) != _normalize(title ?? '') &&
              _looksLikePerson(clean);
        }).toList()..sort((left, right) {
          final vertical = left.top.compareTo(right.top);
          return vertical != 0 ? vertical : right.height.compareTo(left.height);
        });
    if (plausible.isEmpty) return null;
    return EnrichmentCandidate(_cleanLine(plausible.first.text)!, 0.70);
  }

  static bool _looksLikePerson(String value) {
    final clean = value.trim();
    if (clean.length < 5 ||
        clean.length > 80 ||
        clean.contains(RegExp(r'\d'))) {
      return false;
    }
    final words = clean.split(RegExp(r'\s+'));
    if (words.length < 2 || words.length > 6) return false;
    final normalized = _normalize(clean);
    if (_noise.any(normalized.contains)) return false;
    const particles = {
      'de',
      'del',
      'la',
      'las',
      'los',
      'da',
      'das',
      'do',
      'dos',
      'van',
      'von',
      'y',
      'and',
    };
    for (var index = 0; index < words.length; index++) {
      final word = words[index];
      if (index > 0 && particles.contains(word.toLowerCase())) continue;
      if (!RegExp(
        r"^[A-ZÁÉÍÓÚÜÑÀÈÌÒÙÂÊÎÔÛÇ][\p{L}'’.-]*$",
        unicode: true,
      ).hasMatch(word)) {
        return false;
      }
    }
    return true;
  }

  static List<DocumentTopicAssignment> _classifyTopics(String text) {
    final normalized = _normalize(text);
    if (normalized.isEmpty) return const [];
    final scored = <({LibraryTopic topic, double score})>[];
    for (final entry in _topicTerms.entries) {
      var score = 0.0;
      var distinctTerms = 0;
      for (final term in entry.value) {
        final pattern = RegExp('(?:^|\\s)${RegExp.escape(term)}(?:\\s|\$)');
        final matches = pattern.allMatches(normalized).length.clamp(0, 4);
        if (matches == 0) continue;
        distinctTerms++;
        final specificity = term.contains(' ')
            ? 1.8
            : term.length >= 8
            ? 1.3
            : 1.0;
        score += matches * specificity;
      }
      if (score >= 2 && (distinctTerms >= 2 || score >= 4)) {
        scored.add((topic: entry.key, score: score));
      }
    }
    scored.sort((left, right) => right.score.compareTo(left.score));
    final strongest = scored.isEmpty ? 0.0 : scored.first.score;
    return [
      for (final item
          in scored.where((item) => item.score >= strongest * 0.28).take(3))
        DocumentTopicAssignment(
          topic: item.topic,
          confidence: (0.52 + item.score / 24).clamp(0.55, 0.96),
          origin: EnrichmentOrigin.automatic,
        ),
    ];
  }

  static String? _cleanLine(String? value) {
    final clean = value?.replaceAll(RegExp(r'\s+'), ' ').trim();
    return clean == null || clean.isEmpty ? null : clean;
  }

  static bool _looksGeneratedFileName(String value) => RegExp(
    r'\d{4,}|^[a-f0-9]{12,}$|^(?:scan|documento?|archivo|file|untitled)[ _-]*\d*$',
    caseSensitive: false,
  ).hasMatch(value);

  static String _normalize(String value) => value
      .toLowerCase()
      .replaceAll(RegExp(r'[áàäâ]'), 'a')
      .replaceAll(RegExp(r'[éèëê]'), 'e')
      .replaceAll(RegExp(r'[íìïî]'), 'i')
      .replaceAll(RegExp(r'[óòöô]'), 'o')
      .replaceAll(RegExp(r'[úùüû]'), 'u')
      .replaceAll('ñ', 'n')
      .replaceAll(RegExp(r'[^a-z0-9]+'), ' ')
      .replaceAll(RegExp(r'\s+'), ' ')
      .trim();

  static const _noise = <String>[
    'isbn',
    'copyright',
    'all rights reserved',
    'todos los derechos',
    'editorial',
    'publisher',
    'edition',
    'edicion',
    'contents',
    'indice',
    'untitled',
    'microsoft word',
    'acrobat distiller',
    'doi ',
  ];

  static const _genericFileTitles = <String>{
    'document',
    'documento',
    'archivo',
    'file',
    'scan',
    'untitled',
  };

  static const _topicTerms = <LibraryTopic, List<String>>{
    LibraryTopic.history: [
      'historia',
      'historico',
      'history',
      'historical',
      'siglo',
      'century',
      'guerra',
      'war',
      'archivo historico',
      'revolucion',
      'revolution',
      'medieval',
    ],
    LibraryTopic.politics: [
      'politica',
      'political',
      'politics',
      'gobierno',
      'government',
      'democracia',
      'democracy',
      'estado',
      'state',
      'elecciones',
      'elections',
      'parlamento',
      'parliament',
      'constitucional',
    ],
    LibraryTopic.philosophy: [
      'filosofia',
      'philosophy',
      'etica',
      'ethics',
      'metafisica',
      'metaphysics',
      'epistemologia',
      'ontology',
      'ontologia',
      'razon',
      'reason',
      'existencialismo',
    ],
    LibraryTopic.literature: [
      'novela',
      'novel',
      'poesia',
      'poetry',
      'cuento',
      'fiction',
      'literatura',
      'literature',
      'narrativa',
      'narrative',
      'drama',
    ],
    LibraryTopic.science: [
      'ciencia',
      'science',
      'fisica',
      'physics',
      'quimica',
      'chemistry',
      'biologia',
      'biology',
      'astronomia',
      'astronomy',
      'matematicas',
      'mathematics',
      'experimento',
    ],
    LibraryTopic.technology: [
      'tecnologia',
      'technology',
      'software',
      'programacion',
      'programming',
      'computer',
      'algoritmo',
      'algorithm',
      'inteligencia artificial',
      'artificial intelligence',
      'machine learning',
      'datos',
      'computing',
    ],
    LibraryTopic.economics: [
      'economia',
      'economics',
      'mercado',
      'market',
      'capital',
      'finanzas',
      'finance',
      'inflacion',
      'macroeconomia',
      'microeconomia',
      'investment',
      'inversion',
    ],
    LibraryTopic.society: [
      'sociedad',
      'society',
      'sociologia',
      'sociology',
      'cultura',
      'culture',
      'social',
      'comunidad',
      'antropologia',
      'anthropology',
      'genero',
      'inequality',
    ],
    LibraryTopic.psychology: [
      'psicologia',
      'psychology',
      'mente',
      'mind',
      'conducta',
      'behavior',
      'cognicion',
      'emotion',
      'cerebro',
      'brain',
      'cognitive',
      'terapia',
    ],
    LibraryTopic.art: [
      'arte',
      'art',
      'pintura',
      'painting',
      'musica',
      'music',
      'arquitectura',
      'architecture',
      'escultura',
      'sculpture',
      'fotografia',
      'cinema',
    ],
    LibraryTopic.religionMythology: [
      'religion',
      'religious',
      'dios',
      'god',
      'mitologia',
      'mythology',
      'teologia',
      'theology',
      'sagrado',
      'sacred',
      'biblia',
      'bible',
    ],
    LibraryTopic.law: [
      'derecho',
      'law',
      'legal',
      'ley',
      'juridico',
      'court',
      'tribunal',
      'constitucion',
      'jurisprudencia',
      'criminal',
      'legislacion',
      'legislation',
    ],
    LibraryTopic.education: [
      'educacion',
      'education',
      'pedagogia',
      'teaching',
      'aprendizaje',
      'learning',
      'escuela',
      'school',
      'didactica',
      'curriculum',
      'universidad',
      'university',
    ],
    LibraryTopic.health: [
      'salud',
      'health',
      'medicina',
      'medicine',
      'medical',
      'enfermedad',
      'disease',
      'clinical',
      'paciente',
      'patient',
      'anatomia',
      'public health',
    ],
    LibraryTopic.nature: [
      'naturaleza',
      'nature',
      'ecologia',
      'ecology',
      'medio ambiente',
      'environment',
      'clima',
      'climate',
      'biodiversidad',
      'biodiversity',
      'conservacion',
      'sustainability',
    ],
    LibraryTopic.biography: [
      'biografia',
      'biography',
      'memorias',
      'memoir',
      'vida de',
      'life of',
      'autobiografia',
      'diario',
      'diary',
      'correspondencia',
      'letters',
    ],
  };
}
