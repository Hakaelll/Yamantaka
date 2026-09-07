import 'dart:convert';
import 'dart:io';

import 'package:integration_test/integration_test_driver.dart';

Future<void> main() => integrationDriver(
  responseDataCallback: (data) async {
    if (data == null) throw StateError('Native app flow returned no evidence.');
    final report = File('build/verification/native-app-flow.json');
    await report.parent.create(recursive: true);
    await report.writeAsString(
      const JsonEncoder.withIndent('  ').convert(data),
      flush: true,
    );
  },
);
