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
    const MASK: u8 = 0x3F;
    const PADDING: char = '=';
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
            let merged = window.iter().enumerate().fold(0, |merged, (i, byte)| {
                let ls = 8 * (window.len() - 1 - i);
                merged + (u32::from(*byte) << ls)
            });

            let chars = [18, 12, 6, 0]
                .into_iter()
                .map(|rs| (merged >> rs) & u32::from(Self::MASK))
                .filter(|byte| *byte > 0)
                .map(|byte| char::from(Self::ENGINE_TABLE[byte as usize]))
                .chain([Self::PADDING; 3])
                .take(4);

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
                    .unwrap_or_default();

                let ls = 6 * (window.len() - 1 - i);
                merged + (idx << ls)
            });

            for rs in [16, 8, 0] {
                let byte = u8::try_from((merged >> rs) & 0xFF).map_err(|err| err.to_string())?;
                decoded.push(byte)
            }
        }

        Ok(decoded)
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

    // [TODO] adjust the code to support this test
    // #[test]
    // fn standard_encode_works_with_padding() {
    //     let engine = Base64::standard();
    //     let encoded = engine.encode("Many hands make light work");
    //     assert_eq!(&encoded, "TWFueSBoYW5kcyBtYWtlIGxpZ2h0IHdvcms=");
    // }

    #[test]
    fn standard_decode_works() {
        let engine = Base64::standard();
        let input = "Many hands make light work.";
        let encoded = engine.encode(input);
        let decoded = engine
            .decode(encoded)
            .ok()
            .and_then(|decoded| String::from_utf8(decoded).ok())
            .expect("valid decoded string");

        assert_eq!(input, decoded);
    }

    #[test]
    fn works() {
        let num = 255u64;
        assert_eq!(num & 0x3F, 63);

        let string = [b'm', b'a', b'n'];
        let first = u32::from(string[0]) << 16;
        let second = u32::from(string[1]) << 8;
        let third = u32::from(string[2]);
        let merged = first + second + third;

        let first = merged >> 18 & 0x3F;
        let second = (merged >> 12) & 0x3F;
        let third = (merged >> 6) & 0x3F;
        let forth = merged & 0x3F;
        println!("{first} {second} {third} {forth}");

        let transformed = [first, second, third, forth]
            .into_iter()
            .map(|v| usize::try_from(v).expect("bro use a real computer"))
            .map(|i| char::from(StandardEngine::ENGINE_TABLE[i]))
            .collect::<String>();

        assert_eq!(transformed, "bWFu")
    }
}
