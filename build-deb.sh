#!/bin/bash

# Script para construir el paquete .deb de JDS6600 GTK

set -e

PACKAGE_NAME="jds6600-gtk"
VERSION="0.2.4"
ARCH="amd64"

echo "Construyendo paquete .deb para $PACKAGE_NAME v$VERSION..."

# Crear estructura de directorios
rm -rf build
mkdir -p build/$PACKAGE_NAME/DEBIAN
mkdir -p build/$PACKAGE_NAME/usr/bin
mkdir -p build/$PACKAGE_NAME/usr/share/applications
mkdir -p build/$PACKAGE_NAME/usr/share/icons/hicolor/scalable/apps
mkdir -p build/$PACKAGE_NAME/usr/share/icons/hicolor/16x16/apps
mkdir -p build/$PACKAGE_NAME/usr/share/icons/hicolor/32x32/apps
mkdir -p build/$PACKAGE_NAME/usr/share/icons/hicolor/48x48/apps
mkdir -p build/$PACKAGE_NAME/usr/share/icons/hicolor/64x64/apps
mkdir -p build/$PACKAGE_NAME/usr/share/icons/hicolor/128x128/apps
mkdir -p build/$PACKAGE_NAME/usr/share/icons/hicolor/256x256/apps

# Copiar binario
cp target/release/$PACKAGE_NAME build/$PACKAGE_NAME/usr/bin/

# Copiar archivo .desktop
cp $PACKAGE_NAME.desktop build/$PACKAGE_NAME/usr/share/applications/

# Copiar icono SVG
cp $PACKAGE_NAME.svg build/$PACKAGE_NAME/usr/share/icons/hicolor/scalable/apps/

# Copiar iconos PNG
for size in 16 32 48 64 128 256; do
    if [ -f "$PACKAGE_NAME-${size}.png" ]; then
        cp $PACKAGE_NAME-${size}.png build/$PACKAGE_NAME/usr/share/icons/hicolor/${size}x${size}/apps/$PACKAGE_NAME.png
        echo "✓ Icono ${size}x${size} copiado"
    fi
done

# Crear archivo DEBIAN/control
cat > build/$PACKAGE_NAME/DEBIAN/control << EOF
Package: $PACKAGE_NAME
Version: $VERSION
Section: science
Priority: optional
Architecture: $ARCH
Depends: libc6 (>= 2.31), libgtk-4-1 (>= 4.6), libcairo2 (>= 1.14)
Installed-Size: $(du -sk build/$PACKAGE_NAME/usr | cut -f1)
Maintainer: Diez111 <diez@example.com>
Description: Generador de Señales JDS6600
 Aplicación de escritorio nativa en Rust + GTK4 para controlar
 generadores de señales JDS6600 a través de puerto serial USB.
 .
 Características:
 - Auto-detección plug & play del dispositivo
 - Modo claro/oscuro
 - Entrada de frecuencia editable con unidades (Hz/kHz/MHz)
 - 17 formas de onda
 - 8 presets guardables
 - Vista previa de osciloscopio en tiempo real
 - Control de amplitud, offset y duty cycle
EOF

# Establecer permisos
chmod 755 build/$PACKAGE_NAME/usr/bin/$PACKAGE_NAME
chmod 644 build/$PACKAGE_NAME/usr/share/applications/$PACKAGE_NAME.desktop
chmod 644 build/$PACKAGE_NAME/usr/share/icons/hicolor/scalable/apps/$PACKAGE_NAME.svg

# Construir el paquete
dpkg-deb --build build/$PACKAGE_NAME ${PACKAGE_NAME}_${VERSION}_${ARCH}.deb

echo ""
echo "✓ Paquete creado: ${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
echo ""
echo "Para instalar:"
echo "  sudo dpkg -i ${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
echo ""
echo "Para desinstalar:"
echo "  sudo dpkg -r $PACKAGE_NAME"
