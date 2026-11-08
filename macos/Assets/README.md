# Devline app icon

Edit [`Devline.icon`](Devline.icon/) in Icon Composer. The document uses a transparent monogram with light, dark, and tintable variants.

[`package.sh`](../../scripts/package.sh) compiles the icon with Xcode's `actool`. It bundles `Assets.car` and the generated `Devline.icns` fallback and sets `CFBundleIconName=Devline`.

The menu bar uses a cropped rendition of the monogram at 18 points. macOS supplies the tint. [Menu bar asset](../Sources/DevlineApp/Resources/MenuBarMonogram.png)
