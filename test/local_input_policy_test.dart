import 'package:flutter_test/flutter_test.dart';
import 'package:yamantaka/core/errors/app_failure.dart';
import 'package:yamantaka/core/security/local_input_policy.dart';

void main() {
  test('only explicit external schemes are authorized', () {
    expect(
      LocalInputPolicy.authorizedExternalUri(Uri.parse('https://example.test')),
      isNotNull,
    );
    expect(
      LocalInputPolicy.authorizedExternalUri(Uri.parse('file:///secret.pdf')),
      isNull,
    );
    expect(
      () => LocalInputPolicy.requireAuthorizedExternalUri(
        Uri.parse('javascript:alert(1)'),
      ),
      throwsA(isA<AppFailure>()),
    );
  });

  test('password and path bounds produce typed failures', () {
    expect(
      () => LocalInputPolicy.boundedPassword('x' * 513),
      throwsA(isA<AppFailure>()),
    );
    expect(() => LocalInputPolicy.boundedPath(' '), throwsA(isA<AppFailure>()));
  });
}
