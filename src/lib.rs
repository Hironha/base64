#[derive(Clone, Debug)]
pub struct Base64;

impl Base64 {
    pub fn standard() -> StandardEngine {
        StandardEngine
    }

    pub fn url_safe() -> UrlSafeEngine {
        UrlSafeEngine
    }
}

#[derive(Clone, Debug)]
pub struct StandardEngine;

impl StandardEngine {
    // bitmask to get first 6 bits using &(and) operator
    const ENCODE_MASK: u32 = 0x3F;
    // bitmask to get first 8 bits using &(and) operator
    const DECODE_MASK: u32 = 0xFF;
    const PADDING: char = '=';
    const ENCODE_RSH: [u8; 4] = [18, 12, 6, 0];
    const DECODE_RSH: [u8; 3] = [16, 8, 0];
    const ENGINE_TABLE: [u8; 64] = [
        b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O',
        b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd',
        b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's',
        b't', b'u', b'v', b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
        b'8', b'9', b'+', b'/',
    ];

    pub fn encode(&self, bytes: impl AsRef<[u8]>) -> String {
        let bytes = bytes.as_ref();
        let mut encoded = String::with_capacity(bytes.len() * 4 / 3);
        for window in bytes.windows(3).step_by(3) {
            let mut window = window.iter();
            let first = window.next().copied().unwrap_or_default();
            let second = window.next().copied().unwrap_or_default();
            let third = window.next().copied().unwrap_or_default();
            let merged = self.merge_encode_bytes(first, second, third);
            let chars = Self::ENCODE_RSH
                .into_iter()
                .map(|rsh| (merged >> rsh) & Self::ENCODE_MASK)
                .filter(|byte| *byte > 0)
                .map(|byte| char::from(Self::ENGINE_TABLE[byte as usize]));

            encoded.extend(chars);
        }

        let remaining_bytes = bytes.len() % 3;
        if remaining_bytes != 0 {
            let mut window = bytes[bytes.len() - remaining_bytes..].iter();
            let first = window.next().copied().unwrap_or_default();
            let second = window.next().copied().unwrap_or_default();
            let third = window.next().copied().unwrap_or_default();
            let merged = self.merge_encode_bytes(first, second, third);
            let chars = Self::ENCODE_RSH
                .into_iter()
                .map(|rsh| (merged >> rsh) & Self::ENCODE_MASK)
                .map(|byte| match byte {
                    0 => Self::PADDING,
                    byte => char::from(Self::ENGINE_TABLE[byte as usize]),
                });

            encoded.extend(chars);
        }

        encoded
    }

    pub fn decode(&self, encoded: impl AsRef<[u8]>) -> Result<Vec<u8>, String> {
        let bytes = encoded.as_ref();
        let mut decoded = Vec::<u8>::with_capacity(bytes.len() * 3 / 4);
        for window in bytes.windows(4).step_by(4) {
            let merged = window.iter().enumerate().fold(0, |merged, (i, byte)| {
                let idx = Self::ENGINE_TABLE
                    .iter()
                    .position(|b| b == byte)
                    .and_then(|idx| u32::try_from(idx).ok())
                    .unwrap_or_default();

                let lsh = 6 * (3 - i);
                merged + (idx << lsh)
            });

            for byte in Self::DECODE_RSH
                .into_iter()
                .map(|rsh| (merged >> rsh) & Self::DECODE_MASK)
                .take_while(|byte| *byte > 0)
            {
                let byte = u8::try_from(byte).map_err(|err| err.to_string())?;
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

#[derive(Clone, Debug)]
pub struct UrlSafeEngine;

impl UrlSafeEngine {
    pub fn encode(&self, bytes: impl AsRef<[u8]>) -> String {
        let bytes = bytes.as_ref();
        println!("{bytes:?}");
        todo!()
    }

    pub fn decode(&self, encoded: impl AsRef<[u8]>) -> Vec<u8> {
        let bytes = encoded.as_ref();
        println!("{bytes:?}");
        todo!()
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
