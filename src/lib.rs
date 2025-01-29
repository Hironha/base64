#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Base64;

impl Base64 {
    const ENGINE_TABLE_STANDARD: [u8; 64] = [
        b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O',
        b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd',
        b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's',
        b't', b'u', b'v', b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
        b'8', b'9', b'+', b'/',
    ];

    const ENGINE_TABLE_URL_SAFE: [u8; 64] = [
        b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O',
        b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd',
        b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's',
        b't', b'u', b'v', b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
        b'8', b'9', b'-', b'_',
    ];

    pub const fn standard() -> Base64Engine {
        Base64Engine {
            table: &Self::ENGINE_TABLE_STANDARD,
            padding: '=',
        }
    }

    pub const fn url_safe() -> Base64Engine {
        Base64Engine {
            table: &Self::ENGINE_TABLE_URL_SAFE,
            padding: '=',
        }
    }
}

#[derive(Clone, Debug)]
pub struct Base64Engine {
    table: &'static [u8; 64],
    padding: char,
}

impl Base64Engine {
    // bitmask to get first 6 bits using &(and) operator
    const ENCODE_MASK: u32 = 0x3F;
    // bitmask to get first 8 bits using &(and) operator
    const DECODE_MASK: u32 = 0xFF;
    const ENCODE_RSH: [u8; 4] = [18, 12, 6, 0];
    const DECODE_RSH: [u8; 3] = [16, 8, 0];

    pub fn encode(&self, bytes: impl AsRef<[u8]>) -> String {
        let bytes = bytes.as_ref();
        let mut encoded = String::with_capacity(bytes.len() * 4 / 3);
        for window in bytes.chunks_exact(3) {
            let merged = match window {
                [first, second, third] => self.merge_encode_bytes(*first, *second, *third),
                [first, second] => self.merge_encode_bytes(*first, *second, 0),
                [first] => self.merge_encode_bytes(*first, 0, 0),
                _ => self.merge_encode_bytes(0, 0, 0),
            };
            let chars = Self::ENCODE_RSH
                .into_iter()
                .map(|rsh| (merged >> rsh) & Self::ENCODE_MASK)
                .filter(|byte| *byte > 0)
                .map(|byte| char::from(self.table[byte as usize]));

            encoded.extend(chars);
        }

        let remaining_bytes = bytes.len() % 3;
        if remaining_bytes != 0 {
            let merged = match &bytes[bytes.len() - remaining_bytes..] {
                [first, second, third] => self.merge_encode_bytes(*first, *second, *third),
                [first, second] => self.merge_encode_bytes(*first, *second, 0),
                [first] => self.merge_encode_bytes(*first, 0, 0),
                _ => self.merge_encode_bytes(0, 0, 0),
            };
            let chars = Self::ENCODE_RSH
                .into_iter()
                .map(|rsh| (merged >> rsh) & Self::ENCODE_MASK)
                .map(|byte| match byte {
                    0 => self.padding,
                    byte => char::from(self.table[byte as usize]),
                });

            encoded.extend(chars);
        }

        encoded
    }

    pub fn decode(&self, encoded: impl AsRef<[u8]>) -> Result<Vec<u8>, String> {
        let bytes = encoded.as_ref();
        let mut decoded = Vec::<u8>::with_capacity(bytes.len() * 3 / 4);
        for window in bytes.chunks_exact(4) {
            let merged = window.iter().enumerate().fold(0, |merged, (i, byte)| {
                let idx = self
                    .table
                    .iter()
                    .position(|b| b == byte)
                    .and_then(|idx| u32::try_from(idx).ok())
                    .unwrap_or_default();

                let lsh = 6 * (3 - i);
                merged + (idx << lsh)
            });

            for byte in Self::DECODE_RSH
                .into_iter()
                // guaranteed to fit in u8 since we masked with `DECODE_MASK`
                .map(|rsh| ((merged >> rsh) & Self::DECODE_MASK) as u8)
                .take_while(|byte| *byte > 0)
            {
                decoded.push(byte)
            }
        }

        Ok(decoded)
    }

    #[inline(always)]
    fn merge_encode_bytes(&self, first: u8, second: u8, third: u8) -> u32 {
        (u32::from(first) << 16) + (u32::from(second) << 8) + u32::from(third)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_encode_works() {
        let engine = Base64::standard();
        let encoded = engine.encode("Many hands make light work.");
        assert_eq!(&encoded, "TWFueSBoYW5kcyBtYWtlIGxpZ2h0IHdvcmsu");
    }

    #[test]
    fn standard_encode_works_with_padding() {
        let engine = Base64::standard();
        let encoded = engine.encode("Many hands make light work");
        assert_eq!(&encoded, "TWFueSBoYW5kcyBtYWtlIGxpZ2h0IHdvcms=");
    }

    #[test]
    fn standard_decode_works() {
        let engine = Base64::standard();
        let input = "Many hands make light work.";
        let encoded = engine.encode(input);
        let decoded = engine.decode(encoded).expect("should be able to decode");
        let decoded_string = String::from_utf8(decoded).expect("should be a valid string");

        assert_eq!(decoded_string, input);
    }

    #[test]
    fn standard_decode_works_with_padding() {
        let engine = Base64::standard();
        let inputs = ["Many hands make light wor", "Many hands make light work"];
        for input in inputs {
            let encoded = engine.encode(input);
            let decoded = engine.decode(encoded).expect("should be able to decode");
            let decoded_string = String::from_utf8(decoded).expect("should be a valid string");

            assert_eq!(decoded_string, input);
        }
    }
}
