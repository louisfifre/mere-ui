//! Reads back the PNG files `encode` writes: RGBA, 8 bits, stored or fixed-Huffman deflate blocks. It is
//! there to compare pictures in tests, not to open any PNG; a file it cannot read says so.

use crate::deflate::{DIST_BASE, DIST_EXTRA, LENGTH_BASE, LENGTH_EXTRA};

pub struct Decoded {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn decode(file: &[u8]) -> Result<Decoded, String> {
    if file.get(..8) != Some(&crate::SIGNATURE[..]) {
        return Err("not a PNG file".into());
    }
    let (mut pos, mut width, mut height, mut idat) = (8, 0, 0, Vec::new());
    while pos + 8 <= file.len() {
        let len = u32::from_be_bytes(file[pos..pos + 4].try_into().unwrap()) as usize;
        let kind = &file[pos + 4..pos + 8];
        let data = file.get(pos + 8..pos + 8 + len).ok_or("chunk runs past the end")?;
        match kind {
            b"IHDR" => {
                width = u32::from_be_bytes(data[0..4].try_into().unwrap());
                height = u32::from_be_bytes(data[4..8].try_into().unwrap());
                if data[8..13] != [8, 6, 0, 0, 0] {
                    return Err("only 8-bit RGBA without interlace is read".into());
                }
            }
            b"IDAT" => idat.extend_from_slice(data),
            b"IEND" => break,
            _ => {}
        }
        pos += 12 + len;
    }
    let raw = inflate(idat.get(2..).ok_or("empty IDAT")?)?;
    let stride = width as usize * 4;
    if raw.len() != (stride + 1) * height as usize {
        return Err(format!("{} bytes of rows, {} expected", raw.len(), (stride + 1) * height as usize));
    }
    let mut rgba = vec![0u8; stride * height as usize];
    for y in 0..height as usize {
        let kind = raw[y * (stride + 1)];
        let row = &raw[y * (stride + 1) + 1..(y + 1) * (stride + 1)];
        for i in 0..stride {
            let a = if i >= 4 { rgba[y * stride + i - 4] } else { 0 };
            let b = if y > 0 { rgba[(y - 1) * stride + i] } else { 0 };
            let c = if y > 0 && i >= 4 { rgba[(y - 1) * stride + i - 4] } else { 0 };
            let predicted = match kind {
                0 => 0,
                1 => a,
                2 => b,
                3 => ((a as u16 + b as u16) / 2) as u8,
                4 => crate::paeth(a, b, c),
                _ => return Err(format!("unknown filter {kind}")),
            };
            rgba[y * stride + i] = row[i].wrapping_add(predicted);
        }
    }
    Ok(Decoded { width, height, rgba })
}

fn inflate(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut r = Reader { data, pos: 0, bit: 0 };
    let mut out = Vec::new();
    loop {
        let last = r.bits(1)?;
        match r.bits(2)? {
            0 => {
                r.align();
                let len = r.bits(16)? as usize;
                r.bits(16)?;
                for _ in 0..len {
                    out.push(r.bits(8)? as u8);
                }
            }
            1 => loop {
                let symbol = r.fixed_symbol()?;
                if symbol < 256 {
                    out.push(symbol as u8);
                } else if symbol == 256 {
                    break;
                } else {
                    let k = (symbol - 257) as usize;
                    let len = LENGTH_BASE[k] as usize + r.bits(LENGTH_EXTRA[k])? as usize;
                    let d = r.huffman(5)? as usize;
                    let dist = DIST_BASE[d] as usize + r.bits(DIST_EXTRA[d])? as usize;
                    let start = out.len().checked_sub(dist).ok_or("distance before the start")?;
                    for k in 0..len {
                        out.push(out[start + k]);
                    }
                }
            },
            _ => return Err("dynamic Huffman blocks are not read".into()),
        }
        if last == 1 {
            return Ok(out);
        }
    }
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
    bit: u32,
}

impl Reader<'_> {
    fn bits(&mut self, n: u8) -> Result<u32, String> {
        let mut v = 0;
        for k in 0..n {
            let byte = *self.data.get(self.pos).ok_or("deflate data ends early")?;
            v |= ((byte >> self.bit) as u32 & 1) << k;
            self.bit += 1;
            if self.bit == 8 {
                self.bit = 0;
                self.pos += 1;
            }
        }
        Ok(v)
    }
    fn huffman(&mut self, n: u8) -> Result<u32, String> {
        let mut v = 0;
        for _ in 0..n {
            v = (v << 1) | self.bits(1)?;
        }
        Ok(v)
    }
    fn align(&mut self) {
        if self.bit != 0 {
            self.bit = 0;
            self.pos += 1;
        }
    }
    fn fixed_symbol(&mut self) -> Result<u16, String> {
        let mut code = self.huffman(7)?;
        if code <= 0x17 {
            return Ok(256 + code as u16);
        }
        code = (code << 1) | self.bits(1)?;
        if (0x30..=0xBF).contains(&code) {
            return Ok((code - 0x30) as u16);
        }
        if (0xC0..=0xC7).contains(&code) {
            return Ok((280 + code - 0xC0) as u16);
        }
        code = (code << 1) | self.bits(1)?;
        Ok((144 + code - 0x190) as u16)
    }
}
