use anyhow::{bail, Result};
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

pub const WAVEFORMS: &[&str] = &[
    "sine",
    "square",
    "pulse",
    "triangle",
    "parial_sine",
    "cmos",
    "dc",
    "half_wave",
    "full_wave",
    "pos_ladder",
    "neg_ladder",
    "noise",
    "exp_rise",
    "exp_decay",
    "multi_tone",
    "sinc",
    "lorenz",
];

fn waveform_id(name: &str) -> Result<u8> {
    WAVEFORMS
        .iter()
        .position(|&w| w == name)
        .map(|i| i as u8)
        .ok_or_else(|| anyhow::anyhow!("Waveform '{}' no soportado", name))
}

fn waveform_name(id: u8) -> Result<String> {
    WAVEFORMS
        .get(id as usize)
        .map(|&s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Waveform id {} fuera de rango", id))
}

#[derive(Clone, Debug, Default)]
pub struct ChannelState {
    pub enabled: bool,
    pub waveform: String,
    pub frequency: f64,
    pub amplitude: f64,
    pub offset: f64,
    pub duty_cycle: f64,
}

#[derive(Clone, Debug, Default)]
pub struct FullState {
    pub connected: bool,
    pub port: String,
    pub ch1: ChannelState,
    pub ch2: ChannelState,
}

pub struct Jds6600 {
    port_name: String,
    conn: Option<Box<dyn serialport::SerialPort>>,
}

/// Detecta puertos seriales USB disponibles en el sistema.
/// Solo devuelve puertos que existen físicamente (ttyUSB*, ttyACM*) y se pueden abrir.
pub fn detect_serial_ports() -> Vec<String> {
    let mut ports = Vec::new();

    // 1) Buscar dispositivos USB-serial reales via glob
    for pattern in &["/dev/ttyUSB*", "/dev/ttyACM*"] {
        if let Ok(entries) = glob::glob(pattern) {
            for entry in entries.flatten() {
                if let Some(path) = entry.to_str() {
                    // Verificar que realmente se pueda abrir (timeout corto para no bloquear)
                    if serialport::new(path, 115_200)
                        .timeout(Duration::from_millis(100))
                        .open()
                        .is_ok()
                    {
                        ports.push(path.to_string());
                    }
                }
            }
        }
    }

    // 2) Complementar con serialport crate filtrando solo puertos abribles
    if let Ok(available) = serialport::available_ports() {
        for p in available {
            let path = p.port_name;
            // Ignorar puertos ttyS* fantasmas del chipset nativo
            if path.contains("ttyS") {
                continue;
            }
            if !ports.contains(&path) {
                // Verificar que se pueda abrir antes de incluirlo (timeout corto)
                if serialport::new(&path, 115_200)
                    .timeout(Duration::from_millis(100))
                    .open()
                    .is_ok()
                {
                    ports.push(path);
                }
            }
        }
    }

    ports.sort();
    ports
}

/// Prueba si un puerto específico responde como un JDS6600.
/// Usa BufReader sobre referencia mutable directa (sin try_clone, como pyserial).
fn is_jds6600(port: &str) -> Result<bool> {
    eprintln!("[JDS6600] Probando puerto: {}", port);

    let mut p = match serialport::new(port, 115_200)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_secs(1))
        .open()
    {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[JDS6600]   -> No se pudo abrir: {}", e);
            return Ok(false);
        }
    };

    // Algunos adaptadores resetean el MCU al abrir; darle tiempo de arrancar
    std::thread::sleep(Duration::from_millis(400));

    // Limpiar basura previa del buffer
    let _ = p.clear(serialport::ClearBuffer::All);

    // Helper que envía el probe y lee una línea con BufReader sobre ref mut directa
    let probe = |p: &mut Box<dyn serialport::SerialPort>| -> Result<String> {
        p.write_all(b":r20=0.\n")?;
        p.flush()?;
        let mut line = String::new();
        {
            let mut reader = BufReader::new(&mut **p);
            reader.read_line(&mut line)?;
        }
        Ok(line)
    };

    // Primer intento
    let line = match probe(&mut p) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[JDS6600]   -> Error en primer intento: {}", e);
            std::thread::sleep(Duration::from_millis(300));
            let _ = p.clear(serialport::ClearBuffer::All);
            match probe(&mut p) {
                Ok(l) => l,
                Err(e2) => {
                    eprintln!("[JDS6600]   -> Error en segundo intento: {}", e2);
                    return Ok(false);
                }
            }
        }
    };

    let trimmed = line.trim();
    eprintln!("[JDS6600]   -> Respuesta recibida: '{}'", trimmed);

    // Aceptar si contiene "r20=" en cualquier parte
    let ok = trimmed.contains("r20=");
    if ok {
        eprintln!("[JDS6600]   -> ¡JDS6600 confirmado en {}!", port);
    } else {
        eprintln!("[JDS6600]   -> No es JDS6600 (no contiene 'r20=')");
    }
    Ok(ok)
}

