use cairo::Context;
use rand::Rng;

/// Dibuja el fondo de grid tipo osciloscopio y luego la forma de onda
pub fn draw_waveform_with_grid(
    ctx: &Context,
    width: f64,
    height: f64,
    name: &str,
    duty: f64,
    r: f64,
    g: f64,
    b: f64,
) {
    // Fondo negro
    ctx.set_source_rgb(0.02, 0.02, 0.03);
    ctx.paint().unwrap_or_default();

    // Grid osciloscopio
    draw_grid(ctx, width, height);

    // Centro de referencia
    let cy = height / 2.0;
    ctx.set_source_rgb(0.15, 0.15, 0.2);
    ctx.set_line_width(1.0);
    ctx.move_to(0.0, cy);
    ctx.line_to(width, cy);
    ctx.stroke().unwrap_or_default();

    // Dibujar forma de onda con color del canal
    ctx.set_source_rgb(r, g, b);
    ctx.set_line_width(2.0);

    let pts = 300;
    match name {
        "sine" => draw_sine(ctx, width, height, pts),
        "square" => draw_square(ctx, width, height, duty),
        "pulse" => draw_square(ctx, width, height, 0.2),
        "triangle" => draw_triangle(ctx, width, height, pts),
        "parial_sine" => draw_partial_sine(ctx, width, height, pts),
        "cmos" => draw_cmos(ctx, width, height, pts),
        "dc" => draw_dc(ctx, width, height),
        "half_wave" => draw_partial_sine(ctx, width, height, pts),
        "full_wave" => draw_full_wave(ctx, width, height, pts),
        "pos_ladder" => draw_pos_ladder(ctx, width, height, pts),
        "neg_ladder" => draw_neg_ladder(ctx, width, height, pts),
        "noise" => draw_noise(ctx, width, height, pts),
        "exp_rise" => draw_exp_rise(ctx, width, height, pts),
        "exp_decay" => draw_exp_decay(ctx, width, height, pts),
        "multi_tone" => draw_multi_tone(ctx, width, height, pts),
        "sinc" => draw_sinc(ctx, width, height, pts),
        "lorenz" => draw_lorenz(ctx, width, height, pts),
        _ => draw_sine(ctx, width, height, pts),
    }
}

