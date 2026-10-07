//! Dibujo del globo terráqueo, compartido entre la app y build.rs (para el .ico del .exe).

/// RGBA de un globo de `n`x`n` px con antialiasing (4x4 subpíxeles).
/// Azul = online, rojo = sin conexión.
pub fn rgba(online: bool, n: u32) -> Vec<u8> {
    const SS: u32 = 4;
    let s = n as f32 / 32.0;
    let r_max = 15.0 * s;
    let line = (0.9 * s).max(0.6); // semigrosor de las líneas
    let fill: [f32; 3] = if online {
        [0x1E as f32, 0x88 as f32, 0xE5 as f32]
    } else {
        [0xD3 as f32, 0x2F as f32, 0x2F as f32]
    };
    let c = n as f32 / 2.0;

    let mut out = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let (mut circle, mut on_line) = (0u32, 0u32);
            for sy in 0..SS {
                for sx in 0..SS {
                    let px = x as f32 + (sx as f32 + 0.5) / SS as f32 - c;
                    let py = y as f32 + (sy as f32 + 0.5) / SS as f32 - c;
                    let r = (px * px + py * py).sqrt();
                    if r > r_max {
                        continue;
                    }
                    circle += 1;
                    let ring = r > r_max - 2.0 * line;
                    let equator = py.abs() < line;
                    let (a, b) = (r_max * 0.45, r_max);
                    let meridian =
                        (((px / a).powi(2) + (py / b).powi(2)).sqrt() - 1.0).abs() * a < line;
                    if ring || equator || meridian {
                        on_line += 1;
                    }
                }
            }
            if circle == 0 {
                out.extend_from_slice(&[0, 0, 0, 0]);
                continue;
            }
            let t = on_line as f32 / circle as f32;
            let mix = |f: f32| (f + (255.0 - f) * t) as u8;
            let alpha = (circle as f32 / (SS * SS) as f32 * 255.0) as u8;
            out.extend_from_slice(&[mix(fill[0]), mix(fill[1]), mix(fill[2]), alpha]);
        }
    }
    out
}
