# Marey — iconos e imágenes base para Tauri v2

Marca: **A · Cronofotografía** (M con ecos de movimiento y marcadores en las articulaciones).
Colores: fondo `#0E0D0B` · tiza `#EEEAE0` · papel `#F2EFE7` · tinta `#15140F` · acento `#FF5A36`.
Tipografía del wordmark: Sora SemiBold (convertida a trazados en los SVG; no hace falta instalarla).

## Contenido

| Archivo | Uso |
|---|---|
| `app-icon.png` / `.svg` | **Fuente para `tauri icon`** (1024×1024, sangrado completo) |
| `app-icon-macos.png` / `.svg` | Variante con el margen de la rejilla de macOS (forma de 824 px) |
| `tray/tray-icon-template{,@2x}.png` | Icono de bandeja negro sobre transparente (template de macOS) |
| `tray/tray-icon-light{,@2x}.png` | Icono de bandeja blanco (barras oscuras de Windows/Linux) |
| `installer/nsis-header.bmp` | NSIS, cabecera 150×57 |
| `installer/nsis-sidebar.bmp` | NSIS, lateral de bienvenida y fin 164×314 |
| `installer/wix-banner.bmp` | WiX (MSI), banner 493×58 |
| `installer/wix-dialog.bmp` | WiX (MSI), diálogo 493×312 |
| `installer/dmg-background{,@2x}.png` | Fondo del DMG de macOS 660×400 |
| `brand/marey-mark-on-{dark,light}.svg/png` | Marca sola, fondo transparente (UI, README) |
| `brand/marey-lockup-on-{dark,light}.svg/png` | Marca + wordmark |
| `social/og-image.png` | Social preview de GitHub 1280×640 |

## 1. Generar todos los iconos de plataforma

Desde la raíz del proyecto Tauri, copia `app-icon.png` y ejecuta:

```bash
pnpm tauri icon app-icon.png      # o: npm run tauri icon app-icon.png / cargo tauri icon app-icon.png
```

Esto rellena `src-tauri/icons/` con los PNG (32, 128, 128@2x…), `icon.icns`, `icon.ico`, los tiles de Windows Store y, si existen los proyectos móviles, los iconos de Android e iOS.

Si prefieres que el icono de macOS respete el margen estándar del Dock, genera primero con `app-icon.png`, luego vuelve a generar con `app-icon-macos.png` en otra carpeta (`-o /tmp/mac-icons`) y copia solo su `icon.icns` encima.

## 2. Copiar los extras

```
src-tauri/icons/tray/        ← tray/*.png
src-tauri/installer/         ← installer/*.bmp, installer/dmg-background.png
```

## 3. tauri.conf.json (fragmento)

```json
{
  "app": {
    "trayIcon": {
      "iconPath": "icons/tray/tray-icon-template@2x.png",
      "iconAsTemplate": true
    }
  },
  "bundle": {
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "windows": {
      "nsis": {
        "installerIcon": "icons/icon.ico",
        "headerImage": "installer/nsis-header.bmp",
        "sidebarImage": "installer/nsis-sidebar.bmp"
      },
      "wix": {
        "bannerPath": "installer/wix-banner.bmp",
        "dialogImagePath": "installer/wix-dialog.bmp"
      }
    },
    "macOS": {
      "dmg": {
        "background": "installer/dmg-background.png",
        "windowSize": { "width": 660, "height": 400 },
        "appPosition": { "x": 180, "y": 170 },
        "applicationFolderPosition": { "x": 480, "y": 170 }
      }
    }
  }
}
```

Notas:
- La flecha del fondo del DMG está dibujada para esas posiciones (app a la izquierda, Aplicaciones a la derecha). Si las cambias, regenera el fondo.
- `iconAsTemplate: true` solo afecta a macOS. En Windows/Linux con barra oscura usa `tray-icon-light@2x.png` (puedes elegirlo en tiempo de ejecución desde Rust con `TrayIconBuilder::icon`).
- Por debajo de 32 px los ecos de la marca desaparecen y queda la M con sus puntos: es intencionado.
