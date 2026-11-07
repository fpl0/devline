# Third-party notices

This project vendors the Cargo packages identified by the checksums and versions in `Cargo.lock`. `vendor/` preserves the original source and each package's applicable license and notice files.

The initial snapshot includes [Tauri 2.12.5](https://github.com/tauri-apps/tauri/releases/tag/tauri-v2.12.5), [tauri-build 2.7.4](https://github.com/tauri-apps/tauri/releases/tag/tauri-build-v2.7.4), and their locked dependencies. Tauri's MIT and Apache 2.0 licenses are in `vendor/tauri/LICENSE-MIT` and `vendor/tauri/LICENSE-APACHE-2.0`. The native WebKit runtime is part of macOS.

## Mozilla Public License 2.0

The locked `cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`, and `selectors` packages contain MPL 2.0 source. Their exact source is included in the corresponding `vendor/` directories and identified by `Cargo.lock`. The license text is included in [MPL-2.0.txt](MPL-2.0.txt). The `selectors` crate retains its source notices and comes from the [pinned Stylo source](https://github.com/servo/stylo/tree/572ecba2d1600e7c3d490586692a209faf703baa/selectors).

If you redistribute a built app separately, give recipients access to the corresponding MPL-covered source from your project’s `vendor/` directory. See [MPL 2.0, section 3.2](https://www.mozilla.org/en-US/MPL/2.0/#distribution-of-executable-form).
