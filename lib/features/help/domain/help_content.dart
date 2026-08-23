import '../../../core/i18n/app_localizations.dart';

enum HelpTopic {
  gettingStarted,
  privacy,
  backup,
  search,
  annotations,
  workspaces,
  shortcuts,
}

final class HelpArticle {
  const HelpArticle({
    required this.topic,
    required this.title,
    required this.summary,
    required this.body,
    this.keywords = const [],
  });

  final HelpTopic topic;
  final String title;
  final String summary;
  final String body;
  final List<String> keywords;

  bool matches(String query) {
    final needle = query.trim().toLowerCase();
    if (needle.isEmpty) return true;
    return [
      title,
      summary,
      body,
      ...keywords,
    ].join(' ').toLowerCase().contains(needle);
  }
}

abstract final class HelpContent {
  static List<HelpArticle> articles(AppLanguage language) =>
      language == AppLanguage.english ? _english : _spanish;

  static List<HelpArticle> search(AppLanguage language, String query) => [
    for (final article in articles(language))
      if (article.matches(query)) article,
  ];

  static const _spanish = <HelpArticle>[
    HelpArticle(
      topic: HelpTopic.gettingStarted,
      title: 'Empezar a leer',
      summary: 'Abre un PDF local y vuelve siempre a tu posición.',
      body:
          'Usa Abrir para elegir un PDF. La biblioteca conserva el progreso y '
          'la última posición sin copiar ni modificar el archivo original. '
          'También puede proponer título, autor y temas mediante metadatos y '
          'texto local; los escaneos sin texto conservan el nombre de archivo.',
      keywords: ['biblioteca', 'abrir', 'PDF'],
    ),
    HelpArticle(
      topic: HelpTopic.privacy,
      title: 'Privacidad local',
      summary: 'No hay cuentas, nube, telemetría ni OCR.',
      body:
          'La biblioteca, el índice, las anotaciones y las preferencias viven '
          'en este dispositivo. Los enlaces externos piden confirmación y los '
          'diagnósticos excluyen rutas, títulos y texto libre.',
      keywords: ['privacidad', 'red', 'datos'],
    ),
    HelpArticle(
      topic: HelpTopic.backup,
      title: 'Copias y restauración',
      summary: 'Exporta datos locales sin incluir el contenido PDF.',
      body:
          'Una copia contiene metadata, notas de biblioteca, progreso, marcadores, anotaciones y '
          'ajustes. Antes de importar se crea un respaldo de seguridad. Los '
          'PDF originales nunca entran en la copia ni se eliminan.',
      keywords: ['backup', 'copia', 'restaurar'],
    ),
    HelpArticle(
      topic: HelpTopic.search,
      title: 'Búsqueda e índice',
      summary: 'Busca dentro del documento o en tu biblioteca local.',
      body:
          'Ctrl+F busca en el PDF abierto. La búsqueda global es opcional y '
          'construye un índice local cancelable; los escaneos sin texto no '
          'activan OCR.',
      keywords: ['buscar', 'índice', 'texto'],
    ),
    HelpArticle(
      topic: HelpTopic.annotations,
      title: 'Anotaciones',
      summary: 'Tus notas se guardan fuera del PDF.',
      body:
          'Resaltados, marcadores y comentarios se almacenan localmente y '
          'pueden exportarse de forma explícita. Cambiar de archivo no mezcla '
          'anotaciones entre documentos.',
      keywords: ['notas', 'resaltado', 'anotaciones'],
    ),
    HelpArticle(
      topic: HelpTopic.workspaces,
      title: 'Espacios de trabajo',
      summary: 'Guarda una sesión de lectura discreta y recuperable.',
      body:
          'Un espacio conserva IDs, posiciones y paneles. Si un archivo falta, '
          'la recuperación es parcial y explicada; nunca se escanea ni se '
          'copia una unidad completa.',
      keywords: ['workspace', 'sesión', 'panel'],
    ),
    HelpArticle(
      topic: HelpTopic.shortcuts,
      title: 'Atajos esenciales',
      summary: 'Usa el teclado para las acciones principales.',
      body:
          'Ctrl+O abre, Ctrl+F busca, Ctrl+K muestra la paleta, F11 alterna '
          'pantalla completa y Escape cierra la capa temporal superior. En el '
          'lector puedes elegir Una página, Dos páginas o Continua desde '
          'Ctrl+K y ajustar el emparejado del pliego desde Más opciones.',
      keywords: ['teclado', 'atajos', 'Ctrl'],
    ),
  ];

  static const _english = <HelpArticle>[
    HelpArticle(
      topic: HelpTopic.gettingStarted,
      title: 'Start reading',
      summary: 'Open a local PDF and return to your reading position.',
      body:
          'Use Open to choose a PDF. The library keeps progress and the last '
          'position without copying or modifying the original file. It can '
          'also suggest title, author, and topics from local metadata and text; '
          'scans without text keep the file name.',
      keywords: ['library', 'open', 'PDF'],
    ),
    HelpArticle(
      topic: HelpTopic.privacy,
      title: 'Local privacy',
      summary: 'No accounts, cloud, telemetry, or OCR.',
      body:
          'The library, index, annotations, and preferences stay on this '
          'device. External links require confirmation and diagnostics exclude '
          'paths, titles, and free text.',
      keywords: ['privacy', 'network', 'data'],
    ),
    HelpArticle(
      topic: HelpTopic.backup,
      title: 'Backups and restore',
      summary: 'Export local data without including PDF content.',
      body:
          'A backup contains metadata, library notes, progress, bookmarks, annotations, and '
          'settings. Import creates a safety backup first. Original PDFs never '
          'enter the backup or get deleted.',
      keywords: ['backup', 'restore'],
    ),
    HelpArticle(
      topic: HelpTopic.search,
      title: 'Search and indexing',
      summary: 'Search the open document or your local library.',
      body:
          'Ctrl+F searches the open PDF. Global search is optional and builds a '
          'cancellable local index; scanned documents do not trigger OCR.',
      keywords: ['search', 'index', 'text'],
    ),
    HelpArticle(
      topic: HelpTopic.annotations,
      title: 'Annotations',
      summary: 'Your notes are stored outside the PDF.',
      body:
          'Highlights, bookmarks, and comments are stored locally and can be '
          'exported explicitly. Changing files never mixes annotations.',
      keywords: ['notes', 'highlight', 'annotations'],
    ),
    HelpArticle(
      topic: HelpTopic.workspaces,
      title: 'Workspaces',
      summary: 'Save a quiet, recoverable reading session.',
      body:
          'A workspace stores IDs, positions, and panels. Missing files produce '
          'a partial, explained recovery; the whole drive is never scanned.',
      keywords: ['workspace', 'session', 'panel'],
    ),
    HelpArticle(
      topic: HelpTopic.shortcuts,
      title: 'Essential shortcuts',
      summary: 'Use the keyboard for primary actions.',
      body:
          'Ctrl+O opens, Ctrl+F searches, Ctrl+K opens the command palette, '
          'F11 toggles fullscreen, and Escape closes the top temporary layer. '
          'Ctrl+K exposes One page, Two pages, and Continuous; More options '
          'contains spread pairing adjustments.',
      keywords: ['keyboard', 'shortcuts', 'Ctrl'],
    ),
  ];
}
