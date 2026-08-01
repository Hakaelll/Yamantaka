import 'app_failure.dart';

enum FailureAction { retry, chooseAnotherFile, enterPassword, goBack, dismiss }

final class FailurePresentation {
  const FailurePresentation({
    required this.title,
    required this.message,
    required this.action,
    this.isCancellation = false,
  });

  final String title;
  final String message;
  final FailureAction action;
  final bool isCancellation;
}

/// Complete, private-safe copy for each typed failure. The UI can choose the
/// action without exposing native exceptions or paths in a banner.
abstract final class FailurePresentations {
  static FailurePresentation forFailure(AppFailure? failure) {
    if (failure == null) {
      return const FailurePresentation(
        title: 'No hay un documento abierto',
        message: 'Elige un PDF local para comenzar a leer.',
        action: FailureAction.goBack,
      );
    }
    return switch (failure.kind) {
      FailureKind.fileNotFound => const FailurePresentation(
        title: 'El archivo ya no está disponible',
        message: 'Puedes localizar otra copia del mismo PDF o volver atrás.',
        action: FailureAction.chooseAnotherFile,
      ),
      FailureKind.accessDenied => const FailurePresentation(
        title: 'No se puede leer el archivo',
        message: 'Comprueba los permisos del archivo y vuelve a intentarlo.',
        action: FailureAction.retry,
      ),
      FailureKind.corruptPdf => const FailurePresentation(
        title: 'El PDF no se puede abrir',
        message:
            'El archivo puede estar dañado o usar una variante no compatible.',
        action: FailureAction.chooseAnotherFile,
      ),
      FailureKind.emptyPdf => const FailurePresentation(
        title: 'El PDF no contiene páginas',
        message: 'Elige un documento que tenga al menos una página.',
        action: FailureAction.chooseAnotherFile,
      ),
      FailureKind.passwordRequired => const FailurePresentation(
        title: 'El PDF está protegido',
        message: 'Introduce la contraseña para continuar. No se guardará.',
        action: FailureAction.enterPassword,
      ),
      FailureKind.wrongPassword => const FailurePresentation(
        title: 'La contraseña no es correcta',
        message: 'Vuelve a introducirla o elige otro documento.',
        action: FailureAction.enterPassword,
      ),
      FailureKind.unsupportedFormat => const FailurePresentation(
        title: 'Formato no compatible',
        message: 'Yamantaka solo abre archivos PDF.',
        action: FailureAction.chooseAnotherFile,
      ),
      FailureKind.renderFailed => const FailurePresentation(
        title: 'No se pudo dibujar la página',
        message: 'Puedes reintentar sin modificar el PDF original.',
        action: FailureAction.retry,
      ),
      FailureKind.outOfMemory => const FailurePresentation(
        title: 'No hay memoria suficiente',
        message:
            'Cierra otros documentos o reduce el trabajo en segundo plano.',
        action: FailureAction.retry,
      ),
      FailureKind.databaseFailed => const FailurePresentation(
        title: 'No se pudo leer la biblioteca',
        message:
            'Verifica la base local desde el diagnóstico y vuelve a intentarlo.',
        action: FailureAction.retry,
      ),
      FailureKind.thumbnailFailed => const FailurePresentation(
        title: 'No se pudo generar la portada',
        message:
            'La lectura continúa y la portada se puede reintentar después.',
        action: FailureAction.dismiss,
      ),
      FailureKind.restoreFailed => const FailurePresentation(
        title: 'No se pudo restaurar la sesión',
        message:
            'La biblioteca sigue disponible; puedes abrir el PDF manualmente.',
        action: FailureAction.goBack,
      ),
      FailureKind.cancelled => const FailurePresentation(
        title: 'Operación cancelada',
        message: 'No se ha aplicado ningún resultado obsoleto.',
        action: FailureAction.dismiss,
        isCancellation: true,
      ),
      FailureKind.invalidInput => const FailurePresentation(
        title: 'Entrada no válida',
        message: 'Revisa los datos e inténtalo de nuevo.',
        action: FailureAction.retry,
      ),
      FailureKind.unauthorizedLink => const FailurePresentation(
        title: 'Enlace bloqueado',
        message: 'El esquema del enlace no está autorizado por Yamantaka.',
        action: FailureAction.dismiss,
      ),
      FailureKind.unknown => const FailurePresentation(
        title: 'No se pudo completar la operación',
        message:
            'Los datos confirmados se conservan. Puedes volver a intentarlo.',
        action: FailureAction.retry,
      ),
    };
  }
}
