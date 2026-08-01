# Metadata local y notas de biblioteca

Yamantaka conserva tres capas separadas para cada documento:

- `source_title` y `source_author` representan lo detectado o importado del PDF.
- `title_override` y `author_override` son correcciones locales que dominan la
  presentación, la biblioteca y el índice sin tocar el archivo.
- `document_notes` contiene una nota local opcional de hasta 4.000 caracteres.

La nota se guarda en SQLite (esquema 6), se incluye en el backup local v5, se
puede buscar desde biblioteca/índice y se conserva al reabrir o fusionar
registros. Los campos vacíos se normalizan a `NULL`, y los límites se validan
antes de escribir. La fusión de duplicados aplica la misma elección de conflicto
que título y autor, informa `library_note` cuando hay diferencias y nunca
modifica, mueve ni elimina un PDF.
