# FFI

C bindings for FishFile via `Headers/fishfile.h`. The library is built as `cdylib` (`crate-type = ["cdylib", "rlib"]`), so C/C++ programs can link against it.

## Header

`Headers/fishfile.h` – include with `#include "fishfile.h"`.

Opaque type:

```c
typedef struct FishDocument FishDocument;
```

## Document

| Function | Signature | Description | Return |
|---|---|---|---|
| `fishfile_parse` | `FishDocument* fishfile_parse(const char *content)` | Parse fico string | handle or `NULL` on error |
| `fishfile_load` | `FishDocument* fishfile_load(const char *path)` | Load file | handle or `NULL` |
| `fishfile_free` | `void fishfile_free(FishDocument *doc)` | Free document | – |
| `fishfile_to_string` | `char* fishfile_to_string(FishDocument *doc)` | Serialize to fico | allocated string (free with `fishfile_free_string`) |
| `fishfile_save` | `int fishfile_save(FishDocument *doc, const char *path)` | Write file | `0` ok, negative error |
| `fishfile_version` | `const char* fishfile_version(void)` | Library version | DO NOT free |
| `fishfile_last_error` | `const char* fishfile_last_error(void)` | Last thread-local error | string or `NULL` |

## Value Access

| Function | Signature | Description |
|---|---|---|
| `fishfile_get_string` | `char* fishfile_get_string(doc, path)` | Get string by dot-path; allocated, free with `fishfile_free_string`, or `NULL` |
| `fishfile_get_int` | `int fishfile_get_int(doc, path, int64_t *out)` | Get integer; `1` found, `0` not |
| `fishfile_get_float` | `int fishfile_get_float(doc, path, double *out)` | Get float/number; `1` found |
| `fishfile_get_bool` | `int fishfile_get_bool(doc, path, int *out)` | Get bool; `1` found |
| `fishfile_contains` | `int fishfile_contains(doc, path)` | Check existence; `1` yes |
| `fishfile_set_string` | `void fishfile_set_string(doc, path, value)` | Set string (creates tables) |
| `fishfile_set_int` | `void fishfile_set_int(doc, path, int64_t value)` | Set integer |
| `fishfile_set_float` | `void fishfile_set_float(doc, path, double value)` | Set float |
| `fishfile_set_bool` | `void fishfile_set_bool(doc, path, int value)` | Set bool (`0`/`1`) |
| `fishfile_remove` | `int fishfile_remove(doc, path)` | Remove; `1` removed |
| `fishfile_is_empty` | `int fishfile_is_empty(doc)` | Empty check |
| `fishfile_len` | `size_t fishfile_len(doc)` | Top-level count |
| `fishfile_to_json` | `char* fishfile_to_json(doc)` | JSON pretty; free with `fishfile_free_string` |
| `fishfile_from_json` | `FishDocument* fishfile_from_json(const char *json)` | From JSON |

## Memory Rules

| Owner | Must free? | How |
|---|---|---|
| `FishDocument*` from `fishfile_parse`/`load`/`from_json` | Yes | `fishfile_free(doc)` |
| `char*` from `fishfile_to_string` / `to_json` / `get_string` | Yes | `fishfile_free_string(ptr)` |
| `const char*` from `fishfile_version` / `fishfile_last_error` | No | – |
| Input `const char*` params | No | caller owned |

## Example

```c
#include "fishfile.h"
#include <stdio.h>

int main(void) {
    FishDocument *doc = fishfile_parse(
        "system { theme: dark accent: blue }"
    );
    if (!doc) {
        fprintf(stderr, "parse error: %s\n", fishfile_last_error());
        return 1;
    }

    char *theme = fishfile_get_string(doc, "system.theme");
    if (theme) {
        printf("theme=%s\n", theme);
        fishfile_free_string(theme);
    }

    fishfile_set_string(doc, "system.theme", "light");

    char *out = fishfile_to_string(doc);
    if (out) {
        printf("%s\n", out);
        fishfile_free_string(out);
    }

    fishfile_save(doc, "/tmp/out.fico");
    fishfile_free(doc);
    return 0;
}
```

> **Note:** The current crate ships the header but does not yet expose C-ABI symbols – the header documents the intended stable ABI for a future `ffi.rs` implementation. Rust users should use the Rust API (`FishDocument`, etc.).

## Cross References

- [Document.md](Document.md) – Rust document API
- [Value.md](Value.md) – value types
- [Error.md](Error.md) – error handling
