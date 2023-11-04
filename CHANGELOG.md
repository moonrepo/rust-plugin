# Changelog

## 0.4.0

#### 🚀 Updates

- Updated to support proto v0.22 release.

#### ⚙️ Internal

- Updated dependencies.

## 0.3.1

#### 🐞 Fixes

- Fixed an issue where `RUSTUP_HOME` would not respect virtual paths.

## 0.3.0

#### 🚀 Updates

- Will now attempt to install `rustup` if it does not exist on the current machine.
- Updated to support proto v0.20 release.

#### 🐞 Fixes

- Will now respect the `RUSTUP_HOME` environment variable when locating the `.rustup` store.
- Fixed an issue where `sync_manifest` would fail if we didn't have read access to the rustup store.

#### ⚙️ Internal

- Updated dependencies.

## 0.2.3

#### 🐞 Fixes

- Fixed the "bin not found" errors by uninstalling the toolchain before installing it, if we've detected that it's in a broken state.

## 0.2.2

#### 🐞 Fixes

- Temporary hack for "bin not found" errors.

## 0.2.1

#### 🐞 Fixes

- Switched to `cargo` from `rustc` for bin detection.
- Slightly improved logic that detects an installation.

#### ⚙️ Internal

- Updated dependencies.

## 0.2.0

#### 🚀 Updates

- Added support for installing the canary release (via nightly).
- Updated to support proto v0.17 release.

## 0.1.2

#### 🚀 Updates

- Updated to support proto v0.16 release.

## 0.1.1

#### 🐞 Fixes

- Fixed `rustup` detection on Windows.

## 0.1.0

#### 🚀 Updates

- Added uninstall support (uses `rustup toolchain uninstall`).
- Added support for `install_global` and `uninstall_global`.
- Updated to support proto v0.15 release.

#### 🐞 Fixes

- Fixed an issue where the globals directory for `CARGO_INSTALL_ROOT` was incorrect.

## 0.0.3

#### 🐞 Fixes

- Fixed an issue where the `rustc` binary wasn't properly located on Windows.

## 0.0.2

#### 🚀 Updates

- Supports automatic manifest syncing (keeps track of installed versions).

## 0.0.1

#### 🎉 Release

- Initial release!
