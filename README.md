# JDS6600 GTK — Generador de Señales Profesional

Aplicación de escritorio nativa en **Rust + GTK4** para controlar generadores de señales **JDS6600** (y compatibles) a través de puerto serial USB. Interfaz profesional de laboratorio con modo claro/oscuro, auto-detección de puertos, y visualización en tiempo real de formas de onda.

![GTK4](https://img.shields.io/badge/GTK4-0.8-blue)
![Rust](https://img.shields.io/badge/Rust-2021-orange)
![Version](https://img.shields.io/badge/version-0.2.0-green)
![License](https://img.shields.io/badge/license-MIT-green)

---

## Índice

- [Características](#características)
- [Limitaciones — Modulación entre canales](#limitaciones--modulación-entre-canales)
- [Requisitos del sistema](#requisitos-del-sistema)
- [Compilación](#compilación)
- [Uso](#uso)
- [Arquitectura técnica](#arquitectura-técnica)
- [Protocolo serial JDS6600](#protocolo-serial-jds6600)
- [Estructura del proyecto](#estructura-del-proyecto)
- [Dependencias](#dependencias)
- [Limitaciones de hardware](#limitaciones-de-hardware)
- [Changelog](#changelog)
- [Licencia](#licencia)

---

## Características

### Interfaz
- **GTK4 nativo** con `GtkHeaderBar` (decoraciones del sistema operativo)
- **Modo claro/oscuro** con toggle en tiempo real (sin reiniciar)
- **Layout responsivo** con `GtkGrid` — nada desborda, todo alineado
- **Dos paneles de canal** independientes (CH1/CH2) con colores distintivos
- **Vista previa de osciloscopio** con grilla tipo instrumento (Cairo)
- **Toast notifications** flotantes sobre `GtkOverlay`
- **Switches nativos** para encendido/apagado de canales
- **Entrada de frecuencia editable** con selección de unidades (Hz, kHz, MHz)
- **Presets de frecuencia** rápidos: 50Hz, 100Hz, 1kHz, 10kHz, 100kHz, 1MHz, 10MHz
- **Duty cycle inteligente**: se deshabilita automáticamente para formas de onda que no lo usan

### Funcionalidad
- **Auto-detección plug & play** del JDS6600 entre múltiples puertos seriales
- **Filtrado de puertos fantasma** (ignora `ttyS*` nativos del chipset)
- **Conexión verificada**: solo marca "conectado" si el dispositivo responde al protocolo
- **Polling en tiempo real** (500ms) para sincronizar estado con el hardware
- **8 presets** guardados en `presets.json` (click = cargar/guardar, click derecho = borrar)
- **Sync 1→2**: copia configuración del Canal 1 al Canal 2
- **Apagar todo**: desactiva ambos canales instantáneamente
- **17 formas de onda**: senoidal, cuadrada, pulso, triangular, CMOS, DC, media onda, onda completa, escalera pos/neg, ruido, exponencial subida/bajada, multi-tono, sinc, Lorenz

### Comunicación
- Driver serial nativo en Rust (sin dependencias de Python)
- Lectura directa del puerto sin `try_clone()` (compatible con CH340/FTDI/CP210x)
- Timeout configurable, reintentos automáticos en auto-detección
- Manejo robusto de errores con `anyhow`

---

## Limitaciones — Modulación entre canales

**El JDS6600 no soporta modulación directa entre canales a través del protocolo serial.**

El JDS6600 tiene:
- 2 canales **independientes** (cada uno con su propia configuración)
- Modulación interna (AM/FM/PM) usando señales internas del generador
- Entrada de modulación externa (conector BNC trasero)

**No hay comandos seriales** para que el Canal 1 module al Canal 2 directamente.

### Alternativas para lograr modulación:

1. **Modulación externa física**: Conectar la salida del Canal 1 al conector de entrada de modulación externa (si tu JDS6600 lo tiene). Esto es hardware, no software.

2. **Software externo**: Usar un programa como GNU Radio o similar para generar señales moduladas y enviarlas por USB (pero esto requiere hardware adicional como un SDR).

3. **Sincronización manual**: Configurar ambos canales con parámetros relacionados (ej: Canal 1 a 1kHz, Canal 2 a 2kHz) para crear efectos de interferencia, pero no es modulación real.

El protocolo serial del JDS6600 solo expone control básico: forma de onda, frecuencia, amplitud, offset y duty cycle para cada canal. Las funciones avanzadas de modulación solo están disponibles desde el panel frontal del equipo o mediante la entrada externa.

---

## Requisitos del sistema

### Para compilar
- **Rust** 1.70+ (edition 2021)
- **GTK4** development libraries:
  ```bash
  # Debian/Ubuntu
  sudo apt install libgtk-4-dev

  # Fedora
  sudo dnf install gtk4-devel

  # Arch
  sudo pacman -S gtk4
  ```

### Para ejecutar
- **GTK4** runtime libraries
- Permisos de acceso al puerto serial (grupo `dialout` en Linux):
  ```bash
  sudo usermod -aG dialout $USER
  # Cerrar sesión y volver a entrar
  ```
- Generador JDS6600 conectado por USB (chip CH340, FTDI, o compatible)

---

## Compilación

```bash
git clone https://github.com/Diez111/jds6600-gtk.git
cd jds6600-gtk
cargo build --release
```

El binario se genera en `target/release/jds6600-gtk`.

### Ejecutar

```bash
./target/release/jds6600-gtk
```

O crear un script `run.sh`:
```bash
#!/bin/bash
cd "$(dirname "$0")"
./target/release/jds6600-gtk
```

---

## Uso

1. **Conectar el generador** por USB al computador
2. **Ejecutar la aplicación**
3. **Tocar "Escanear"** en la barra superior — la app prueba cada puerto serial y auto-detecta el JDS6600
4. **Tocar "Conectar"** — se verifica la comunicación real con el dispositivo
5. **Configurar canales**: forma de onda, frecuencia, amplitud, offset, duty cycle
6. **Usar presets de frecuencia** para cambios rápidos
7. **Guardar presets**: tocar un botón numérico (1-8) para guardar el estado actual; tocarlo de nuevo para restaurarlo
8. **Borrar preset**: click derecho sobre el botón del preset
9. **Cambiar tema**: botón ☀/☾ en la barra superior

---

## Arquitectura técnica

### Módulos

```
src/
├── main.rs       → Entry point, inicializa GTK Application
├── app.rs        → UI completa (GTK4 widgets, CSS, callbacks, threading)
├── driver.rs     → Driver serial nativo JDS6600 (protocolo, auto-detect)
├── model.rs      → Estructuras de datos (PresetBank, Preset, ChannelPreset)
└── waveform.rs   → Renderizado Cairo de 17 formas de onda con grilla osciloscopio
```

### `driver.rs` — Driver serial

- **Protocolo**: texto a 115200 baud, 8N1, timeout 1s
- **Comandos**: `:rREG=0.\n` (leer), `:wREG=VAL.\n` (escribir)
- **Registros**:
  - r20/w20: estado de canales (enable/disable)
  - r21/w21, r22/w22: forma de onda CH1/CH2
  - r23/w23, r24/w24: frecuencia CH1/CH2 (Hz × 100, magnitud)
  - r25/w25, r26/w26: amplitud CH1/CH2 (mV)
  - r27/w27, r28/w28: offset CH1/CH2 (centiV + 1000)
  - r29/w29, r30/w30: duty cycle CH1/CH2 (décimas %)
- **Auto-detección**: `auto_detect_port()` itera candidatos, abre cada uno, envía `:r20=0.\n`, valida respuesta `:r20=...`
- **Filtrado**: solo busca `/dev/ttyUSB*` y `/dev/ttyACM*`, ignora `ttyS*` fantasma
- **Lectura directa**: usa `BufReader::new(&mut **conn)` sin `try_clone()` (compatible con adaptadores USB-serial que no soportan duplicación de fd)

### `app.rs` — Interfaz GTK4

- **HeaderBar nativo**: status dot, port combo, scan/connect/theme buttons
- **Channel panels**: `GtkGrid` con labels, switches, combos, spin buttons, scale, preview
- **CSS dual**: `dark_css()` y `light_css()` con providers intercambiables en runtime
- **Threading**: operaciones seriales en `std::thread::spawn`, resultados via `Arc<Mutex<Option<T>>>` + `timeout_add_local` polling (50ms)
- **Polling de estado**: `timeout_add_local(500ms)` lee `get_full_state()` y actualiza UI
- **Toast**: `GtkRevealer` con slide-up animation, auto-hide a 2.5s
- **Overlay**: `GtkOverlay` para posicionar toast sobre el contenido principal

### `waveform.rs` — Renderizado Cairo

- **Grilla de osciloscopio**: 8×6 divisiones con líneas finas, ejes centrales más gruesos
- **17 funciones de dibujo**: `draw_sine`, `draw_square`, `draw_triangle`, etc.
- **Colores por canal**: CH1 naranja (0.94, 0.53, 0.24), CH2 verde (0.25, 0.73, 0.31)
- **300 puntos** por curva para suavidad
- **Duty cycle** aplicado en square/pulse/triangle/CMOS

### `model.rs` — Persistencia

- **PresetBank**: HashMap de slots (1-8) a `Option<Preset>`
- **Preset**: contiene `ChannelPreset` para CH1 y CH2
- **ChannelPreset**: enabled, waveform, frequency, amplitude, offset, duty_cycle
- **Serialización**: `serde_json` → `presets.json` en el directorio de trabajo

---

## Protocolo serial JDS6600

### Configuración
- **Baudrate**: 115200
- **Data bits**: 8
- **Stop bits**: 1
- **Parity**: None
- **Timeout**: 1 segundo

### Formato de comandos

**Lectura:**
```
Host → :rREG=0.\n
Device ← :rREG=VALUE.
```

**Escritura:**
```
Host → :wREG=VALUE.\n
Device ← :ok
```

### Registros principales

| Registro | Función | Formato de valor |
|----------|---------|------------------|
| 20 | Estado canales | `CH1,CH2` (0=off, 1=on) |
| 21/22 | Forma de onda CH1/CH2 | ID 0-16 |
| 23/24 | Frecuencia CH1/CH2 | `FREQ,MAG` (Hz×100, magnitud 0-6) |
| 25/26 | Amplitud CH1/CH2 | mV (1-20000) |
| 27/28 | Offset CH1/CH2 | centiV + 1000 (1-1999) |
| 29/30 | Duty cycle CH1/CH2 | décimas % (0-1000) |

### Ejemplo

**Leer frecuencia del canal 1:**
```
→ :r23=0.\n
← :r23=100000,3.
```
Interpretación: 100000 × 10^(-3) / 100 = 100.0 Hz

**Escribir frecuencia del canal 1 a 1kHz:**
```
→ :w23=100000,3.\n
← :ok
```

---

## Estructura del proyecto

```
jds6600-gtk/
├── Cargo.toml          → Dependencias y metadata
├── Cargo.lock          → Lockfile de versiones exactas
├── README.md           → Esta documentación
├── .gitignore          → Ignorar target/, presets.json, etc.
├── run.sh              → Script de ejecución
├── presets.json        → Presets guardados (generado en runtime)
└── src/
    ├── main.rs         → Entry point
    ├── app.rs          → UI GTK4 completa (~1300 líneas)
    ├── driver.rs       → Driver serial JDS6600 (~470 líneas)
    ├── model.rs        → Modelos de datos con serde
    └── waveform.rs     → Renderizado Cairo de formas de onda
```

---

## Dependencias

| Crate | Versión | Propósito |
|-------|---------|-----------|
| `gtk4` | 0.8 | Interfaz gráfica GTK4 |
| `glib` | 0.19 | Event loop, threading helpers |
| `cairo-rs` | 0.19 | Renderizado 2D (osciloscopio) |
| `serialport` | 4.2 | Comunicación serial cross-platform |
| `serde` | 1.0 | Serialización de presets |
| `serde_json` | 1.0 | JSON para presets.json |
| `anyhow` | 1.0 | Manejo de errores ergonómico |
| `rand` | 0.8 | Generación de ruido (forma de onda) |
| `glob` | 0.3 | Detección de puertos `/dev/ttyUSB*` |

---

## Limitaciones de hardware

El JDS6600 tiene límites físicos que la aplicación respeta:

| Parámetro | Mínimo | Máximo | Resolución |
|-----------|--------|--------|------------|
| Frecuencia | 0.01 Hz | 60 MHz | 0.01 Hz |
| Amplitud | 1 mV | 20 V | 1 mV |
| Offset | -9.99 V | 9.99 V | 10 mV |
| Duty cycle | 0% | 100% | 0.1% |

**Nota**: La versión "lite" del JDS6600 solo genera hasta 15 MHz. La app permite hasta 60 MHz; el hardware limitará automáticamente.

**Duty cycle**: Solo aplicable a formas de onda: square, pulse, triangle, CMOS. Para otras formas de onda, el control se deshabilita en la UI.

---

## Solución de problemas

### "No se detectaron puertos seriales"
- Verificar que el generador esté conectado por USB
- Verificar que el driver CH340/FTDI esté cargado: `lsmod | grep ch341`
- Verificar que `/dev/ttyUSB0` exista: `ls -la /dev/ttyUSB*`
- Verificar permisos: `groups | grep dialout`

### "El dispositivo no responde"
- El generador puede estar en uso por otra aplicación
- Cerrar otros programas que usen el puerto serial
- Desconectar y reconectar el USB

### La app no arranca
- Verificar que GTK4 esté instalado: `pkg-config --modversion gtk4`
- En Wayland, puede necesitar `GDK_BACKEND=x11` para compatibilidad

---

## Changelog

### v0.2.1 — Protección de entrada de frecuencia

**Correcciones críticas:**
- Fix de edición interrumpida: el polling ya no actualiza el Entry mientras el usuario escribe
- Uso de EventControllerFocus para detectar foco correctamente en GTK4
- Validación de límites al cambiar unidad (Hz/kHz/MHz) para evitar valores inválidos
- Clamp automático según límites del hardware al cambiar unidad

### v0.2.0 — Rediseño profesional y correcciones

**UI/UX profesional:**
- Rediseño completo con `GtkHeaderBar` nativo (decoraciones del sistema)
- Modo claro/oscuro con toggle en tiempo real (sin reiniciar)
- Layout con `GtkGrid` alineado — nada desborda
- `GtkSwitch` nativo para encendido/apagado de canales
- `GtkOverlay` para toast flotantes
- Preview de osciloscopio más grande (400x120)
- Presets de frecuencia rápidos: 50Hz, 100Hz, 1kHz, 10kHz, 100kHz, 1MHz, 10MHz
- Duty cycle se deshabilita automáticamente para formas de onda que no lo usan
- Límites de hardware exactos en todos los ajustes

**Entrada de frecuencia mejorada:**
- Campo de texto editable para escribir frecuencia directamente
- Selector de unidades: Hz, kHz, MHz
- Conversión automática al presionar Enter
- Actualización en tiempo real al cambiar unidad
- Sincronización con presets y polling
- **Protección contra edición interrumpida**: el polling no actualiza el campo mientras el usuario escribe
- **Validación de límites**: al cambiar unidad, se clamp automáticamente para evitar valores inválidos (ej: 6 millones de MHz)
- **Selección automática al enfocar**: al hacer click en el campo, se selecciona todo el texto para facilitar edición

**Auto-detección mejorada:**
- Filtrado de puertos fantasma `ttyS*` (nativos del chipset)
- Solo busca puertos USB reales (`/dev/ttyUSB*`, `/dev/ttyACM*`)
- Verificación de puertos abribles antes de incluirlos en la lista
- Priorización de puertos USB/ACM sobre otros

**Correcciones críticas:**
- Fix de lectura serial: `BufReader::new(&mut **conn)` en vez de `try_clone()` (fallaba en CH340/FTDI)
- Fix de preview: `queue_draw()` en callbacks de forma de onda y duty cycle
- Fix de frecuencia: eliminado loop de polling que reenviaba frecuencia al generador
- Fix de edición de frecuencia: el polling no interrumpe la edición del usuario
  - Flags `is_editing` bloquean actualizaciones del SpinButton mientras se edita
  - Cambio se aplica al presionar Enter o perder el foco
  - `connect_value_changed` bloqueado durante edición manual
- Timeout normalizado a 1 segundo (como el proyecto Python de referencia)
- Delay post-conexión de 400ms para arranque del MCU

**Documentación:**
- README completo en español con documentación técnica
- Sección sobre limitaciones de modulación entre canales
- Protocolo serial documentado con ejemplos
- Solución de problemas comunes

### v0.1.0 — Versión inicial

- Control básico de JDS6600 via serial USB
- 2 canales independientes con 17 formas de onda
- 8 presets con persistencia JSON
- Sync 1→2, Apagar todo
- Polling en tiempo real (500ms)
- Auto-detección básica de puertos

---

## Licencia

MIT License — ver archivo LICENSE para detalles.

---

## Créditos

- **Protocolo JDS6600**: Documentación de [Joy-IT JT-JDS6600](https://joy-it.net/de/products/JT-JDS6600) y proyecto [WimDH/JDS6600](https://github.com/WimDH/JDS6600)
- **Desarrollo**: Diez111
- **Tecnologías**: Rust, GTK4, Cairo, serialport-rs

---

## Capturas de pantalla

*Modo oscuro:*

<img width="951" height="732" alt="image" src="https://github.com/user-attachments/assets/0d9374e9-4730-420a-9089-dac9494b834d" />


*Modo claro:*

<img width="981" height="757" alt="image" src="https://github.com/user-attachments/assets/787b2802-860d-4d21-81d7-dae940cb61ed" />



---

**Desarrollado con Rust + GTK4 para control profesional de instrumentación de laboratorio.**
