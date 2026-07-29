import 'package:flutter/foundation.dart';

import '../../reader/application/reader_controller.dart';
import '../../reader/domain/reader_models.dart';
import '../domain/reading_profile.dart';

final class ReadingProfilesController extends ChangeNotifier {
  ReadingProfilesController(this.reader);

  final ReaderController reader;
  List<ReadingProfile> profiles = BuiltinReadingProfiles.all;
  String defaultProfileId = 'builtin-reading';

  void apply(String id) {
    final profile = profiles.where((item) => item.id == id).firstOrNull;
    final preferences = reader.preferences;
    if (profile == null || preferences == null) return;
    final applied = profile.apply(preferences);
    reader
      ..setTheme(applied.readingTheme)
      ..setMargins(applied.horizontalMargin)
      ..setPageLayout(applied.pageLayout);
    switch (applied.zoomMode) {
      case ZoomMode.fitWidth:
        reader.fitWidth();
      case ZoomMode.fitPage:
        reader.fitPage();
      case ZoomMode.custom:
        break;
    }
    notifyListeners();
  }

  void setDefault(String id) {
    if (!profiles.any((profile) => profile.id == id)) return;
    defaultProfileId = id;
    notifyListeners();
  }
}
