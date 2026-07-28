import 'dart:async';
import 'dart:io';

import 'app/app_config.dart';
import 'app/bootstrap.dart';

Future<void> main(List<String> arguments) async {
  await runZonedGuarded(
    () =>
        bootstrap(arguments.isEmpty ? Platform.executableArguments : arguments),
    (error, stackTrace) {
      stderr.writeln('${AppConfig.productName} could not start: $error');
      stderr.writeln(stackTrace);
    },
  );
}
