/*
 * FishFile - C Header
 * Fish Config (.fico) parser for TontooOS
 *
 * This header provides C bindings for the fishfile library.
 */

#ifndef FISHFILE_H
#define FISHFILE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ======================== */
/* Document                 */
/* ======================== */

/** Opaque handle to a FishDocument. */
typedef struct FishDocument FishDocument;

/**
 * Parse a .fico string.
 *
 * @param content UTF-8 string with fico content
 * @return handle pointer or NULL on error
 */
FishDocument* fishfile_parse(const char *content);

/**
 * Load a .fico file from disk.
 *
 * @param path file path
 * @return handle pointer or NULL on error
 */
FishDocument* fishfile_load(const char *path);

/**
 * Free a document.
 *
 * @param doc document handle
 */
void fishfile_free(FishDocument *doc);

/**
 * Serialize document to fico string.
 *
 * @param doc document handle
 * @return allocated string (must be freed with fishfile_free_string) or NULL
 */
char* fishfile_to_string(FishDocument *doc);

/**
 * Write document to file.
 *
 * @param doc document handle
 * @param path file path
 * @return 0 on success, negative on error
 */
int fishfile_save(FishDocument *doc, const char *path);

/* ======================== */
/* Value Access             */
/* ======================== */

/**
 * Get a string value by dot-path.
 *
 * @param doc document handle
 * @param path dot-separated path e.g. "system.theme"
 * @return allocated string or NULL if not found / not a string (free with fishfile_free_string)
 */
char* fishfile_get_string(FishDocument *doc, const char *path);

/**
 * Get an integer value by path.
 *
 * @param doc document handle
 * @param path dot-separated path
 * @param out output pointer
 * @return 1 if found and is integer, 0 otherwise
 */
int fishfile_get_int(FishDocument *doc, const char *path, int64_t *out);

/**
 * Get a float value by path.
 *
 * @param doc document handle
 * @param path dot-separated path
 * @param out output pointer
 * @return 1 if found and is number, 0 otherwise
 */
int fishfile_get_float(FishDocument *doc, const char *path, double *out);

/**
 * Get a bool value by path.
 *
 * @param doc document handle
 * @param path dot-separated path
 * @param out output pointer (1=true, 0=false)
 * @return 1 if found and is bool, 0 otherwise
 */
int fishfile_get_bool(FishDocument *doc, const char *path, int *out);

/**
 * Check if a path exists.
 *
 * @param doc document handle
 * @param path dot-separated path
 * @return 1 if exists, 0 otherwise
 */
int fishfile_contains(FishDocument *doc, const char *path);

/**
 * Set a string value (creates intermediate tables).
 *
 * @param doc document handle
 * @param path dot-separated path
 * @param value string value
 */
void fishfile_set_string(FishDocument *doc, const char *path, const char *value);

/**
 * Set an integer value.
 *
 * @param doc document handle
 * @param path dot-separated path
 * @param value integer value
 */
void fishfile_set_int(FishDocument *doc, const char *path, int64_t value);

/**
 * Set a float value.
 *
 * @param doc document handle
 * @param path dot-separated path
 * @param value float value
 */
void fishfile_set_float(FishDocument *doc, const char *path, double value);

/**
 * Set a bool value.
 *
 * @param doc document handle
 * @param path dot-separated path
 * @param value 1=true, 0=false
 */
void fishfile_set_bool(FishDocument *doc, const char *path, int value);

/**
 * Remove a value by path.
 *
 * @param doc document handle
 * @param path dot-separated path
 * @return 1 if removed, 0 if not found
 */
int fishfile_remove(FishDocument *doc, const char *path);

/**
 * Check if document is empty.
 *
 * @param doc document handle
 * @return 1 if empty, 0 otherwise
 */
int fishfile_is_empty(FishDocument *doc);

/**
 * Get number of top-level keys.
 *
 * @param doc document handle
 * @return count
 */
size_t fishfile_len(FishDocument *doc);

/**
 * Convert document to JSON (pretty).
 *
 * @param doc document handle
 * @return allocated JSON string (free with fishfile_free_string) or NULL
 */
char* fishfile_to_json(FishDocument *doc);

/**
 * Create document from JSON string.
 *
 * @param json JSON string (must be object at root)
 * @return handle or NULL on error
 */
FishDocument* fishfile_from_json(const char *json);

/* ======================== */
/* Memory Management        */
/* ======================== */

/**
 * Free a string returned by fishfile_* functions.
 *
 * @param ptr string to free
 */
void fishfile_free_string(char *ptr);

/**
 * Get library version string.
 *
 * @return version string (do NOT free)
 */
const char* fishfile_version(void);

/**
 * Get last error message (thread-local).
 *
 * @return error string or NULL if no error
 */
const char* fishfile_last_error(void);

#ifdef __cplusplus
}
#endif

#endif /* FISHFILE_H */
