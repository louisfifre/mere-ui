//! Deflate (RFC 1951), one block with the fixed Huffman codes. LZ77 finds repeats through hash chains: the
//! three bytes at each position are hashed, and `head`/`prev` link every earlier position with the same
//! hash, newest first, so a search walks back only through candidates that can match.

const WINDOW: usize = 32 * 1024;
const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 258;
// How many candidates a search walks before taking the best so far: zlib's level 6 uses 128.
const CHAIN: usize = 64;
const HASH_BITS: u32 = 15;

pub fn compress(data: &[u8]) -> Vec<u8> {
    let mut bits = Bits::default();
    // BFINAL = 1 (last block), BTYPE = 01 (fixed codes).
    bits.put(1, 1);
    bits.put(1, 2);

    let mut head = vec![u32::MAX; 1 << HASH_BITS];
    let mut prev = vec![u32::MAX; WINDOW];
    let hash = |i: usize| {
        let v = (data[i] as u32) | (data[i + 1] as u32) << 8 | (data[i + 2] as u32) << 16;
        (v.wrapping_mul(2_654_435_761) >> (32 - HASH_BITS)) as usize
    };
    let insert =|i: usize, head: &mut Vec<u32>, prev: &mut Vec<u32>| {
        if i + MIN_MATCH <= data.len() {
            let h = hash(i);
            prev[i % WINDOW] = head[h];
            head[h] = i as u32;
        }
    };

    let mut i = 0;
    while i < data.len() {
        let (mut best_len, mut best_dist) = (0, 0);
        if i + MIN_MATCH <= data.len() {
            let limit = (data.len() - i).min(MAX_MATCH);
            let mut candidate = head[hash(i)];
            let mut walked = 0;
            while candidate != u32::MAX && walked < CHAIN {
                let c = candidate as usize;
                if i - c > WINDOW - 1 {
                    break;
                }
                let len = data[c..].iter().zip(&data[i..i + limit]).take_while(|(a, b)| a == b).count();
                if len > best_len {
                    best_len = len;
                    best_dist = i - c;
                    if len == limit {
                        break;
                    }
                }
                let next = prev[c % WINDOW];
                // A link older than the window, or one the ring has overwritten, ends the chain.
                if next == u32::MAX || next as usize >= c {
                    break;
                }
                candidate = next;
                walked += 1;
            }
        }
        if best_len >= MIN_MATCH {
            put_length(&mut bits, best_len);
            put_distance(&mut bits, best_dist);
            for k in i..i + best_len {
                insert(k, &mut head, &mut prev);
            }
            i += best_len;
        } else {
            put_literal(&mut bits, data[i] as u16);
            insert(i, &mut head, &mut prev);
            i += 1;
        }
    }
    put_literal(&mut bits, 256); // end of block
    bits.finish()
}

// RFC 1951 §3.2.5: the first length or distance each code stands for, and how many extra bits follow it.
pub(crate) const LENGTH_BASE: [u16; 29] =
    [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
pub(crate) const LENGTH_EXTRA: [u8; 29] =
    [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
pub(crate) const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097,
    6145, 8193, 12289, 16385, 24577,
];
pub(crate) const DIST_EXTRA: [u8; 30] =
    [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];

/// The fixed literal/length code of a symbol 0..=287, as (code, bit count).
fn fixed_code(symbol: u16) -> (u16, u8) {
    match symbol {
        0..=143 => (0x30 + symbol, 8),
        144..=255 => (0x190 + symbol - 144, 9),
        256..=279 => (symbol - 256, 7),
        _ => (0xC0 + symbol - 280, 8),
    }
}

fn put_literal(bits: &mut Bits, symbol: u16) {
    let (code, n) = fixed_code(symbol);
    bits.put_huffman(code as u32, n);
}

fn put_length(bits: &mut Bits, len: usize) {
    let k = LENGTH_BASE.iter().rposition(|&b| b as usize <= len).unwrap();
    put_literal(bits, 257 + k as u16);
    bits.put((len - LENGTH_BASE[k] as usize) as u32, LENGTH_EXTRA[k]);
}

fn put_distance(bits: &mut Bits, dist: usize) {
    let k = DIST_BASE.iter().rposition(|&b| b as usize <= dist).unwrap();
    bits.put_huffman(k as u32, 5);
    bits.put((dist - DIST_BASE[k] as usize) as u32, DIST_EXTRA[k]);
}

/// Deflate packs bits from the least significant end of each byte; Huffman codes alone go in from their
/// most significant bit, hence the two ways in.
#[derive(Default)]
struct Bits {
    out: Vec<u8>,
    acc: u64,
    count: u32,
}

impl Bits {
    fn put(&mut self, value: u32, n: u8) {
        self.acc |= (value as u64) << self.count;
        self.count += n as u32;
        while self.count >= 8 {
            self.out.push(self.acc as u8);
            self.acc >>= 8;
            self.count -= 8;
        }
    }
    fn put_huffman(&mut self, code: u32, n: u8) {
        let reversed = code.reverse_bits() >> (32 - n as u32);
        self.put(reversed, n);
    }
    fn finish(mut self) -> Vec<u8> {
        if self.count > 0 {
            self.out.push(self.acc as u8);
        }
        self.out
    }
}
