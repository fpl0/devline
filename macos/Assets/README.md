# DevKit icon

The production source is `DevKit.icon`: a native layered Icon Composer document. A single transparent monogram has explicit charcoal-on-warm-white and warm-white-on-charcoal fills, plus a tintable variant. Apple supplies the native enclosure and lighting.

`package.sh` compiles the document with Xcode's `actool`, includes `Assets.car` and the generated fallback `DevKit.icns`, and sets `CFBundleIconName=DevKit`. The earlier flattened `DevKit.png` and iconset remain design history; they are no longer packaged.

The transparent monogram was extracted using the built-in image generation tool, preserving the selected modular lowercase d. Prompt: "Extract the exact existing geometric lowercase d monogram as a flat black glyph on actual transparency. Preserve its proportions, the square transparent counter and narrow gap between loop and vertical spine. Remove the rounded-square background, all texture, lighting and shadows. Center the glyph on a 1024 by 1024 transparent canvas. No extra marks or text."

`native-light.png` and `native-dark.png` are exports from Apple's `ictool` for macOS Default and Dark renditions, not a static icon placed on different backgrounds. `light-dark-preview.png` compares these native renders at 16, 32, 64 and 128 pixels. The TintedDark rendition was also rendered for inspection.

Generate the contact sheet after exporting the renditions:

```sh
swift scripts/check-icon.swift macos/Assets/native-light.png macos/Assets/native-dark.png macos/Assets/light-dark-preview.png
```

The installed Dock appearance must also be checked after packaging because macOS caches app icons and independently chooses its icon appearance.

The menu bar uses `../Sources/DevKitApp/Resources/MenuBarMonogram.png`, a cropped rendition of the same monogram at 18 points (36 pixels at 2×). macOS supplies the menu bar tint.
