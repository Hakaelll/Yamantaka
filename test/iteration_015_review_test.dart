import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/theme/app_components.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/app/theme/app_tokens.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/core/logging/app_logger.dart';
import 'package:yamantaka/features/automations/application/local_automation_engine.dart';
import 'package:yamantaka/features/automations/domain/local_automation.dart';
import 'package:yamantaka/features/automations/presentation/automation_manager_dialog.dart';
import 'package:yamantaka/features/library/presentation/library_screen.dart';
import 'package:yamantaka/storage/database/app_database.dart';
import 'package:yamantaka/storage/repositories/automation_repository.dart';

void main() {
  testWidgets('actionable notices can close without invoking undo', (
    tester,
  ) async {
    var undone = false;
    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.light(),
        home: Scaffold(
          body: Builder(
            builder: (context) => TextButton(
              onPressed: () => ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: const Text('Setting updated'),
                  action: SnackBarAction(
                    label: 'Undo',
                    onPressed: () => undone = true,
                  ),
                ),
              ),
              child: const Text('Show notice'),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('Show notice'));
    await tester.pumpAndSettle();
    await tester.tap(find.byIcon(Icons.close));
    await tester.pumpAndSettle();
    expect(find.byType(SnackBar), findsNothing);
    expect(undone, isFalse);
    expect(tester.takeException(), isNull);
  });

  testWidgets('session details stay secondary until explicitly expanded', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: AppTheme.light(),
        home: Scaffold(
          body: Column(
            children: [
              ContinuityCenter(
                summary: const ContinuitySummary(pendingReadings: 1),
                hasActiveReading: false,
                hasRecentlyClosed: true,
                onOpenNext: () {},
                onResumeSession: () {},
                onDiscardContext: () {},
              ),
              const Expanded(child: Placeholder()),
            ],
          ),
        ),
      ),
    );
    expect(
      tester.getSize(find.byType(ContinuityCenter)).height,
      lessThanOrEqualTo(56),
    );
    expect(find.byKey(const Key('continuity-resume')), findsNothing);
    await tester.tap(find.byType(ExpansionTile));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('continuity-resume')), findsOneWidget);
    expect(find.text('pendientes: 1'), findsOneWidget);
    expect(find.text('1 pendientes'), findsNothing);
    expect(tester.takeException(), isNull);
  });

  testWidgets(
    'shared menu stacks its shortcut at 200 percent without clipping',
    (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: AppTheme.light(),
          home: Scaffold(
            body: MediaQuery(
              data: const MediaQueryData(textScaler: TextScaler.linear(2)),
              child: const Center(
                child: SizedBox(
                  width: 240,
                  child: AppMenuItemContent(
                    icon: Icons.file_open_outlined,
                    label: 'Open document',
                    shortcut: 'Ctrl+O',
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      expect(tester.takeException(), isNull);
      final label = tester.getRect(find.text('Open document'));
      final shortcut = tester.getRect(find.text('Ctrl+O'));
      expect(shortcut.top, greaterThanOrEqualTo(label.bottom));
      final paragraph = tester.renderObject<RenderParagraph>(
        find.text('Open document'),
      );
      expect(paragraph.didExceedMaxLines, isFalse);
    },
  );

  testWidgets(
    'local rules use readable localized descriptions in both themes',
    (tester) async {
      tester.view.physicalSize = const Size(640, 480);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final database = AppDatabase.inMemory(
        AppLogger(minimumLevel: LogLevel.error),
      );
      addTearDown(database.close);
      final repository = AutomationRepository(database);
      final now = DateTime.utc(2026, 9, 6);
      repository.save(
        LocalAutomationRule(
          id: '',
          name: 'Regla de prueba',
          trigger: const AutomationTrigger(
            AutomationTriggerKind.extension,
            value: '.pdf',
          ),
          actions: const [
            AutomationAction(AutomationActionKind.addTag, value: 'lectura'),
          ],
          createdAt: now,
          updatedAt: now,
        ),
      );
      for (final locale in ['es', 'en']) {
        for (final dark in [false, true]) {
          await tester.pumpWidget(
            MaterialApp(
              key: ValueKey('$locale-$dark'),
              locale: Locale(locale),
              supportedLocales: const [Locale('es'), Locale('en')],
              localizationsDelegates: const [
                YamantakaLocalizations.delegate,
                GlobalMaterialLocalizations.delegate,
                GlobalWidgetsLocalizations.delegate,
                GlobalCupertinoLocalizations.delegate,
              ],
              theme: dark ? AppTheme.dark() : AppTheme.light(),
              builder: (context, child) => MediaQuery(
                data: MediaQuery.of(
                  context,
                ).copyWith(textScaler: const TextScaler.linear(2)),
                child: child!,
              ),
              home: Scaffold(
                body: AutomationManagerDialog(
                  repository: repository,
                  engine: LocalAutomationEngine(repository),
                  documents: const [],
                  applyAction: (_, _) async => const {},
                  applyUndo: (_) async {},
                ),
              ),
            ),
          );
          await tester.pumpAndSettle();
          expect(tester.takeException(), isNull);
          expect(
            find.text(
              locale == 'es'
                  ? 'Extensión .pdf · Aplicar etiqueta: lectura'
                  : 'Extension .pdf · Apply tag: lectura',
            ),
            findsOneWidget,
          );
          expect(find.textContaining('addTag'), findsNothing);
          final paragraph = tester.renderObject<RenderParagraph>(
            find.text('Regla de prueba'),
          );
          final colors = tester.element(find.text('Regla de prueba')).appColors;
          expect(paragraph.text.style?.color, colors.textPrimary);
        }
      }
    },
  );
}
