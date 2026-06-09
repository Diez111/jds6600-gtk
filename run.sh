#!/bin/bash
# JDS6600 Signal Generator — Native Rust GTK Launcher

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Verificar permisos de puertos seriales
for port in /dev/ttyUSB* /dev/ttyACM*; do
    if [ -e "$port" ]; then
        if [ ! -w "$port" ]; then
            echo "Dando permisos al puerto $port..."
            echo 'Solyverano101' | sudo -S chmod 666 "$port" 2>/dev/null || true
        fi
    fi
done

# Ejecutar app nativa
echo "Iniciando JDS6600 Signal Generator (Rust + GTK4)..."
./target/release/jds6600-gtk
