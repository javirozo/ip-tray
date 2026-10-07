#[path = "src/icon.rs"]
mod icon;

use std::{env, fs, path::PathBuf};

/// Arma un .ico (entradas BMP/BGRA) con el globo azul en varios tamaños.
fn build_ico(sizes: &[u32]) -> Vec<u8> {
    let mut images: Vec<Vec<u8>> = Vec::new();
    for &n in sizes {
        let rgba = icon::rgba(true, n);
        let mut img = Vec::new();
        // BITMAPINFOHEADER
        img.extend_from_slice(&40u32.to_le_bytes());
        img.extend_from_slice(&(n as i32).to_le_bytes());
        img.extend_from_slice(&((n * 2) as i32).to_le_bytes()); // alto doble (XOR + AND)
        img.extend_from_slice(&1u16.to_le_bytes());
        img.extend_from_slice(&32u16.to_le_bytes());
        img.extend_from_slice(&[0u8; 24]);
        // Píxeles BGRA, de abajo hacia arriba
        for y in (0..n).rev() {
            for x in 0..n {
                let i = ((y * n + x) * 4) as usize;
                img.extend_from_slice(&[rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
            }
        }
        // Máscara AND (todo ceros: el alfa manda)
        img.extend(std::iter::repeat(0u8).take((n * ((n + 31) / 32 * 4)) as usize));
        images.push(img);
    }

    let mut ico = Vec::new();
    ico.extend_from_slice(&[0, 0, 1, 0]);
    ico.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (&n, img) in sizes.iter().zip(&images) {
        ico.extend_from_slice(&[n as u8, n as u8, 0, 0]);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&(img.len() as u32).to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += img.len() as u32;
    }
    for img in images {
        ico.extend_from_slice(&img);
    }
    ico
}

fn main() {
    println!("cargo:rerun-if-changed=src/icon.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("ip-tray.ico");
    fs::write(&path, build_ico(&[16, 32, 48, 64])).unwrap();

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon(path.to_str().unwrap());
        res.set("FileDescription", "IP pública en la bandeja del sistema");
        res.compile().unwrap();
    }
}
