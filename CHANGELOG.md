# Changelog

## 0.1.0-alpha.1 - 2026-06-11

### Features

- Add a thread-safe `Storage` facade with driver registration, configuration, default driver selection, and fluent filesystem resolution.
- Add an object-safe adapter contract with async and sync operations for read, write, delete, exists, directory creation, listing, metadata, copy, and move.
- Add the native local filesystem driver as the default built-in driver behind the `native` feature.

### Safety

- Reject path traversal for local filesystem operations while allowing root-safe list, exists, and metadata calls.

### Testing

- Cover async and sync native filesystem flows, custom driver registration, default driver resolution, root listing, and path normalization.
