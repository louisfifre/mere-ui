//! Widget identity: a hash of the keys on the way down from the root, so the same widget has the same id
//! in every frame and two widgets under different parents never share one.

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub u64);

impl Id {
    pub const ROOT: Id = Id(0xcbf2_9ce4_8422_2325);

    /// The id of the child called `key` of this one.
    pub fn child(self, key: &str) -> Id {
        Id(fnv1a(self.0.to_le_bytes().into_iter().chain(key.bytes())))
    }
}

/// FNV-1a, 64 bits: a dozen lines, no dependency, and spread well enough for a few hundred widgets.
fn fnv1a(bytes: impl IntoIterator<Item = u8>) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_matches_the_published_vectors() {
        assert_eq!(fnv1a(*b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a(*b"a"), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn siblings_differ_and_paths_are_stable() {
        assert_ne!(Id::ROOT.child("a"), Id::ROOT.child("b"));
        assert_eq!(Id::ROOT.child("x").child("y"), Id::ROOT.child("x").child("y"));
        assert_ne!(Id::ROOT.child("x").child("y"), Id::ROOT.child("y"));
    }
}
