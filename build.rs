// Membuat ikon app (bendera Minesweeper + badge gear Rust) secara prosedural.
// Output: icon.ico (di-embed ke .exe) dan icon{16,32,64}.rgba (ikon jendela).
use std::{env, fs, path::PathBuf};

const BG: [f32; 3] = [30.0, 36.0, 51.0];
const RED: [f32; 3] = [229.0, 57.0, 53.0];
const WHITE: [f32; 3] = [236.0, 239.0, 244.0];
const RUST: [f32; 3] = [247.0, 76.0, 0.0];

fn in_rrect(x: f32, y: f32, r: f32) -> bool {
    let (cx, cy) = (x.clamp(r, 1.0 - r), y.clamp(r, 1.0 - r));
    (x - cx).powi(2) + (y - cy).powi(2) <= r * r
}

fn seg_dist(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (dx, dy) = (bx - ax, by - ay);
    let t = (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    ((px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2)).sqrt()
}

fn in_tri(px: f32, py: f32, a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> bool {
    let s = |p: (f32, f32), q: (f32, f32)| (px - q.0) * (p.1 - q.1) - (p.0 - q.0) * (py - q.1);
    let (d1, d2, d3) = (s(a, b), s(b, c), s(c, a));
    !((d1 < 0.0 || d2 < 0.0 || d3 < 0.0) && (d1 > 0.0 || d2 > 0.0 || d3 > 0.0))
}

fn in_gear(x: f32, y: f32, cx: f32, cy: f32) -> bool {
    let (dx, dy) = (x - cx, y - cy);
    let d = (dx * dx + dy * dy).sqrt();
    let a = dy.atan2(dx);
    let r = if (8.0 * a).cos() > 0.15 { 0.245 } else { 0.195 };
    d <= r
}

fn in_letter_r(x: f32, y: f32, cx: f32, cy: f32) -> bool {
    let (u, v) = (x - cx, y - cy);
    let stem = (-0.065..=-0.038).contains(&u) && (-0.078..=0.078).contains(&v);
    let (bu, bv) = (u + 0.008, v + 0.030);
    let bd = (bu * bu + bv * bv).sqrt();
    let bowl = u >= -0.045 && bd <= 0.049 && bd >= 0.024;
    let bar_top = (-0.065..=-0.008).contains(&u) && (-0.079..=-0.056).contains(&v);
    let bar_mid = (-0.065..=-0.008).contains(&u) && (-0.004..=0.019).contains(&v);
    let leg = seg_dist(u, v, -0.012, 0.012, 0.058, 0.078) <= 0.013;
    stem || bowl || bar_top || bar_mid || leg
}

/// Warna piksel pada koordinat ternormalisasi (0..1); alpha 0 di luar kotak membulat.
fn shade(x: f32, y: f32) -> Option<[f32; 3]> {
    if !in_rrect(x, y, 0.2) {
        return None;
    }
    let (gx, gy) = (0.73, 0.73);
    // Badge Rust (gear + huruf R), dengan lingkaran pemisah warna latar.
    if in_letter_r(x, y, gx, gy) {
        return Some(RUST);
    }
    if ((x - gx).powi(2) + (y - gy).powi(2)).sqrt() <= 0.135 {
        return Some(BG);
    }
    if in_gear(x, y, gx, gy) {
        return Some(RUST);
    }
    if ((x - gx).powi(2) + (y - gy).powi(2)).sqrt() <= 0.285 {
        return Some(BG);
    }
    // Bendera.
    if in_tri(x, y, (0.34, 0.13), (0.34, 0.45), (0.70, 0.29)) {
        return Some(RED);
    }
    if (0.30..=0.345).contains(&x) && (0.13..=0.78).contains(&y) {
        return Some(WHITE);
    }
    if (0.16..=0.50).contains(&x) && (0.78..=0.85).contains(&y) {
        return Some(WHITE);
    }
    Some(BG)
}

fn render(n: usize) -> Vec<u8> {
    const SS: usize = 4;
    let mut out = vec![0u8; n * n * 4];
    for py in 0..n {
        for px in 0..n {
            let (mut acc, mut cov) = ([0.0f32; 3], 0.0f32);
            for sy in 0..SS {
                for sx in 0..SS {
                    let x = (px as f32 + (sx as f32 + 0.5) / SS as f32) / n as f32;
                    let y = (py as f32 + (sy as f32 + 0.5) / SS as f32) / n as f32;
                    if let Some(c) = shade(x, y) {
                        for i in 0..3 {
                            acc[i] += c[i];
                        }
                        cov += 1.0;
                    }
                }
            }
            let o = (py * n + px) * 4;
            if cov > 0.0 {
                for i in 0..3 {
                    out[o + i] = (acc[i] / cov).round() as u8;
                }
                out[o + 3] = (cov / (SS * SS) as f32 * 255.0).round() as u8;
            }
        }
    }
    out
}

/// Encode satu gambar sebagai entri DIB (BMP tanpa header file) untuk ICO.
fn dib(n: usize, rgba: &[u8]) -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(&40u32.to_le_bytes());
    d.extend_from_slice(&(n as i32).to_le_bytes());
    d.extend_from_slice(&((n * 2) as i32).to_le_bytes());
    d.extend_from_slice(&1u16.to_le_bytes());
    d.extend_from_slice(&32u16.to_le_bytes());
    d.extend_from_slice(&[0u8; 24]);
    for y in (0..n).rev() {
        for x in 0..n {
            let o = (y * n + x) * 4;
            d.extend_from_slice(&[rgba[o + 2], rgba[o + 1], rgba[o], rgba[o + 3]]);
        }
    }
    let row = ((n + 31) / 32) * 4;
    d.extend(std::iter::repeat(0u8).take(row * n)); // AND mask (alpha dipakai)
    d
}

fn make_ico(sizes: &[usize]) -> Vec<u8> {
    let imgs: Vec<Vec<u8>> = sizes.iter().map(|&n| dib(n, &render(n))).collect();
    let mut ico = vec![0, 0, 1, 0];
    ico.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (i, &n) in sizes.iter().enumerate() {
        let b = if n >= 256 { 0 } else { n as u8 };
        ico.extend_from_slice(&[b, b, 0, 0, 1, 0, 32, 0]);
        ico.extend_from_slice(&(imgs[i].len() as u32).to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += imgs[i].len() as u32;
    }
    for img in imgs {
        ico.extend(img);
    }
    ico
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    for n in [16usize, 32, 64] {
        fs::write(out.join(format!("icon{n}.rgba")), render(n)).unwrap();
    }
    let ico_path = out.join("icon.ico");
    fs::write(&ico_path, make_ico(&[16, 32, 48, 256])).unwrap();

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon(ico_path.to_str().unwrap());
        res.compile().expect("gagal embed ikon");
    }
}