fn draw_grid(ctx: &Context, w: f64, h: f64) {
    ctx.set_source_rgb(0.08, 0.08, 0.12);
    ctx.set_line_width(0.5);

    let divs_x = 10.0;
    let divs_y = 4.0;
    let step_x = w / divs_x;
    let step_y = h / divs_y;

    // Líneas verticales
    for i in 0..=divs_x as i32 {
        let x = i as f64 * step_x;
        ctx.move_to(x, 0.0);
        ctx.line_to(x, h);
    }
    // Líneas horizontales
    for i in 0..=divs_y as i32 {
        let y = i as f64 * step_y;
        ctx.move_to(0.0, y);
        ctx.line_to(w, y);
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_sine(ctx: &Context, w: f64, h: f64, pts: i32) {
    let cy = h / 2.0;
    let amp = h / 2.0 - 6.0;
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let y = cy - (t * std::f64::consts::PI * 2.0).sin() * amp;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_square(ctx: &Context, w: f64, h: f64, duty: f64) {
    let d = duty.clamp(0.0, 1.0);
    let y_high = 6.0;
    let y_low = h - 6.0;
    ctx.move_to(0.0, y_high);
    ctx.line_to(w * d, y_high);
    ctx.line_to(w * d, y_low);
    ctx.line_to(w, y_low);
    ctx.stroke().unwrap_or_default();
}

fn draw_triangle(ctx: &Context, w: f64, h: f64, pts: i32) {
    let cy = h / 2.0;
    let amp = h / 2.0 - 6.0;
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let y_norm = if t < 0.25 {
            t * 4.0
        } else if t < 0.75 {
            1.0 - (t - 0.25) * 2.0
        } else {
            (t - 0.75) * 4.0 - 1.0
        };
        let y = cy - y_norm * amp;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_partial_sine(ctx: &Context, w: f64, h: f64, pts: i32) {
    let cy = h / 2.0;
    let amp = h / 2.0 - 6.0;
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let val = if t < 0.5 {
            (t * std::f64::consts::PI * 2.0).sin()
        } else {
            0.0
        };
        let y = cy - val * amp;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_cmos(ctx: &Context, w: f64, h: f64, pts: i32) {
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let y = if t < 0.45 {
            6.0
        } else if t < 0.55 {
            h / 2.0
        } else {
            h - 6.0
        };
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_dc(ctx: &Context, w: f64, h: f64) {
    let y = h / 2.0;
    ctx.move_to(0.0, y);
    ctx.line_to(w, y);
    ctx.stroke().unwrap_or_default();
}

fn draw_full_wave(ctx: &Context, w: f64, h: f64, pts: i32) {
    let cy = h / 2.0;
    let amp = h / 2.0 - 6.0;
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let val = (t * std::f64::consts::PI * 2.0).sin().abs();
        let y = cy - val * amp;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_pos_ladder(ctx: &Context, w: f64, h: f64, pts: i32) {
    let cy = h / 2.0;
    let amp = h / 2.0 - 6.0;
    let steps = 6.0;
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let step = (t * steps).floor() / steps;
        let y = cy - (step * 2.0 - 1.0) * amp;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_neg_ladder(ctx: &Context, w: f64, h: f64, pts: i32) {
    let cy = h / 2.0;
    let amp = h / 2.0 - 6.0;
    let steps = 6.0;
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let step = (t * steps).floor() / steps;
        let y = cy - (1.0 - step * 2.0) * amp;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_noise(ctx: &Context, w: f64, h: f64, pts: i32) {
    let cy = h / 2.0;
    let range = h - 12.0;
    let mut rng = rand::thread_rng();
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let y = cy + (rng.gen::<f64>() - 0.5) * range;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_exp_rise(ctx: &Context, w: f64, h: f64, pts: i32) {
    let cy = h / 2.0;
    let amp = h / 2.0 - 6.0;
    let e5 = std::f64::consts::E.powf(5.0);
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let val = ((std::f64::consts::E.powf(t * 5.0) - 1.0) / (e5 - 1.0)) * 2.0 - 1.0;
        let y = cy - val * amp;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_exp_decay(ctx: &Context, w: f64, h: f64, pts: i32) {
    let cy = h / 2.0;
    let amp = h / 2.0 - 6.0;
    let e5 = std::f64::consts::E.powf(5.0);
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let val = ((std::f64::consts::E.powf((1.0 - t) * 5.0) - 1.0) / (e5 - 1.0)) * 2.0 - 1.0;
        let y = cy - val * amp;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_multi_tone(ctx: &Context, w: f64, h: f64, pts: i32) {
    let cy = h / 2.0;
    let amp = (h / 2.0 - 6.0) * 0.7;
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let val = (t * std::f64::consts::PI * 2.0).sin()
            + 0.5 * (t * std::f64::consts::PI * 10.0).sin();
        let y = cy - val * amp;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_sinc(ctx: &Context, w: f64, h: f64, pts: i32) {
    let cy = h / 2.0;
    let amp = h / 2.0 - 6.0;
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let x = t * w;
        let t2 = (t - 0.5) * 20.0;
        let val = if t2.abs() < 1e-9 {
            1.0
        } else {
            t2.sin() / t2
        };
        let y = cy - val * amp;
        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }
    ctx.stroke().unwrap_or_default();
}

fn draw_lorenz(ctx: &Context, w: f64, h: f64, pts: i32) {
    let scale = 4.0;
    let dt = 0.02;
    let mut x = 0.1;
    let mut y = 0.0;
    let mut z = 0.0;
    for i in 0..=pts {
        let t = i as f64 / pts as f64;
        let px_coord = t * w;
        let dx = 10.0 * (y - x);
        let dy = x * (28.0 - z) - y;
        let dz = x * y - (8.0 / 3.0) * z;
        x += dx * dt;
        y += dy * dt;
        z += dz * dt;
        let py_coord = (h / 2.0 - y * scale).clamp(6.0, h - 6.0);
        if i == 0 {
            ctx.move_to(px_coord, py_coord);
        } else {
            ctx.line_to(px_coord, py_coord);
        }
    }
    ctx.stroke().unwrap_or_default();
}
