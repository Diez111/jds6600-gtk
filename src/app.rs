use gtk4::prelude::*;
use gtk4::{
    Align, Application, ApplicationWindow, Box as GtkBox, Button, ComboBoxText,
    DrawingArea, Entry, Frame, GestureClick, Grid, HeaderBar, Label, Orientation, Overlay,
    Revealer, Separator, SpinButton, Switch,
};
use glib::source::timeout_add_local;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::driver::{Jds6600, WAVEFORMS};
use crate::model::PresetBank;
use crate::waveform::draw_waveform_with_grid;

const PRESETS_PATH: &str = "presets.json";
const FREQ_MIN_HZ: f64 = 0.01;
const FREQ_MAX_HZ: f64 = 60_000_000.0;
const AMP_MIN_V: f64 = 0.001;
const AMP_MAX_V: f64 = 20.0;
const OFFSET_MIN_V: f64 = -9.99;
const OFFSET_MAX_V: f64 = 9.99;

fn waveform_display(name: &str) -> String {
    match name {
        "sine" => "Senoidal",
        "square" => "Cuadrada",
        "pulse" => "Pulso",
        "triangle" => "Triangular",
        "parial_sine" => "Seno Parcial",
        "cmos" => "CMOS",
        "dc" => "DC",
        "half_wave" => "Media Onda",
        "full_wave" => "Onda Completa",
        "pos_ladder" => "Escalera Pos",
        "neg_ladder" => "Escalera Neg",
        "noise" => "Ruido",
        "exp_rise" => "Exp Subida",
        "exp_decay" => "Exp Bajada",
        "multi_tone" => "Multi-Tono",
        "sinc" => "Sinc",
        "lorenz" => "Lorenz",
        _ => name,
    }
    .to_string()
}

fn waveform_has_duty(name: &str) -> bool {
    matches!(name, "square" | "pulse" | "triangle" | "cmos")
}

