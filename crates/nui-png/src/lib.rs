//! Pictures of the interface, written as PNG. Lossless on purpose: a review looks at one-pixel hairlines and
//! small text, which a lossy format (JPEG, lossy WebP) blurs first.
//!
//! The format, in short: an 8-byte signature, then chunks (length, 4-letter type, data, CRC32 of type and
//! data). IHDR gives the size and pixel kind, IDAT the pixels, IEND closes. The pixels are rows, each
//! prefixed with a filter byte, compressed as one zlib stream (2-byte header, deflate data, Adler-32).
//!
//! Compression is deflate with the fixed Huffman codes of RFC 1951 §3.2.6 and a hash-chain LZ77 search, the
//! way zlib does it at its fast levels. Dynamic codes would shrink files by another 10-20 % for three times
//! the code; an interface picture is mostly flat colour, which the repeat matches already compress well.

mod deflate;
mod inflate;

pub use inflate::{Decoded, decode};

const SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Encodes `rgba` (4 bytes a pixel, rows top to bottom) as a PNG file.
pub fn encode(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    assert_eq!(rgba.len(), width as usize * height as usize * 4, "rgba is not width x height x 4");
    let mut out = SIGNATURE.to_vec();

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    // Bit depth 8, colour type 6 (RGBA), compression 0, filter method 0, no interlace.
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);

    let raw = filter(width as usize, height as usize, rgba);
    chunk(&mut out, b"IDAT", &zlib(&raw));
    chunk(&mut out, b"IEND", &[]);
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn zlib(raw: &[u8]) -> Vec<u8> {
    // CMF 0x78: deflate with a 32 KiB window. FLG 0x01: makes CMF*256+FLG a multiple of 31, as required.
    let mut out = vec![0x78, 0x01];
    out.extend_from_slice(&deflate::compress(raw));
    out.extend_from_slice(&adler32(raw).to_be_bytes());
    out
}

/// Each row gets the filter that leaves the smallest sum of bytes taken as signed, the heuristic the PNG
/// specification suggests: small values repeat more, and repeats compress.
fn filter(width: usize, height: usize, rgba: &[u8]) -> Vec<u8> {
    let stride = width * 4;
    let mut out = Vec::with_capacity((stride + 1) * height);
    let zero = vec![0u8; stride];
    let mut candidate = vec![0u8; stride];
    let mut best = vec![0u8; stride];
    for y in 0..height {
        let row = &rgba[y * stride..(y + 1) * stride];
        let up = if y > 0 { &rgba[(y - 1) * stride..y * stride] } else { &zero[..] };
        let mut best_kind = 0u8;
        let mut best_score = u64::MAX;
        for kind in 0..5u8 {
            for i in 0..stride {
                let a = if i >= 4 { row[i - 4] } else { 0 };
                let b = up[i];
                let c = if i >= 4 { up[i - 4] } else { 0 };
                let predicted = match kind {
                    0 => 0,
                    1 => a,
                    2 => b,
                    3 => ((a as u16 + b as u16) / 2) as u8,
                    _ => paeth(a, b, c),
                };
                candidate[i] = row[i].wrapping_sub(predicted);
            }
            let score: u64 = candidate.iter().map(|&v| (v as i8).unsigned_abs() as u64).sum();
            if score < best_score {
                best_score = score;
                best_kind = kind;
                best.copy_from_slice(&candidate);
            }
        }
        out.push(best_kind);
        out.extend_from_slice(&best);
    }
    out
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i16 + b as i16 - c as i16;
    let (pa, pb, pc) = ((p - a as i16).abs(), (p - b as i16).abs(), (p - c as i16).abs());
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

fn crc32(data: &[u8]) -> u32 {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let table = TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        for (n, slot) in t.iter_mut().enumerate() {
            let mut c = n as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
            *slot = c;
        }
        t
    });
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    // 5552 is the longest run before b can overflow 32 bits (zlib's NMAX), so the modulo waits that long.
    for block in data.chunks(5552) {
        for &x in block {
            a += x as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn picture(w: u32, h: u32) -> Vec<u8> {
        // A gradient, a flat field and a checker: the three things an interface picture is made of.
        let mut px = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let checker = (x / 8 + y / 8) % 2 == 0;
                let c = if y < h / 3 {
                    [(x * 255 / w) as u8, (y * 3) as u8, 40, 255]
                } else if y < 2 * h / 3 {
                    [16, 18, 20, 255]
                } else if checker {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 255, 128]
                };
                px.extend_from_slice(&c);
            }
        }
        px
    }

    #[test]
    fn what_is_written_reads_back_the_same() {
        for (w, h) in [(1, 1), (3, 7), (257, 64), (640, 360)] {
            let px = picture(w, h);
            let file = encode(w, h, &px);
            let back = decode(&file).expect("decodes");
            assert_eq!((back.width, back.height), (w, h));
            assert!(back.rgba == px, "pixels differ at {w}x{h}");
        }
    }

    #[test]
    fn an_interface_picture_compresses_well() {
        let (w, h) = (640, 360);
        let file = encode(w, h, &picture(w, h));
        let raw = (w * h * 4) as usize;
        assert!(file.len() * 8 < raw, "{} bytes for {} raw", file.len(), raw);
    }

    #[test]
    fn checksums_match_known_values() {
        assert_eq!(crc32(b"IEND"), 0xAE42_6082);
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }
}
