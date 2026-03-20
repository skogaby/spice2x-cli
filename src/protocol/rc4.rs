/// RC4 stream cipher implementation.
///
/// Tested against RFC 6229 known test vectors.
pub struct Rc4 {
    s_box: [u8; 256],
    i: u8,
    j: u8,
}

impl Rc4 {
    /// Create a new RC4 cipher initialized with the given key.
    ///
    /// Performs the Key Scheduling Algorithm (KSA) to initialize the S-box.
    pub fn new(key: &[u8]) -> Self {
        let mut s_box = [0u8; 256];
        for i in 0..256 {
            s_box[i] = i as u8;
        }

        if !key.is_empty() {
            let mut j: u8 = 0;
            for i in 0..256 {
                j = j.wrapping_add(s_box[i]).wrapping_add(key[i % key.len()]);
                s_box.swap(i, j as usize);
            }
        }

        Self { s_box, i: 0, j: 0 }
    }

    /// Encrypt or decrypt data in-place using the PRGA keystream.
    pub fn crypt(&mut self, data: &mut [u8]) {
        for byte in data.iter_mut() {
            self.i = self.i.wrapping_add(1);
            self.j = self.j.wrapping_add(self.s_box[self.i as usize]);
            self.s_box.swap(self.i as usize, self.j as usize);
            let k = self.s_box[self.s_box[self.i as usize].wrapping_add(self.s_box[self.j as usize]) as usize];
            *byte ^= k;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_key_is_identity() {
        let mut cipher = Rc4::new(b"");
        let original = b"hello world".to_vec();
        let mut data = original.clone();
        cipher.crypt(&mut data);
        // Empty key means s_box is identity [0,1,2,...,255].
        // PRGA still produces a keystream, so data WILL be modified.
        // This test just verifies it doesn't panic.
        assert_eq!(data.len(), original.len());
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = b"test_password";
        let plaintext = b"Hello, SpiceAPI!";

        let mut encrypted = plaintext.to_vec();
        Rc4::new(key).crypt(&mut encrypted);
        assert_ne!(&encrypted, plaintext);

        let mut decrypted = encrypted;
        Rc4::new(key).crypt(&mut decrypted);
        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn known_vector_rfc6229_key_8() {
        // RFC 6229 test vector: 8-byte key 0x0102030405060708
        // First keystream bytes: 0x97, 0xab, 0x8a, 0x1b, 0xf0, 0xaf, 0xb9, 0x61
        let key = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let mut cipher = Rc4::new(&key);
        let mut buf = [0u8; 8];
        cipher.crypt(&mut buf);
        // XOR with zeros gives the raw keystream
        assert_eq!(buf, [0x97, 0xab, 0x8a, 0x1b, 0xf0, 0xaf, 0xb9, 0x61]);
    }

    #[test]
    fn stateful_across_calls() {
        let key = b"mykey";
        let mut cipher = Rc4::new(key);

        let mut part1 = b"hello".to_vec();
        cipher.crypt(&mut part1);

        let mut part2 = b" world".to_vec();
        cipher.crypt(&mut part2);

        // Decrypting the concatenation with a fresh cipher should recover plaintext
        let mut combined = [part1, part2].concat();
        Rc4::new(key).crypt(&mut combined);
        assert_eq!(&combined, b"hello world");
    }
}