fn dark_css() -> &'static str {
    r#"
    window { background-color: #0d1117; color: #e6edf3; }
    headerbar {
        background-color: #161b22;
        border-bottom: 1px solid #21262d;
        padding: 4px 8px;
        min-height: 44px;
    }
    headerbar .title { font-weight: 700; font-size: 14px; color: #e6edf3; }

    .status-dot { font-size: 16px; color: #484f58; }
    .status-dot.on { color: #3fb950; text-shadow: 0 0 8px rgba(63,185,80,0.5); }
    .status-text { font-weight: 600; font-size: 13px; color: #7d8590; font-family: monospace; }
    .status-text.on { color: #e6edf3; }

    .port-combo {
        background-color: #0d1117;
        color: #58a6ff;
        border: 1px solid #30363d;
        border-radius: 6px;
        padding: 4px 8px;
        font-family: monospace;
        font-size: 12px;
        min-width: 180px;
    }

    .btn-connect {
        background-color: #238636;
        color: #ffffff;
        font-weight: 700;
        border-radius: 6px;
        padding: 6px 16px;
        font-size: 12px;
        border: 1px solid #2ea043;
    }
    .btn-connect:hover { background-color: #2ea043; }
    .btn-connect.disconnect {
        background-color: #da3633;
        border-color: #f85149;
    }
    .btn-connect.disconnect:hover { background-color: #f85149; }

    .btn-scan {
        background-color: #21262d;
        color: #58a6ff;
        border: 1px solid #30363d;
        border-radius: 6px;
        padding: 6px 12px;
        font-weight: 600;
        font-size: 12px;
    }
    .btn-scan:hover { background-color: #30363d; border-color: #58a6ff; }

    .btn-theme {
        background-color: transparent;
        border: 1px solid #30363d;
        border-radius: 6px;
        padding: 4px 8px;
        font-size: 16px;
        min-width: 32px;
        min-height: 32px;
    }
    .btn-theme:hover { background-color: #21262d; border-color: #58a6ff; }

    .channel-frame {
        background-color: #161b22;
        border: 1px solid #21262d;
        border-radius: 10px;
        padding: 0;
    }
    .ch1-frame { border-left: 3px solid #f0883e; }
    .ch2-frame { border-left: 3px solid #3fb950; }
    .ch1-frame.on { box-shadow: inset 0 0 0 1px rgba(240,136,62,0.3), 0 0 16px rgba(240,136,62,0.08); }
    .ch2-frame.on { box-shadow: inset 0 0 0 1px rgba(63,185,80,0.3), 0 0 16px rgba(63,185,80,0.08); }

    .ch-title { font-size: 15px; font-weight: 800; letter-spacing: 0.3px; }
    .ch1-title { color: #f0883e; }
    .ch2-title { color: #3fb950; }

    .param-label {
        font-size: 10px;
        text-transform: uppercase;
        letter-spacing: 1.2px;
        color: #7d8590;
        font-weight: 600;
    }
    .param-unit {
        font-size: 11px;
        color: #484f58;
        font-weight: 700;
        font-family: monospace;
    }

    .freq-display {
        font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
        font-size: 20px;
        font-weight: 700;
        color: #58a6ff;
        background-color: #0d1117;
        border: 1px solid #21262d;
        border-radius: 6px;
        padding: 6px 12px;
    }

    .freq-preset {
        background-color: #0d1117;
        color: #7d8590;
        border: 1px solid #21262d;
        border-radius: 4px;
        padding: 2px 6px;
        font-size: 10px;
        font-weight: 600;
        font-family: monospace;
        min-height: 24px;
    }
    .freq-preset:hover {
        border-color: #58a6ff;
        color: #58a6ff;
        background-color: #0d1b2a;
    }

    .freq-entry {
        background-color: #0d1117;
        color: #58a6ff;
        border: 1px solid #30363d;
        border-radius: 6px;
        padding: 6px 10px;
        font-family: 'JetBrains Mono', 'Fira Code', monospace;
        font-size: 14px;
        font-weight: 600;
        min-height: 30px;
    }
    .freq-entry:focus {
        border-color: #58a6ff;
        box-shadow: 0 0 0 2px rgba(88, 166, 255, 0.2);
    }

    .freq-unit-combo {
        background-color: #0d1117;
        color: #7d8590;
        border: 1px solid #30363d;
        border-radius: 6px;
        padding: 4px 8px;
        font-family: monospace;
        font-size: 12px;
        font-weight: 600;
        min-width: 60px;
        min-height: 30px;
    }
    .freq-unit-combo:hover { border-color: #58a6ff; }

    .param-spin {
        background-color: #0d1117;
        color: #e6edf3;
        border: 1px solid #30363d;
        border-radius: 6px;
        padding: 4px 8px;
        font-family: monospace;
        font-size: 13px;
        min-height: 30px;
    }
    .param-spin:focus { border-color: #58a6ff; }

    .waveform-select {
        background-color: #0d1117;
        color: #e6edf3;
        border: 1px solid #30363d;
        border-radius: 6px;
        padding: 4px 8px;
        font-size: 12px;
        min-height: 30px;
    }

    .scope-preview {
        background-color: #010409;
        border: 1px solid #21262d;
        border-radius: 6px;
    }

    .footer-bar {
        background-color: #161b22;
        border: 1px solid #21262d;
        border-radius: 8px;
        padding: 8px 12px;
    }
    .presets-title {
        font-size: 10px;
        text-transform: uppercase;
        letter-spacing: 2px;
        color: #484f58;
        font-weight: 800;
    }
    .preset-btn {
        min-width: 36px;
        min-height: 36px;
        border: 1px dashed #30363d;
        border-radius: 6px;
        background-color: #0d1117;
        color: #484f58;
        font-weight: 800;
        font-size: 13px;
        font-family: monospace;
    }
    .preset-btn:hover {
        border-color: #58a6ff;
        color: #58a6ff;
        background-color: #0d1b2a;
    }
    .preset-btn.saved {
        border-style: solid;
        border-color: #3fb950;
        color: #3fb950;
        background-color: rgba(63,185,80,0.08);
    }
    .preset-btn.saved:hover { background-color: rgba(63,185,80,0.15); }

    .btn-action {
        background-color: #21262d;
        color: #e6edf3;
        border: 1px solid #30363d;
        border-radius: 6px;
        padding: 6px 14px;
        font-weight: 600;
        font-size: 12px;
    }
    .btn-action:hover { border-color: #58a6ff; color: #58a6ff; }

    .btn-danger {
        background-color: #da3633;
        color: #ffffff;
        font-weight: 700;
        border-radius: 6px;
        padding: 6px 14px;
        font-size: 12px;
        border: 1px solid #f85149;
    }
    .btn-danger:hover { background-color: #f85149; }

    .toast-frame {
        background-color: #1c2128;
        border: 1px solid #30363d;
        border-radius: 8px;
        padding: 8px 20px;
        box-shadow: 0 4px 12px rgba(0,0,0,0.4);
    }
    .toast-label { color: #e6edf3; font-weight: 600; font-size: 13px; }
    .toast-error { color: #f85149; }

    separator { background-color: #21262d; }
    switch slider { background-color: #484f58; }
    switch:checked slider { background-color: #3fb950; }
    switch trough { background-color: #21262d; }
    switch:checked trough { background-color: rgba(63,185,80,0.3); }
    "#
}

fn light_css() -> &'static str {
    r#"
    window { background-color: #f6f8fa; color: #1f2328; }
    headerbar {
        background-color: #ffffff;
        border-bottom: 1px solid #d0d7de;
        padding: 4px 8px;
        min-height: 44px;
    }
    headerbar .title { font-weight: 700; font-size: 14px; color: #1f2328; }

    .status-dot { font-size: 16px; color: #afb8c1; }
    .status-dot.on { color: #1a7f37; text-shadow: 0 0 8px rgba(26,127,55,0.3); }
    .status-text { font-weight: 600; font-size: 13px; color: #656d76; font-family: monospace; }
    .status-text.on { color: #1f2328; }

    .port-combo {
        background-color: #ffffff;
        color: #0969da;
        border: 1px solid #d0d7de;
        border-radius: 6px;
        padding: 4px 8px;
        font-family: monospace;
        font-size: 12px;
        min-width: 180px;
    }

    .btn-connect {
        background-color: #1a7f37;
        color: #ffffff;
        font-weight: 700;
        border-radius: 6px;
        padding: 6px 16px;
        font-size: 12px;
        border: 1px solid #1a7f37;
    }
    .btn-connect:hover { background-color: #2da44e; }
    .btn-connect.disconnect {
        background-color: #cf222e;
        border-color: #cf222e;
    }
    .btn-connect.disconnect:hover { background-color: #a40e26; }

    .btn-scan {
        background-color: #f6f8fa;
        color: #0969da;
        border: 1px solid #d0d7de;
        border-radius: 6px;
        padding: 6px 12px;
        font-weight: 600;
        font-size: 12px;
    }
    .btn-scan:hover { background-color: #eaeef2; border-color: #0969da; }

    .btn-theme {
        background-color: transparent;
        border: 1px solid #d0d7de;
        border-radius: 6px;
        padding: 4px 8px;
        font-size: 16px;
        min-width: 32px;
        min-height: 32px;
    }
    .btn-theme:hover { background-color: #eaeef2; border-color: #0969da; }

    .channel-frame {
        background-color: #ffffff;
        border: 1px solid #d0d7de;
        border-radius: 10px;
        padding: 0;
    }
    .ch1-frame { border-left: 3px solid #bc4c00; }
    .ch2-frame { border-left: 3px solid #1a7f37; }
    .ch1-frame.on { box-shadow: inset 0 0 0 1px rgba(188,76,0,0.2), 0 0 12px rgba(188,76,0,0.06); }
    .ch2-frame.on { box-shadow: inset 0 0 0 1px rgba(26,127,55,0.2), 0 0 12px rgba(26,127,55,0.06); }

    .ch-title { font-size: 15px; font-weight: 800; letter-spacing: 0.3px; }
    .ch1-title { color: #bc4c00; }
    .ch2-title { color: #1a7f37; }

    .param-label {
        font-size: 10px;
        text-transform: uppercase;
        letter-spacing: 1.2px;
        color: #656d76;
        font-weight: 600;
    }
    .param-unit {
        font-size: 11px;
        color: #afb8c1;
        font-weight: 700;
        font-family: monospace;
    }

    .freq-display {
        font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
        font-size: 20px;
        font-weight: 700;
        color: #0969da;
        background-color: #f6f8fa;
        border: 1px solid #d0d7de;
        border-radius: 6px;
        padding: 6px 12px;
    }

    .freq-preset {
        background-color: #f6f8fa;
        color: #656d76;
        border: 1px solid #d0d7de;
        border-radius: 4px;
        padding: 2px 6px;
        font-size: 10px;
        font-weight: 600;
        font-family: monospace;
        min-height: 24px;
    }
    .freq-preset:hover {
        border-color: #0969da;
        color: #0969da;
        background-color: #ddf4ff;
    }

    .freq-entry {
        background-color: #ffffff;
        color: #0969da;
        border: 1px solid #d0d7de;
        border-radius: 6px;
        padding: 6px 10px;
        font-family: 'JetBrains Mono', 'Fira Code', monospace;
        font-size: 14px;
        font-weight: 600;
        min-height: 30px;
    }
    .freq-entry:focus {
        border-color: #0969da;
        box-shadow: 0 0 0 2px rgba(9, 105, 218, 0.2);
    }

    .freq-unit-combo {
        background-color: #ffffff;
        color: #656d76;
        border: 1px solid #d0d7de;
        border-radius: 6px;
        padding: 4px 8px;
        font-family: monospace;
        font-size: 12px;
        font-weight: 600;
        min-width: 60px;
        min-height: 30px;
    }
    .freq-unit-combo:hover { border-color: #0969da; }

    .param-spin {
        background-color: #ffffff;
        color: #1f2328;
        border: 1px solid #d0d7de;
        border-radius: 6px;
        padding: 4px 8px;
        font-family: monospace;
        font-size: 13px;
        min-height: 30px;
    }
    .param-spin:focus { border-color: #0969da; }

    .waveform-select {
        background-color: #ffffff;
        color: #1f2328;
        border: 1px solid #d0d7de;
        border-radius: 6px;
        padding: 4px 8px;
        font-size: 12px;
        min-height: 30px;
    }

    .scope-preview {
        background-color: #010409;
        border: 1px solid #d0d7de;
        border-radius: 6px;
    }

    .footer-bar {
        background-color: #ffffff;
        border: 1px solid #d0d7de;
        border-radius: 8px;
        padding: 8px 12px;
    }
    .presets-title {
        font-size: 10px;
        text-transform: uppercase;
        letter-spacing: 2px;
        color: #afb8c1;
        font-weight: 800;
    }
    .preset-btn {
        min-width: 36px;
        min-height: 36px;
        border: 1px dashed #d0d7de;
        border-radius: 6px;
        background-color: #f6f8fa;
        color: #afb8c1;
        font-weight: 800;
        font-size: 13px;
        font-family: monospace;
    }
    .preset-btn:hover {
        border-color: #0969da;
        color: #0969da;
        background-color: #ddf4ff;
    }
    .preset-btn.saved {
        border-style: solid;
        border-color: #1a7f37;
        color: #1a7f37;
        background-color: rgba(26,127,55,0.06);
    }
    .preset-btn.saved:hover { background-color: rgba(26,127,55,0.12); }

    .btn-action {
        background-color: #f6f8fa;
        color: #1f2328;
        border: 1px solid #d0d7de;
        border-radius: 6px;
        padding: 6px 14px;
        font-weight: 600;
        font-size: 12px;
    }
    .btn-action:hover { border-color: #0969da; color: #0969da; }

    .btn-danger {
        background-color: #cf222e;
        color: #ffffff;
        font-weight: 700;
        border-radius: 6px;
        padding: 6px 14px;
        font-size: 12px;
        border: 1px solid #cf222e;
    }
    .btn-danger:hover { background-color: #a40e26; }

    .toast-frame {
        background-color: #ffffff;
        border: 1px solid #d0d7de;
        border-radius: 8px;
        padding: 8px 20px;
        box-shadow: 0 4px 12px rgba(0,0,0,0.1);
    }
    .toast-label { color: #1f2328; font-weight: 600; font-size: 13px; }
    .toast-error { color: #cf222e; }

    separator { background-color: #d0d7de; }
    switch slider { background-color: #afb8c1; }
    switch:checked slider { background-color: #1a7f37; }
    switch trough { background-color: #eaeef2; }
    switch:checked trough { background-color: rgba(26,127,55,0.25); }
    "#
}

pub fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("JDS6600 — Generador de Señales")
        .default_width(1100)
        .default_height(750)
        .build();

    let is_dark = Rc::new(RefCell::new(true));

    let header = HeaderBar::new();
    header.set_show_title_buttons(true);

    let title_label = Label::new(Some("JDS6600"));
    title_label.add_css_class("title");
    header.set_title_widget(Some(&title_label));

    let provider = gtk4::CssProvider::new();
    provider.load_from_data(dark_css());
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("No display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let status_dot = Label::new(Some("●"));
    status_dot.add_css_class("status-dot");
    let status_text = Label::new(Some("Desconectado"));
    status_text.add_css_class("status-text");
    let status_box = GtkBox::new(Orientation::Horizontal, 6);
    status_box.set_valign(Align::Center);
    status_box.append(&status_dot);
    status_box.append(&status_text);
    header.pack_start(&status_box);

    let port_combo = ComboBoxText::new();
    port_combo.add_css_class("port-combo");
    header.pack_end(&port_combo);

    let btn_scan = Button::with_label("Escanear");
    btn_scan.set_tooltip_text(Some("Escanear y auto-detectar JDS6600"));
    btn_scan.add_css_class("btn-scan");
    header.pack_end(&btn_scan);

    let btn_connect = Button::with_label("Conectar");
    btn_connect.add_css_class("btn-connect");
    header.pack_end(&btn_connect);

    let btn_theme = Button::with_label("☀");
    btn_theme.set_tooltip_text(Some("Cambiar tema claro/oscuro"));
    btn_theme.add_css_class("btn-theme");
    header.pack_end(&btn_theme);

    window.set_titlebar(Some(&header));

    let overlay = Overlay::new();
    window.set_child(Some(&overlay));

    let main_vbox = GtkBox::new(Orientation::Vertical, 0);
    overlay.set_child(Some(&main_vbox));

    let driver = Arc::new(Mutex::new(Jds6600::new("/dev/ttyUSB0")));
    let connected = Rc::new(RefCell::new(false));

    let toast_revealer = Revealer::new();
    toast_revealer.set_transition_type(gtk4::RevealerTransitionType::SlideUp);
    toast_revealer.set_transition_duration(250);
    toast_revealer.set_reveal_child(false);
    toast_revealer.set_halign(Align::Center);
    toast_revealer.set_valign(Align::End);
    toast_revealer.set_margin_bottom(16);
    let toast_frame = Frame::new(None);
    toast_frame.add_css_class("toast-frame");
    let toast_label = Label::new(None);
    toast_label.add_css_class("toast-label");
    toast_frame.set_child(Some(&toast_label));
    toast_revealer.set_child(Some(&toast_frame));
    overlay.add_overlay(&toast_revealer);

    let show_toast = {
        let rev = toast_revealer.clone();
        let lbl = toast_label.clone();
        move |msg: &str, is_error: bool| {
            lbl.set_text(msg);
            if is_error {
                lbl.add_css_class("toast-error");
            } else {
                lbl.remove_css_class("toast-error");
            }
            rev.set_reveal_child(true);
            let rev2 = rev.clone();
            timeout_add_local(Duration::from_millis(2500), move || {
                rev2.set_reveal_child(false);
                glib::ControlFlow::Break
            });
        }
    };

    let channels_box = GtkBox::new(Orientation::Horizontal, 12);
    channels_box.set_margin_top(12);
    channels_box.set_margin_bottom(8);
    channels_box.set_margin_start(12);
    channels_box.set_margin_end(12);
    channels_box.set_homogeneous(true);
    main_vbox.append(&channels_box);

    fn build_channel_panel(
        ch_num: u8,
    ) -> (Frame, Switch, ComboBoxText, gtk4::Adjustment, gtk4::Adjustment, gtk4::Adjustment, gtk4::Adjustment, Entry, ComboBoxText, DrawingArea, SpinButton) {
        let frame = Frame::new(None);
        frame.add_css_class("channel-frame");
        frame.add_css_class(if ch_num == 1 { "ch1-frame" } else { "ch2-frame" });
        frame.set_vexpand(true);

        let outer = GtkBox::new(Orientation::Vertical, 0);
        outer.set_margin_top(12);
        outer.set_margin_bottom(12);
        outer.set_margin_start(14);
        outer.set_margin_end(14);
        frame.set_child(Some(&outer));

        let title_row = GtkBox::new(Orientation::Horizontal, 0);
        let ch_title = Label::new(Some(&format!("CANAL {}", ch_num)));
        ch_title.add_css_class("ch-title");
        ch_title.add_css_class(if ch_num == 1 { "ch1-title" } else { "ch2-title" });
        ch_title.set_hexpand(true);
        ch_title.set_xalign(0.0);
        title_row.append(&ch_title);
        let toggle = Switch::new();
        toggle.set_valign(Align::Center);
        title_row.append(&toggle);
        outer.append(&title_row);

        let sep = Separator::new(Orientation::Horizontal);
        sep.set_margin_top(8);
        sep.set_margin_bottom(8);
        outer.append(&sep);

        let grid = Grid::new();
        grid.set_row_spacing(6);
        grid.set_column_spacing(8);
        outer.append(&grid);

        let wave_label = Label::new(Some("FORMA DE ONDA"));
        wave_label.add_css_class("param-label");
        wave_label.set_xalign(0.0);
        wave_label.set_halign(Align::Start);
        wave_label.set_valign(Align::Center);
        grid.attach(&wave_label, 0, 0, 1, 1);

        let wave = ComboBoxText::new();
        for &w in WAVEFORMS {
            wave.append(Some(w), &waveform_display(w));
        }
        wave.set_active_id(Some("sine"));
        wave.add_css_class("waveform-select");
        wave.set_hexpand(true);
        grid.attach(&wave, 1, 0, 2, 1);

        let freq_label = Label::new(Some("FRECUENCIA"));
        freq_label.add_css_class("param-label");
        freq_label.set_xalign(0.0);
        freq_label.set_halign(Align::Start);
        freq_label.set_valign(Align::Center);
        grid.attach(&freq_label, 0, 1, 1, 1);

        let freq_adj = gtk4::Adjustment::new(1000.0, FREQ_MIN_HZ, FREQ_MAX_HZ, 1.0, 1000.0, 0.0);
        
        let freq_box = GtkBox::new(Orientation::Horizontal, 4);
        freq_box.set_hexpand(true);
        
        let freq_entry = Entry::new();
        freq_entry.set_text("1000");
        freq_entry.set_width_chars(10);
        freq_entry.add_css_class("freq-entry");
        freq_entry.set_hexpand(true);
        freq_box.append(&freq_entry);
        
        let freq_unit_combo = ComboBoxText::new();
        freq_unit_combo.append(Some("hz"), "Hz");
        freq_unit_combo.append(Some("khz"), "kHz");
        freq_unit_combo.append(Some("mhz"), "MHz");
        freq_unit_combo.set_active_id(Some("hz"));
        freq_unit_combo.add_css_class("freq-unit-combo");
        freq_box.append(&freq_unit_combo);
        
        grid.attach(&freq_box, 1, 1, 2, 1);

        let presets_box = GtkBox::new(Orientation::Horizontal, 3);
        presets_box.set_halign(Align::Center);
        presets_box.set_hexpand(true);
        let freq_presets: &[(f64, &str)] = &[
            (50.0, "50Hz"), (100.0, "100Hz"), (1000.0, "1kHz"),
            (10_000.0, "10kHz"), (100_000.0, "100kHz"),
            (1_000_000.0, "1MHz"), (10_000_000.0, "10MHz"),
        ];
        for &(freq, label) in freq_presets {
            let btn = Button::with_label(label);
            btn.add_css_class("freq-preset");
            let adj = freq_adj.clone();
            btn.connect_clicked(move |_| { adj.set_value(freq); });
            presets_box.append(&btn);
        }
        grid.attach(&presets_box, 0, 2, 3, 1);

        let amp_label = Label::new(Some("AMPLITUD"));
        amp_label.add_css_class("param-label");
        amp_label.set_xalign(0.0);
        amp_label.set_halign(Align::Start);
        grid.attach(&amp_label, 0, 3, 1, 1);
        let amp_adj = gtk4::Adjustment::new(1.0, AMP_MIN_V, AMP_MAX_V, 0.001, 0.1, 0.0);
        let amp_spin = SpinButton::new(Some(&amp_adj), 0.001, 3);
        amp_spin.set_width_chars(10);
        amp_spin.add_css_class("param-spin");
        amp_spin.set_hexpand(true);
        grid.attach(&amp_spin, 1, 3, 1, 1);
        let amp_unit = Label::new(Some("V"));
        amp_unit.add_css_class("param-unit");
        amp_unit.set_width_chars(2);
        grid.attach(&amp_unit, 2, 3, 1, 1);

        let off_label = Label::new(Some("OFFSET"));
        off_label.add_css_class("param-label");
        off_label.set_xalign(0.0);
        off_label.set_halign(Align::Start);
        grid.attach(&off_label, 0, 4, 1, 1);
        let off_adj = gtk4::Adjustment::new(0.0, OFFSET_MIN_V, OFFSET_MAX_V, 0.01, 0.1, 0.0);
        let off_spin = SpinButton::new(Some(&off_adj), 0.01, 2);
        off_spin.set_width_chars(10);
        off_spin.add_css_class("param-spin");
        grid.attach(&off_spin, 1, 4, 1, 1);
        let off_unit = Label::new(Some("V"));
        off_unit.add_css_class("param-unit");
        grid.attach(&off_unit, 2, 4, 1, 1);

        let duty_label = Label::new(Some("CICLO TRABAJO"));
        duty_label.add_css_class("param-label");
        duty_label.set_xalign(0.0);
        duty_label.set_halign(Align::Start);
        grid.attach(&duty_label, 0, 5, 1, 1);
        let duty_adj = gtk4::Adjustment::new(50.0, 0.0, 100.0, 0.1, 1.0, 0.0);
        let duty_spin = SpinButton::new(Some(&duty_adj), 0.1, 1);
        duty_spin.set_width_chars(10);
        duty_spin.add_css_class("param-spin");
        grid.attach(&duty_spin, 1, 5, 1, 1);
        let duty_unit = Label::new(Some("%"));
        duty_unit.add_css_class("param-unit");
        grid.attach(&duty_unit, 2, 5, 1, 1);

        let preview = DrawingArea::new();
        preview.set_content_width(400);
        preview.set_content_height(120);
        preview.set_margin_top(8);
        preview.set_hexpand(true);
        preview.set_vexpand(true);
        preview.add_css_class("scope-preview");
        grid.attach(&preview, 0, 6, 3, 1);

        (frame, toggle, wave, freq_adj, amp_adj, off_adj, duty_adj, freq_entry, freq_unit_combo, preview, duty_spin)
    }

    let (ch1_frame, ch1_toggle, ch1_wave, ch1_freq_adj, ch1_amp_adj, ch1_off_adj, ch1_duty_adj, ch1_freq_entry, ch1_freq_unit, ch1_preview, ch1_duty_spin) =
        build_channel_panel(1);
    channels_box.append(&ch1_frame);

    let (ch2_frame, ch2_toggle, ch2_wave, ch2_freq_adj, ch2_amp_adj, ch2_off_adj, ch2_duty_adj, ch2_freq_entry, ch2_freq_unit, ch2_preview, ch2_duty_spin) =
        build_channel_panel(2);
    channels_box.append(&ch2_frame);

    let footer = GtkBox::new(Orientation::Horizontal, 8);
    footer.set_margin_top(4);
    footer.set_margin_bottom(12);
    footer.set_margin_start(12);
    footer.set_margin_end(12);
    footer.add_css_class("footer-bar");
    main_vbox.append(&footer);

    let presets_title = Label::new(Some("PRESETS"));
    presets_title.add_css_class("presets-title");
    presets_title.set_valign(Align::Center);
    footer.append(&presets_title);

    let presets = Arc::new(Mutex::new(PresetBank::load_or_default(PRESETS_PATH)));
    let preset_buttons: Rc<RefCell<Vec<Button>>> = Rc::new(RefCell::new(Vec::new()));

    for i in 1..=8 {
        let btn = Button::with_label(&i.to_string());
        btn.set_tooltip_text(Some("Click: cargar/guardar | Click derecho: borrar"));
        btn.add_css_class("preset-btn");
        let has = presets.lock().unwrap().slots.get(&i.to_string()).unwrap_or(&None).is_some();
        if has { btn.add_css_class("saved"); }
        footer.append(&btn);
        preset_buttons.borrow_mut().push(btn.clone());

        let presets_ref = presets.clone();
        let drv = driver.clone();
        let btn_ref = btn.clone();

        btn.connect_clicked(move |_b| {
            let has = presets_ref.lock().unwrap().slots.get(&i.to_string()).unwrap_or(&None).is_some();
            if has {
                let preset = presets_ref.lock().unwrap().slots.get(&i.to_string()).unwrap_or(&None).clone();
                if let Some(p) = preset {
                    let drv = drv.clone();
                    std::thread::spawn(move || {
                        let mut d = drv.lock().unwrap();
                        let _ = d.set_waveform(1, &p.ch1.waveform);
                        let _ = d.set_frequency(1, p.ch1.frequency);
                        let _ = d.set_amplitude(1, p.ch1.amplitude);
                        let _ = d.set_offset(1, p.ch1.offset);
                        let _ = d.set_dutycycle(1, p.ch1.duty_cycle);
                        let _ = d.set_waveform(2, &p.ch2.waveform);
                        let _ = d.set_frequency(2, p.ch2.frequency);
                        let _ = d.set_amplitude(2, p.ch2.amplitude);
                        let _ = d.set_offset(2, p.ch2.offset);
                        let _ = d.set_dutycycle(2, p.ch2.duty_cycle);
                        let _ = d.set_channels(p.ch1.enabled, p.ch2.enabled);
                    });
                }
            } else {
                if let Ok(state) = drv.lock().unwrap().get_full_state() {
                    let preset = crate::model::Preset {
                        ch1: state.ch1.into(),
                        ch2: state.ch2.into(),
                    };
                    presets_ref.lock().unwrap().slots.insert(i.to_string(), Some(preset));
                    let _ = presets_ref.lock().unwrap().save(PRESETS_PATH);
                    btn_ref.add_css_class("saved");
                }
            }
        });

        let gesture = GestureClick::new();
        gesture.set_button(3);
        let presets_ref = presets.clone();
        let pbtns = preset_buttons.clone();
        gesture.connect_pressed(move |_gesture, _n_press, _x, _y| {
            let has = presets_ref.lock().unwrap().slots.get(&i.to_string()).unwrap_or(&None).is_some();
            if has {
                presets_ref.lock().unwrap().slots.insert(i.to_string(), None);
                let _ = presets_ref.lock().unwrap().save(PRESETS_PATH);
                glib::source::idle_add_local({
                    let pbtns = pbtns.clone();
                    let presets_ref = presets_ref.clone();
                    move || {
                        for (idx, btn) in pbtns.borrow().iter().enumerate() {
                            let num = idx + 1;
                            let has = presets_ref.lock().unwrap().slots.get(&num.to_string()).unwrap_or(&None).is_some();
                            if has { btn.add_css_class("saved"); } else { btn.remove_css_class("saved"); }
                        }
                        glib::ControlFlow::Break
                    }
                });
            }
        });
        btn.add_controller(gesture);
    }

    footer.append(&Label::new(None));

    let btn_sync = Button::with_label("Sync 1→2");
    btn_sync.set_tooltip_text(Some("Copiar configuración del Canal 1 al Canal 2"));
    btn_sync.add_css_class("btn-action");
    footer.append(&btn_sync);

    let btn_all_off = Button::with_label("Apagar Todo");
    btn_all_off.add_css_class("btn-danger");
    footer.append(&btn_all_off);

    let update_ui = {
        let ch1_t = ch1_toggle.clone();
        let ch2_t = ch2_toggle.clone();
        let ch1_w = ch1_wave.clone();
        let ch2_w = ch2_wave.clone();
        // No necesitamos ch1_f/ch2_f porque el polling solo actualiza el Entry, no el Adjustment
        let ch1_a = ch1_amp_adj.clone();
        let ch2_a = ch2_amp_adj.clone();
        let ch1_o = ch1_off_adj.clone();
        let ch2_o = ch2_off_adj.clone();
        let ch1_d = ch1_duty_adj.clone();
        let ch2_d = ch2_duty_adj.clone();
        let ch1_fe = ch1_freq_entry.clone();
        let ch2_fe = ch2_freq_entry.clone();
        let ch1_fu = ch1_freq_unit.clone();
        let ch2_fu = ch2_freq_unit.clone();
        let dot = status_dot.clone();
        let txt = status_text.clone();
        let btn = btn_connect.clone();
        let ch1_frm = ch1_frame.clone();
        let ch2_frm = ch2_frame.clone();

        move |state: crate::driver::FullState| {
            ch1_t.set_active(state.ch1.enabled);
            ch2_t.set_active(state.ch2.enabled);
            ch1_w.set_active_id(Some(&state.ch1.waveform));
            ch2_w.set_active_id(Some(&state.ch2.waveform));
            // NO actualizar Adjustments de frecuencia desde polling para evitar loops
            // ch1_f.set_value(state.ch1.frequency);
            // ch2_f.set_value(state.ch2.frequency);
            ch1_a.set_value(state.ch1.amplitude);
            ch2_a.set_value(state.ch2.amplitude);
            ch1_o.set_value(state.ch1.offset);
            ch2_o.set_value(state.ch2.offset);
            ch1_d.set_value(state.ch1.duty_cycle);
            ch2_d.set_value(state.ch2.duty_cycle);
            
            // Actualizar SOLO el Entry de frecuencia según la unidad seleccionada
            let ch1_unit = ch1_fu.active_id().map(|s| s.to_string()).unwrap_or_else(|| "hz".to_string());
            let ch2_unit = ch2_fu.active_id().map(|s| s.to_string()).unwrap_or_else(|| "hz".to_string());
            let ch1_val = match ch1_unit.as_str() {
                "khz" => state.ch1.frequency / 1_000.0,
                "mhz" => state.ch1.frequency / 1_000_000.0,
                _ => state.ch1.frequency,
            };
            let ch2_val = match ch2_unit.as_str() {
                "khz" => state.ch2.frequency / 1_000.0,
                "mhz" => state.ch2.frequency / 1_000_000.0,
                _ => state.ch2.frequency,
            };
            ch1_fe.set_text(&format!("{:.4}", ch1_val).trim_end_matches('0').trim_end_matches('.').to_string());
            ch2_fe.set_text(&format!("{:.4}", ch2_val).trim_end_matches('0').trim_end_matches('.').to_string());

            if state.connected {
                dot.add_css_class("on");
                txt.add_css_class("on");
                txt.set_text(&format!("Conectado — {}", state.port));
                btn.set_label("Desconectar");
                btn.add_css_class("disconnect");
                if state.ch1.enabled { ch1_frm.add_css_class("on"); } else { ch1_frm.remove_css_class("on"); }
                if state.ch2.enabled { ch2_frm.add_css_class("on"); } else { ch2_frm.remove_css_class("on"); }
            } else {
                dot.remove_css_class("on");
                txt.remove_css_class("on");
                txt.set_text("Desconectado");
                btn.set_label("Conectar");
                btn.remove_css_class("disconnect");
                ch1_frm.remove_css_class("on");
                ch2_frm.remove_css_class("on");
            }

            ch1_frm.queue_draw();
            ch2_frm.queue_draw();
        }
    };

    let refresh_ports = {
        let combo = port_combo.clone();
        move || {
            let ports = crate::driver::detect_serial_ports();
            let prev = combo.active_id();
            combo.remove_all();
            if ports.is_empty() {
                combo.append(Some(""), "— Sin puertos —");
                combo.set_active_id(Some(""));
            } else {
                for p in &ports {
                    combo.append(Some(p), p);
                }
                if let Some(ref id) = prev {
                    let id_str = id.to_string();
                    if ports.contains(&id_str) { combo.set_active_id(Some(&id_str)); }
                    else { combo.set_active(Some(0)); }
                } else {
                    combo.set_active(Some(0));
                }
            }
            ports
        }
    };

    refresh_ports();

    btn_scan.connect_clicked({
        let refresh = refresh_ports.clone();
        let show_t = show_toast.clone();
        let combo = port_combo.clone();
        move |_btn| {
            show_t("Analizando puertos en busca de JDS6600...", false);
            let result: Arc<Mutex<Option<Option<String>>>> = Arc::new(Mutex::new(None));
            let result_thread = result.clone();
            std::thread::spawn(move || {
                let detected = crate::driver::auto_detect_port();
                *result_thread.lock().unwrap() = Some(detected);
            });
            let refresh2 = refresh.clone();
            let combo2 = combo.clone();
            let show_t2 = show_t.clone();
            glib::source::timeout_add_local(Duration::from_millis(50), move || {
                if let Some(detected) = result.lock().unwrap().take() {
                    let ports = refresh2();
                    if let Some(port) = detected {
                        combo2.set_active_id(Some(&port));
                        show_t2(&format!("JDS6600 detectado en {}", port), false);
                    } else {
                        if ports.is_empty() {
                            show_t2("No se detectaron puertos seriales. Conecte el dispositivo.", true);
                        } else {
                            show_t2("No se detectó JDS6600 automáticamente. Seleccione manualmente.", true);
                        }
                    }
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        }
    });

    {
        let drv = driver.clone();
        let connected_flag = connected.clone();
        let update_ui = update_ui.clone();
        let show_t = show_toast.clone();
        let combo = port_combo.clone();
        btn_connect.connect_clicked(move |_btn| {
            let is_connected = *connected_flag.borrow();
            let drv = drv.clone();
            let show_t = show_t.clone();
            let update_ui = update_ui.clone();
            let combo = combo.clone();
            let connected_flag = connected_flag.clone();

            if is_connected {
                std::thread::spawn(move || {
                    let mut d = drv.lock().unwrap();
                    d.disconnect();
                });
                *connected_flag.borrow_mut() = false;
                update_ui(crate::driver::FullState::default());
                show_t("Desconectado", false);
            } else {
                let port = combo.active_id().map(|s| s.to_string()).unwrap_or_default();
                if port.is_empty() {
                    show_t("Seleccione un puerto primero. Use 'Escanear' para detectar.", true);
                    return;
                }
                let port_msg = port.clone();
                show_t(&format!("Conectando a {}...", port_msg), false);
                let result: Arc<Mutex<Option<Result<crate::driver::FullState, String>>>> = Arc::new(Mutex::new(None));
                let result_thread = result.clone();
                std::thread::spawn(move || {
                    let mut d = drv.lock().unwrap();
                    let res = match d.connect(&port) {
                        Ok(()) => {
                            match d.get_full_state() {
                                Ok(state) => Ok(state),
                                Err(e) => { d.disconnect(); Err(format!("El dispositivo no responde en {}: {}", port, e)) }
                            }
                        }
                        Err(e) => Err(format!("Error de conexión: {}", e)),
                    };
                    *result_thread.lock().unwrap() = Some(res);
                });
                glib::source::timeout_add_local(Duration::from_millis(50), move || {
                    if let Some(res) = result.lock().unwrap().take() {
                        match res {
                            Ok(state) => {
                                show_t(&format!("Conectado a {}", port_msg), false);
                                update_ui(state);
                                *connected_flag.borrow_mut() = true;
                            }
                            Err(msg) => {
                                show_t(&msg, true);
                                update_ui(crate::driver::FullState::default());
                            }
                        }
                        glib::ControlFlow::Break
                    } else {
                        glib::ControlFlow::Continue
                    }
                });
            }
        });
    }

    {
        let drv = driver.clone();
        let show_t = show_toast.clone();
        btn_sync.connect_clicked(move |_btn| {
            let drv = drv.clone();
            let show_t = show_t.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                match d.sync_channels() {
                    Ok(()) => {}
                    Err(e) => { eprintln!("Sync error: {}", e); }
                }
            });
            show_t("Canal 1 copiado al Canal 2", false);
        });
    }

    {
        let drv = driver.clone();
        let show_t = show_toast.clone();
        btn_all_off.connect_clicked(move |_btn| {
            let drv = drv.clone();
            let show_t = show_t.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                let _ = d.all_off();
            });
            show_t("Canales apagados", false);
        });
    }

    {
        let prov = provider.clone();
        let dark = is_dark.clone();
        let btn = btn_theme.clone();
        btn_theme.connect_clicked(move |_| {
            let mut d = dark.borrow_mut();
            *d = !*d;
            if *d {
                prov.load_from_data(dark_css());
                btn.set_label("☀");
            } else {
                prov.load_from_data(light_css());
                btn.set_label("☾");
            }
        });
    }

    {
        let drv = driver.clone();
        let connected_flag = connected.clone();
        let update_ui = update_ui.clone();
        let show_t = show_toast.clone();
        timeout_add_local(Duration::from_millis(500), move || {
            let mut d = drv.lock().unwrap();
            if d.is_connected() {
                match d.get_full_state() {
                    Ok(state) => {
                        *connected_flag.borrow_mut() = true;
                        update_ui(state);
                    }
                    Err(e) => {
                        eprintln!("Polling error: {}. Desconectando.", e);
                        d.disconnect();
                        *connected_flag.borrow_mut() = false;
                        glib::source::idle_add_local({
                            let show_t = show_t.clone();
                            let update_ui = update_ui.clone();
                            move || {
                                show_t("Conexión perdida. Desconectado.", true);
                                update_ui(crate::driver::FullState::default());
                                glib::ControlFlow::Break
                            }
                        });
                    }
                }
            }
            glib::ControlFlow::Continue
        });
    }

    {
        let drv = driver.clone();
        ch1_toggle.connect_state_set(move |_toggle, is_active| {
            let drv = drv.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                if let Ok((_, ch2_on)) = d.get_channels() {
                    let _ = d.set_channels(is_active, ch2_on);
                }
            });
            glib::Propagation::Stop
        });
    }
    {
        let drv = driver.clone();
        let duty = ch1_duty_spin.clone();
        let preview = ch1_preview.clone();
        ch1_wave.connect_changed(move |combo| {
            if let Some(id) = combo.active_id() {
                let name = id.to_string();
                let has_duty = waveform_has_duty(&name);
                let drv = drv.clone();
                std::thread::spawn(move || {
                    let mut d = drv.lock().unwrap();
                    let _ = d.set_waveform(1, &name);
                });
                duty.set_sensitive(has_duty);
                preview.queue_draw();
            }
        });
    }
    {
        let drv = driver.clone();
        let entry = ch1_freq_entry.clone();
        let unit_combo = ch1_freq_unit.clone();
        let adj = ch1_freq_adj.clone();
        
        // Flag para evitar actualizaciones mientras el usuario edita
        let is_editing = Rc::new(RefCell::new(false));
        
        // Controller para detectar foco y seleccionar todo el texto
        let focus_controller = gtk4::EventControllerFocus::new();
        let is_editing_in = is_editing.clone();
        let entry_sel = entry.clone();
        focus_controller.connect_enter(move |_| {
            *is_editing_in.borrow_mut() = true;
            // Seleccionar todo el texto para facilitar edición
            entry_sel.select_region(0, -1);
        });
        let is_editing_out = is_editing.clone();
        focus_controller.connect_leave(move |_| {
            *is_editing_out.borrow_mut() = false;
        });
        entry.add_controller(focus_controller);
        
        // Callback cuando el usuario presiona Enter en el Entry
        let adj2 = adj.clone();
        let unit_combo2 = unit_combo.clone();
        let is_editing_enter = is_editing.clone();
        entry.connect_activate(move |e| {
            *is_editing_enter.borrow_mut() = false;
            let text = e.text().to_string();
            let unit = unit_combo2.active_id().map(|s| s.to_string()).unwrap_or_else(|| "hz".to_string());
            if let Ok(val) = text.parse::<f64>() {
                let hz = match unit.as_str() {
                    "khz" => val * 1_000.0,
                    "mhz" => val * 1_000_000.0,
                    _ => val,
                };
                let hz = hz.clamp(FREQ_MIN_HZ, FREQ_MAX_HZ);
                adj2.set_value(hz);
                // connect_value_changed se encargará de enviar al generador
            }
        });
        
        // Callback cuando cambia la unidad - con validación de límites
        let entry3 = entry.clone();
        let adj3 = adj.clone();
        unit_combo.connect_changed(move |combo| {
            let unit = combo.active_id().map(|s| s.to_string()).unwrap_or_else(|| "hz".to_string());
            let hz = adj3.value();
            let val = match unit.as_str() {
                "khz" => hz / 1_000.0,
                "mhz" => hz / 1_000_000.0,
                _ => hz,
            };
            // Validar que el valor en la nueva unidad no exceda los límites
            let max_val = match unit.as_str() {
                "khz" => FREQ_MAX_HZ / 1_000.0,
                "mhz" => FREQ_MAX_HZ / 1_000_000.0,
                _ => FREQ_MAX_HZ,
            };
            let min_val = match unit.as_str() {
                "khz" => FREQ_MIN_HZ / 1_000.0,
                "mhz" => FREQ_MIN_HZ / 1_000_000.0,
                _ => FREQ_MIN_HZ,
            };
            let val = val.clamp(min_val, max_val);
            entry3.set_text(&format!("{:.4}", val).trim_end_matches('0').trim_end_matches('.').to_string());
        });
        
        // Callback cuando cambia el adjustment (desde presets o entrada manual)
        let entry4 = entry.clone();
        let unit_combo4 = unit_combo.clone();
        let drv4 = drv.clone();
        let is_editing_update = is_editing.clone();
        adj.connect_value_changed(move |a| {
            // No actualizar el Entry si el usuario está editando
            if *is_editing_update.borrow() {
                return;
            }
            
            let hz = a.value();
            eprintln!("[DEBUG CH1] connect_value_changed disparado: {} Hz", hz);
            let unit = unit_combo4.active_id().map(|s| s.to_string()).unwrap_or_else(|| "hz".to_string());
            let val = match unit.as_str() {
                "khz" => hz / 1_000.0,
                "mhz" => hz / 1_000_000.0,
                _ => hz,
            };
            entry4.set_text(&format!("{:.4}", val).trim_end_matches('0').trim_end_matches('.').to_string());
            let drv = drv4.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                eprintln!("[DEBUG CH1] Enviando set_frequency(1, {}) al generador", hz);
                let _ = d.set_frequency(1, hz);
            });
        });
    }
    {
        let drv = driver.clone();
        ch1_amp_adj.connect_value_changed(move |adj| {
            let v = adj.value();
            let drv = drv.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                let _ = d.set_amplitude(1, v);
            });
        });
    }
    {
        let drv = driver.clone();
        ch1_off_adj.connect_value_changed(move |adj| {
            let v = adj.value();
            let drv = drv.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                let _ = d.set_offset(1, v);
            });
        });
    }
    {
        let drv = driver.clone();
        let preview = ch1_preview.clone();
        ch1_duty_adj.connect_value_changed(move |adj| {
            let v = adj.value();
            let drv = drv.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                let _ = d.set_dutycycle(1, v);
            });
            preview.queue_draw();
        });
    }

    {
        let drv = driver.clone();
        ch2_toggle.connect_state_set(move |_toggle, is_active| {
            let drv = drv.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                if let Ok((ch1_on, _)) = d.get_channels() {
                    let _ = d.set_channels(ch1_on, is_active);
                }
            });
            glib::Propagation::Stop
        });
    }
    {
        let drv = driver.clone();
        let duty = ch2_duty_spin.clone();
        let preview = ch2_preview.clone();
        ch2_wave.connect_changed(move |combo| {
            if let Some(id) = combo.active_id() {
                let name = id.to_string();
                let has_duty = waveform_has_duty(&name);
                let drv = drv.clone();
                std::thread::spawn(move || {
                    let mut d = drv.lock().unwrap();
                    let _ = d.set_waveform(2, &name);
                });
                duty.set_sensitive(has_duty);
                preview.queue_draw();
            }
        });
    }
    {
        let drv = driver.clone();
        let entry = ch2_freq_entry.clone();
        let unit_combo = ch2_freq_unit.clone();
        let adj = ch2_freq_adj.clone();
        
        // Flag para evitar actualizaciones mientras el usuario edita
        let is_editing = Rc::new(RefCell::new(false));
        
        // Controller para detectar foco y seleccionar todo el texto
        let focus_controller = gtk4::EventControllerFocus::new();
        let is_editing_in = is_editing.clone();
        let entry_sel = entry.clone();
        focus_controller.connect_enter(move |_| {
            *is_editing_in.borrow_mut() = true;
            // Seleccionar todo el texto para facilitar edición
            entry_sel.select_region(0, -1);
        });
        let is_editing_out = is_editing.clone();
        focus_controller.connect_leave(move |_| {
            *is_editing_out.borrow_mut() = false;
        });
        entry.add_controller(focus_controller);
        
        // Callback cuando el usuario presiona Enter en el Entry
        let adj2 = adj.clone();
        let unit_combo2 = unit_combo.clone();
        let is_editing_enter = is_editing.clone();
        entry.connect_activate(move |e| {
            *is_editing_enter.borrow_mut() = false;
            let text = e.text().to_string();
            let unit = unit_combo2.active_id().map(|s| s.to_string()).unwrap_or_else(|| "hz".to_string());
            if let Ok(val) = text.parse::<f64>() {
                let hz = match unit.as_str() {
                    "khz" => val * 1_000.0,
                    "mhz" => val * 1_000_000.0,
                    _ => val,
                };
                let hz = hz.clamp(FREQ_MIN_HZ, FREQ_MAX_HZ);
                adj2.set_value(hz);
                // connect_value_changed se encargará de enviar al generador
            }
        });
        
        // Callback cuando cambia la unidad - con validación de límites
        let entry3 = entry.clone();
        let adj3 = adj.clone();
        unit_combo.connect_changed(move |combo| {
            let unit = combo.active_id().map(|s| s.to_string()).unwrap_or_else(|| "hz".to_string());
            let hz = adj3.value();
            let val = match unit.as_str() {
                "khz" => hz / 1_000.0,
                "mhz" => hz / 1_000_000.0,
                _ => hz,
            };
            // Validar que el valor en la nueva unidad no exceda los límites
            let max_val = match unit.as_str() {
                "khz" => FREQ_MAX_HZ / 1_000.0,
                "mhz" => FREQ_MAX_HZ / 1_000_000.0,
                _ => FREQ_MAX_HZ,
            };
            let min_val = match unit.as_str() {
                "khz" => FREQ_MIN_HZ / 1_000.0,
                "mhz" => FREQ_MIN_HZ / 1_000_000.0,
                _ => FREQ_MIN_HZ,
            };
            let val = val.clamp(min_val, max_val);
            entry3.set_text(&format!("{:.4}", val).trim_end_matches('0').trim_end_matches('.').to_string());
        });
        
        // Callback cuando cambia el adjustment (desde presets o entrada manual)
        let entry4 = entry.clone();
        let unit_combo4 = unit_combo.clone();
        let drv4 = drv.clone();
        let is_editing_update = is_editing.clone();
        adj.connect_value_changed(move |a| {
            // No actualizar el Entry si el usuario está editando
            if *is_editing_update.borrow() {
                return;
            }
            
            let hz = a.value();
            eprintln!("[DEBUG CH2] connect_value_changed disparado: {} Hz", hz);
            let unit = unit_combo4.active_id().map(|s| s.to_string()).unwrap_or_else(|| "hz".to_string());
            let val = match unit.as_str() {
                "khz" => hz / 1_000.0,
                "mhz" => hz / 1_000_000.0,
                _ => hz,
            };
            entry4.set_text(&format!("{:.4}", val).trim_end_matches('0').trim_end_matches('.').to_string());
            let drv = drv4.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                eprintln!("[DEBUG CH2] Enviando set_frequency(2, {}) al generador", hz);
                let _ = d.set_frequency(2, hz);
            });
        });
    }
    {
        let drv = driver.clone();
        ch2_amp_adj.connect_value_changed(move |adj| {
            let v = adj.value();
            let drv = drv.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                let _ = d.set_amplitude(2, v);
            });
        });
    }
    {
        let drv = driver.clone();
        ch2_off_adj.connect_value_changed(move |adj| {
            let v = adj.value();
            let drv = drv.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                let _ = d.set_offset(2, v);
            });
        });
    }
    {
        let drv = driver.clone();
        let preview = ch2_preview.clone();
        ch2_duty_adj.connect_value_changed(move |adj| {
            let v = adj.value();
            let drv = drv.clone();
            std::thread::spawn(move || {
                let mut d = drv.lock().unwrap();
                let _ = d.set_dutycycle(2, v);
            });
            preview.queue_draw();
        });
    }

    {
        let w1 = ch1_wave.clone();
        let d1 = ch1_duty_adj.clone();
        ch1_preview.set_draw_func(move |_area, ctx, w, h| {
            let name = w1.active_id().map(|s| s.to_string()).unwrap_or_else(|| "sine".to_string());
            let duty = d1.value() / 100.0;
            draw_waveform_with_grid(ctx, w as f64, h as f64, &name, duty, 0.94, 0.53, 0.24);
        });
    }
    {
        let w2 = ch2_wave.clone();
        let d2 = ch2_duty_adj.clone();
        ch2_preview.set_draw_func(move |_area, ctx, w, h| {
            let name = w2.active_id().map(|s| s.to_string()).unwrap_or_else(|| "sine".to_string());
            let duty = d2.value() / 100.0;
            draw_waveform_with_grid(ctx, w as f64, h as f64, &name, duty, 0.25, 0.73, 0.31);
        });
    }

    window.present();
}
