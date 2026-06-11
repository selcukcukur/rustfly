# Changelog

## Unreleased

### Features

- Add `Filesystem::size` and `Filesystem::last_modified` with sync variants.
- Add `Storage::size` and `Storage::last_modified` with sync variants.
- Add UTF-8 `read_string` and `get_string` shortcuts with sync variants.
- Add `append` and `prepend` content shortcuts with sync variants.
- Add `missing` path-check shortcuts with sync variants.
- Normalize storage paths with platform-independent `/` and `\` separators for native storage.

### Testing

- Cover file size and last-modified shortcuts against an isolated native driver.
- Cover UTF-8 string reads against an isolated native driver.
- Cover append and prepend flows against an isolated native driver.
- Cover missing path checks against an isolated native driver.
- Cover Windows-style separators and absolute path rejection in portable storage paths.

## 0.1.0-alpha.6 - 2026-06-12

### Features

- Add default recursive listing behavior to the adapter contract.
- Add `list_recursive`, `all_files`, and `all_directories` to `Filesystem`.
- Add `Storage::list_recursive`, `Storage::all_files`, and `Storage::all_directories` with sync variants.

### Testing

- Cover recursive file and directory listing against an isolated native driver.

## 0.1.0-alpha.5 - 2026-06-12

### Features

- Add `Metadata::is_file` and `Metadata::is_directory` helpers.
- Add `Filesystem::files` and `Filesystem::directories` with sync variants.
- Add `Storage::files` and `Storage::directories` with sync variants.

### Testing

- Cover file and directory listing filters against an isolated native driver.

## 0.1.0-alpha.4 - 2026-06-12

### Features

- Add typed `StorageConfig` builders and getters for path, bool, and integer values.
- Add storage driver registry inspection with `Storage::has_driver` and `Storage::driver_names`.
- Add replaceable driver registration with `Storage::extend_or_replace`.

### Testing

- Cover typed config composition and driver registry replacement behavior.

## 0.1.0-alpha.3 - 2026-06-12

### Features

- Add Laravel-style default storage shortcuts such as `Storage::get`, `Storage::put`, `Storage::exists`, `Storage::list`, `Storage::metadata`, `Storage::copy`, and `Storage::move_file`.
- Add sync counterparts for default storage shortcuts, including `Storage::get_sync` and `Storage::put_sync`.

### Changed

- Continue alpha semver releases with `0.1.0-alpha.3`.

## 0.1.0-alpha.2 - 2026-06-11

### Features

- Add `rustfly-core` as the shared adapter contract crate for third-party and first-party drivers.
- Add `rustfly-inmemory` as the first standalone driver crate.
- Register the in-memory driver automatically as `memory` and `inmemory` when the `inmemory` feature is enabled.

### Changed

- Re-export core contract, metadata, error, and path types from `rustfly` while keeping external drivers cycle-free.

## 0.1.0-alpha.1 - 2026-06-11

### Features

- Add a thread-safe `Storage` facade with driver registration, configuration, default driver selection, and fluent filesystem resolution.
- Add an object-safe adapter contract with async and sync operations for read, write, delete, exists, directory creation, listing, metadata, copy, and move.
- Add the native local filesystem driver as the default built-in driver behind the `native` feature.

### Safety

- Reject path traversal for local filesystem operations while allowing root-safe list, exists, and metadata calls.

### Testing

- Cover async and sync native filesystem flows, custom driver registration, default driver resolution, root listing, and path normalization.
