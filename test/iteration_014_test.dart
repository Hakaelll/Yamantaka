import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/app/theme/app_components.dart';
import 'package:yamantaka/app/theme/app_theme.dart';
import 'package:yamantaka/app/theme/app_tokens.dart';
import 'package:yamantaka/app/ui/yama_components.dart';
import 'package:yamantaka/core/i18n/app_localizations.dart';
import 'package:yamantaka/features/settings/presentation/settings_screen.dart';

void main() {
  group('0.14 visual system contracts', () {
    test('uses one stable token scale for desktop controls and dialogs', () {
      expect(AppIconSizes.standard, 18);
      expect(AppControlHeights.desktop, AppSizes.desktopControlExtent);
      expect(AppControlHeights.touch, AppSizes.touchControlExtent);
      expect(AppBorders.focus, greaterThan(AppBorders.hairline));
      expect(AppSizes.dialogCompactWidth, lessThan(AppSizes.dialogWideWidth));
      expect(AppOpacity.faint, lessThan(AppOpacity.toneSurface));
      expect(AppCurves.standard, Curves.easeInOutCubic);
    });

    test('classifies every visual-review width deterministically', () {
      const widths = [
        640.0,
        800.0,
        1280.0,
        1600.0,
        1920.0,
        1920.0,
        2560.0,
        3440.0,
      ];
      expect(widths.map(AppBreakpoints.classify), [
        AppWindowClass.compact,
        AppWindowClass.medium,
        AppWindowClass.expanded,
        AppWindowClass.expanded,
        AppWindowClass.expanded,
        AppWindowClass.expanded,
        AppWindowClass.expanded,
        AppWindowClass.expanded,
      ]);
    });
  });

  group('0.14 settings taxonomy', () {
    test('keeps the eight user-facing categories in intentional order', () {
      expect(SettingsSectionId.values.map((value) => value.name), [
        'appearance',
        'reading',
        'accessibility',
        'library',
        'privacy',
        'storage',
        'recovery',
        'diagnostics',
      ]);
    });

    test('localizes every category and new dialog message in ES and EN', () {
      for (final language in AppLanguage.values) {
        final strings = YamantakaLocalizations(language);
        for (final section in SettingsSectionId.values) {
          final label = section.label(strings);
          expect(label, isNot(startsWith('settings.')));
          expect(label, isNotEmpty);
        }
        for (final key in const [
          'settings.accessibility.description',
          'settings.privacy.description',
          'settings.storage.description',
          'settings.recovery.description',
          'settings.diagnosticsSection.description',
          'backup.previewDescription',
          'backup.preBackupRequired',
        ]) {
          expect(strings.get(key), isNot(key));
        }
      }
    });
  });

  group('0.14 adaptive and accessible components', () {
    testWidgets('dialog stays reachable at 640x480 and 200 percent text', (
      tester,
    ) async {
      tester.view
        ..physicalSize = const Size(640, 480)
        ..devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);

      await tester.pumpWidget(
        _testApp(
          textScale: 2,
          home: Builder(
            builder: (context) => Center(
              child: FilledButton(
                onPressed: () => showDialog<void>(
                  context: context,
                  builder: (context) => AppDialogFrame(
                    icon: Icons.lock_outline_rounded,
                    title: 'Protected document',
                    description:
                        'The password is used in memory and is never stored.',
                    actions: [
                      TextButton(onPressed: () {}, child: const Text('Cancel')),
                      FilledButton(onPressed: () {}, child: const Text('Open')),
                    ],
                    content: const Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        TextField(
                          decoration: InputDecoration(
                            errorText: 'Wrong password',
                          ),
                        ),
                        SizedBox(height: AppSpacing.space16),
                        Text('The original PDF remains unchanged.'),
                      ],
                    ),
                  ),
                ),
                child: const Text('Show'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Show'));
      await tester.pumpAndSettle();

      expect(tester.takeException(), isNull);
      expect(
        find.text('The password is used in memory and is never stored.'),
        findsOneWidget,
      );
      expect(find.text('Wrong password'), findsOneWidget);
      expect(find.text('Cancel'), findsOneWidget);

      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pumpAndSettle();
      expect(find.text('Protected document'), findsNothing);
    });

    testWidgets('operation status exposes progress and cancellation', (
      tester,
    ) async {
      var cancelled = false;
      final semantics = tester.ensureSemantics();
      await tester.pumpWidget(
        _testApp(
          home: AppOperationStatus(
            label: 'Applying recipe',
            phase: 'Updating documents',
            value: 0.42,
            onCancel: () => cancelled = true,
            cancelLabel: 'Cancel',
          ),
        ),
      );

      expect(find.byType(LinearProgressIndicator), findsOneWidget);
      expect(find.text('42 %'), findsOneWidget);
      expect(
        find.bySemanticsLabel(RegExp('Applying recipe.*42 %')),
        findsOneWidget,
      );
      await tester.tap(find.text('Cancel'));
      expect(cancelled, isTrue);
      semantics.dispose();
    });

    testWidgets(
      'icon buttons define normal, hover, focus, selected and disabled',
      (tester) async {
        await tester.pumpWidget(
          _testApp(
            home: const Row(
              children: [
                YamaIconButton(
                  icon: Icons.add,
                  semanticLabel: 'Normal',
                  onPressed: _noop,
                ),
                YamaIconButton(
                  icon: Icons.check,
                  semanticLabel: 'Selected',
                  selected: true,
                  onPressed: _noop,
                ),
                YamaIconButton(
                  icon: Icons.block,
                  semanticLabel: 'Disabled',
                  onPressed: null,
                ),
              ],
            ),
          ),
        );

        final normal = tester.widget<IconButton>(find.byType(IconButton).at(0));
        final selected = tester.widget<IconButton>(
          find.byType(IconButton).at(1),
        );
        final disabled = tester.widget<IconButton>(
          find.byType(IconButton).at(2),
        );
        expect(
          normal.style!.backgroundColor!.resolve({WidgetState.hovered}),
          isNot(Colors.transparent),
        );
        expect(
          normal.style!.side!.resolve({WidgetState.focused})!.width,
          AppBorders.focus,
        );
        expect(
          selected.style!.backgroundColor!.resolve(const {}),
          isNot(Colors.transparent),
        );
        expect(
          disabled.style!.foregroundColor!.resolve({WidgetState.disabled}),
          isNot(normal.style!.foregroundColor!.resolve(const {})),
        );
      },
    );

    testWidgets(
      'shared dialog composes in RTL, high contrast and reduced motion',
      (tester) async {
        await tester.pumpWidget(
          _testApp(
            direction: TextDirection.rtl,
            highContrast: true,
            reduceMotion: true,
            home: const AppDialogFrame(
              icon: Icons.info_outline,
              title: 'عنوان',
              description: 'وصف',
              actions: [],
              content: AppOperationStatus(label: 'عملية', phase: 'قيد التنفيذ'),
            ),
          ),
        );
        expect(tester.takeException(), isNull);
        expect(
          Directionality.of(tester.element(find.text('عنوان'))),
          TextDirection.rtl,
        );
        expect(
          MediaQuery.highContrastOf(tester.element(find.text('وصف'))),
          isTrue,
        );
        expect(
          MediaQuery.disableAnimationsOf(tester.element(find.text('عملية'))),
          isTrue,
        );
      },
    );
  });
}

void _noop() {}

Widget _testApp({
  required Widget home,
  double textScale = 1,
  TextDirection direction = TextDirection.ltr,
  bool highContrast = false,
  bool reduceMotion = false,
}) => MaterialApp(
  debugShowCheckedModeBanner: false,
  theme: AppTheme.light(highContrast: highContrast),
  builder: (context, child) => MediaQuery(
    data: MediaQuery.of(context).copyWith(
      textScaler: TextScaler.linear(textScale),
      highContrast: highContrast,
      disableAnimations: reduceMotion,
    ),
    child: Directionality(textDirection: direction, child: child!),
  ),
  home: Scaffold(body: home),
);
