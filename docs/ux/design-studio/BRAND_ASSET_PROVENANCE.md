# UX3 Brand Asset Provenance

## Husky logo

- **Source:** `C:/Users/Asus Vivobook/Desktop/EncryptDrive/src/main/resources/com/fabianrodas/images/logo.png` in the original Java application repository.
- **Product evidence:** the source repository README presents the image as the EncryptDrive logo and states `Copyright © 2026 Fabián Rodas. All rights reserved.`
- **Destination:** `apps/encryptdrive-desktop/assets/brand/encryptdrive-husky.png` (135,787 bytes at audit time).
- **Integrity:** source and destination SHA-256 both equal `252CEADB526CC8266F9DB82A92ACA11BB51142862E3A6AFA0DC9288448DB371B`.
- **Use:** app header and native Windows window icon. The image is embedded in the binary; it is not loaded from disk at runtime.
- **Authorization boundary:** the user explicitly authorized reuse in this product repository for UX3. The Java repository and its asset were not modified. The Java repository has no product-level open-source license in the reviewed state; this report does not grant public redistribution, sublicensing, or ownership rights. Product/legal owner approval is required before public distribution of this image.

The earlier geometric placeholder is no longer the current source asset. No private credentials or user data were copied. The owner should inspect genuine header/title-bar captures before visual acceptance.

## Icon font

`Phosphor-Regular.ttf` is embedded from the `egui-phosphor` 0.14.0 package resource and the MIT license is included at `apps/encryptdrive-desktop/assets/licenses/PHOSPHOR_LICENSE.txt`. Its copyright and permission notice must accompany redistribution. See [iconography audit](ICONOGRAPHY_AUDIT.md).