/// Intenta detectar automáticamente un JDS6600 conectado probando cada puerto serial candidato.
/// Devuelve el nombre del puerto si lo encuentra, o None si no hay respuesta válida.
pub fn auto_detect_port() -> Option<String> {
    let candidates = detect_serial_ports();
    let (usb, other): (Vec<_>, Vec<_>) = candidates
        .into_iter()
        .partition(|p| p.contains("ttyUSB") || p.contains("ttyACM"));
    for port in usb.into_iter().chain(other.into_iter()) {
        if let Ok(true) = is_jds6600(&port) {
            return Some(port);
        }
    }
    None
}

impl Jds6600 {
    pub fn new(port: &str) -> Self {
        Self {
            port_name: port.to_string(),
            conn: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.conn.is_some()
    }

    pub fn connect(&mut self, port: &str) -> Result<()> {
        self.disconnect();
        self.port_name = port.to_string();
        let port = serialport::new(port, 115_200)
            .data_bits(serialport::DataBits::Eight)
            .stop_bits(serialport::StopBits::One)
            .parity(serialport::Parity::None)
            .timeout(Duration::from_millis(200))
            .open()?;
        // Algunos adaptadores resetean el MCU al abrir (DTR/RTS); darle tiempo de arrancar
        std::thread::sleep(Duration::from_millis(400));
        self.conn = Some(port);
        Ok(())
    }

    pub fn disconnect(&mut self) {
        if let Some(c) = self.conn.take() {
            let _ = c.clear(serialport::ClearBuffer::All);
        }
        self.conn = None;
    }

    fn ensure_connected(&self) -> Result<()> {
        if self.conn.is_none() {
            bail!("Dispositivo no conectado");
        }
        Ok(())
    }

    fn send_cmd(&mut self, cmd: &str) -> Result<String> {
        self.ensure_connected()?;
        let conn = self.conn.as_mut().unwrap();
        conn.write_all(cmd.as_bytes())?;
        conn.flush()?;

        // Leer respuesta con BufReader sobre referencia mutable directa (sin try_clone)
        let mut line = String::new();
        {
            let mut reader = BufReader::new(&mut **conn);
            reader.read_line(&mut line)?;
        }

        // Limpia terminadores: puede venir :r20=1,1.\r\n o similar
        let clean = line.trim().trim_end_matches('.').to_string();
        if clean.is_empty() {
            bail!("Respuesta vacía del dispositivo");
        }
        Ok(clean)
    }

    fn parse_response(data: &str) -> Result<String> {
        if data == ":ok" {
            return Ok("ok".to_string());
        }
        if let Some(pos) = data.find('=') {
            let val = &data[pos + 1..];
            // Algunos firmwares no incluyen punto final, otros sí. Ya lo quitamos.
            return Ok(val.to_string());
        }
        bail!("Formato de respuesta no reconocido: {}", data)
    }

    // ── Getters ─────────────────────────────────

    pub fn get_channels(&mut self) -> Result<(bool, bool)> {
        let resp = self.send_cmd(":r20=0.\n")?;
        let val = Self::parse_response(&resp)?;
        let parts: Vec<&str> = val.split(',').collect();
        if parts.len() != 2 {
            bail!("Esperado 2 valores de canal, obtenido: {}", val);
        }
        Ok((parts[0].trim() == "1", parts[1].trim() == "1"))
    }

    pub fn get_waveform(&mut self, ch: u8) -> Result<String> {
        if ch != 1 && ch != 2 {
            bail!("Canal debe ser 1 o 2");
        }
        let resp = self.send_cmd(&format!(":r{}=0.\n", 20 + ch))?;
        let val = Self::parse_response(&resp)?;
        let id = val.parse::<u8>()?;
        waveform_name(id)
    }

    pub fn get_frequency(&mut self, ch: u8) -> Result<f64> {
        if ch != 1 && ch != 2 {
            bail!("Canal debe ser 1 o 2");
        }
        let resp = self.send_cmd(&format!(":r{}=0.\n", 22 + ch))?;
        let val = Self::parse_response(&resp)?;
        let parts: Vec<&str> = val.split(',').collect();
        if parts.len() != 2 {
            bail!("Formato frecuencia inválido: {}", val);
        }
        let freq = parts[0].trim().parse::<f64>()?;
        let mag = parts[1].trim().parse::<usize>()?;
        let mut hz = freq;
        for _ in 0..mag {
            hz /= 1000.0;
        }
        Ok(hz / 100.0)
    }

    pub fn get_amplitude(&mut self, ch: u8) -> Result<f64> {
        if ch != 1 && ch != 2 {
            bail!("Canal debe ser 1 o 2");
        }
        let resp = self.send_cmd(&format!(":r{}=0.\n", 24 + ch))?;
        let val = Self::parse_response(&resp)?;
        Ok(val.parse::<f64>()? / 1000.0)
    }

    pub fn get_offset(&mut self, ch: u8) -> Result<f64> {
        if ch != 1 && ch != 2 {
            bail!("Canal debe ser 1 o 2");
        }
        let resp = self.send_cmd(&format!(":r{}=0.\n", 26 + ch))?;
        let val = Self::parse_response(&resp)?;
        let raw = val.parse::<f64>()?;
        Ok((raw - 1000.0) / 100.0)
    }

    pub fn get_dutycycle(&mut self, ch: u8) -> Result<f64> {
        if ch != 1 && ch != 2 {
            bail!("Canal debe ser 1 o 2");
        }
        let resp = self.send_cmd(&format!(":r{}=0.\n", 28 + ch))?;
        let val = Self::parse_response(&resp)?;
        let raw = val.parse::<f64>()?;
        Ok((raw / 10.0).round() / 10.0) // redondea a 1 decimal
    }

    // ── Setters ─────────────────────────────────

    pub fn set_channels(&mut self, ch1: bool, ch2: bool) -> Result<()> {
        let s1 = if ch1 { "1" } else { "0" };
        let s2 = if ch2 { "1" } else { "0" };
        let resp = self.send_cmd(&format!(":w20={s1},{s2}.\n"))?;
        let val = Self::parse_response(&resp)?;
        if val != "ok" {
            bail!("Error al setear canales: {}", val);
        }
        Ok(())
    }

    pub fn set_waveform(&mut self, ch: u8, name: &str) -> Result<()> {
        if ch != 1 && ch != 2 {
            bail!("Canal debe ser 1 o 2");
        }
        let id = waveform_id(name)?;
        let resp = self.send_cmd(&format!(":w{}={}.\n", 20 + ch, id))?;
        let val = Self::parse_response(&resp)?;
        if val != "ok" {
            bail!("Error al setear waveform: {}", val);
        }
        Ok(())
    }

    pub fn set_frequency(&mut self, ch: u8, hz: f64) -> Result<()> {
        if ch != 1 && ch != 2 {
            bail!("Canal debe ser 1 o 2");
        }
        let val = (hz * 100.0) as i64;
        eprintln!("[DEBUG] set_frequency(ch={}, hz={}, val={})", ch, hz, val);
        let resp = self.send_cmd(&format!(":w{}={val},0.\n", 22 + ch))?;
        eprintln!("[DEBUG] Respuesta: {}", resp);
        let val = Self::parse_response(&resp)?;
        if val != "ok" {
            bail!("Error al setear frecuencia: {}", val);
        }
        Ok(())
    }

    pub fn set_amplitude(&mut self, ch: u8, v: f64) -> Result<()> {
        if ch != 1 && ch != 2 {
            bail!("Canal debe ser 1 o 2");
        }
        if v < 0.001 || v > 20.0 {
            bail!("Amplitud debe estar entre 1mV y 20V");
        }
        let val = (v * 1000.0) as i64;
        let resp = self.send_cmd(&format!(":w{}={val}.\n", 24 + ch))?;
        let val = Self::parse_response(&resp)?;
        if val != "ok" {
            bail!("Error al setear amplitud: {}", val);
        }
        Ok(())
    }

    pub fn set_offset(&mut self, ch: u8, v: f64) -> Result<()> {
        if ch != 1 && ch != 2 {
            bail!("Canal debe ser 1 o 2");
        }
        let offset = (v * 100.0).round() / 100.0;
        if offset < -10.0 || offset > 10.0 {
            bail!("Offset debe estar entre -9.99V y 9.99V");
        }
        let reg_val = ((offset * 100.0) + 1000.0) as i64;
        let resp = self.send_cmd(&format!(":w{}={reg_val}.\n", 26 + ch))?;
        let val = Self::parse_response(&resp)?;
        if val != "ok" {
            bail!("Error al setear offset: {}", val);
        }
        Ok(())
    }

    pub fn set_dutycycle(&mut self, ch: u8, pct: f64) -> Result<()> {
        if ch != 1 && ch != 2 {
            bail!("Canal debe ser 1 o 2");
        }
        let dc = (pct * 10.0).round() / 10.0;
        if dc < 0.0 || dc > 100.0 {
            bail!("Duty cycle debe estar entre 0% y 100%");
        }
        let reg_val = (dc * 10.0) as i64;
        let resp = self.send_cmd(&format!(":w{}={reg_val}.\n", 28 + ch))?;
        let val = Self::parse_response(&resp)?;
        if val != "ok" {
            bail!("Error al setear duty cycle: {}", val);
        }
        Ok(())
    }

    // ── Bulk ops ────────────────────────────────

    pub fn get_full_state(&mut self) -> Result<FullState> {
        let (ch1_on, ch2_on) = self.get_channels()?;
        let s1 = ChannelState {
            enabled: ch1_on,
            waveform: self.get_waveform(1).unwrap_or_else(|_| "sine".to_string()),
            frequency: self.get_frequency(1).unwrap_or(1000.0),
            amplitude: self.get_amplitude(1).unwrap_or(1.0),
            offset: self.get_offset(1).unwrap_or(0.0),
            duty_cycle: self.get_dutycycle(1).unwrap_or(50.0),
        };
        let s2 = ChannelState {
            enabled: ch2_on,
            waveform: self.get_waveform(2).unwrap_or_else(|_| "sine".to_string()),
            frequency: self.get_frequency(2).unwrap_or(1000.0),
            amplitude: self.get_amplitude(2).unwrap_or(1.0),
            offset: self.get_offset(2).unwrap_or(0.0),
            duty_cycle: self.get_dutycycle(2).unwrap_or(50.0),
        };
        Ok(FullState {
            connected: true,
            port: self.port_name.clone(),
            ch1: s1,
            ch2: s2,
        })
    }

    pub fn all_off(&mut self) -> Result<()> {
        self.set_channels(false, false)
    }

    pub fn sync_channels(&mut self) -> Result<()> {
        let s1 = self.get_full_state()?.ch1;
        let (_, _ch2_on) = self.get_channels()?;
        self.set_waveform(2, &s1.waveform)?;
        self.set_frequency(2, s1.frequency)?;
        self.set_amplitude(2, s1.amplitude)?;
        self.set_offset(2, s1.offset)?;
        self.set_dutycycle(2, s1.duty_cycle)?;
        self.set_channels(s1.enabled, s1.enabled)?;
        Ok(())
    }
}
