const ALPHABET_STANDARD: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/',
];

const ALPHABET_URL_SAFE: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'-', b'_',
];

// bitmask to get first 6 bits using &(and) operator
const ENCODE_MASK: u32 = 0x3F;
// bitmask to get first 8 bits using &(and) operator
const DECODE_MASK: u32 = 0xFF;
// encode right shifts amount to split one u32 into 4 u8
const ENCODE_RSH: [u8; 4] = [18, 12, 6, 0];
// decode right shifts amount to split one u32 into 3 u8
const DECODE_RSH: [u8; 3] = [16, 8, 0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Base64 {
    alphabet: &'static [u8; 64],
    padding: char,
}

impl Base64 {
    pub const fn standard() -> Self {
        Self {
            alphabet: &ALPHABET_STANDARD,
            padding: '=',
        }
    }

    pub const fn url_safe() -> Self {
        Self {
            alphabet: &ALPHABET_URL_SAFE,
            padding: '=',
        }
    }

    pub fn encode(&self, bytes: impl AsRef<[u8]>) -> String {
        let bytes = bytes.as_ref();
        let mut encoded = String::with_capacity(bytes.len() * 4 / 3);

        for window in bytes.chunks_exact(3) {
            // merge 3 bytes in 4 base64 bytes (6 bits each)
            let merged = match window {
                [first, second, third] => self.make_u32_from_parts(*first, *second, *third),
                // `chunks_exact` guarantees a window of length 3
                w => panic!("Unexpected encode window with len {}", w.len()),
            };

            // transform 4 merged base64 bytes (u32) into characters from alphabet
            let chars = ENCODE_RSH
                .iter()
                .map(|rsh| (merged >> rsh) & ENCODE_MASK)
                // .filter(|byte| *byte > 0)
                .map(|byte| char::from(self.alphabet[byte as usize]));

            encoded.extend(chars);
        }

        let remaining_bytes = bytes.len() % 3;
        if remaining_bytes > 0 {
            let remaining_start_at = bytes.len() - remaining_bytes;
            let merged = match &bytes[remaining_start_at..] {
                [first, second] => self.make_u32_from_parts(*first, *second, 0),
                [first] => self.make_u32_from_parts(*first, 0, 0),
                _ => 0u32,
            };

            let chars = ENCODE_RSH.iter().enumerate().map(|(i, rsh)| {
                if i <= remaining_bytes {
                    let sextet = ((merged >> rsh) & ENCODE_MASK) as usize;
                    self.alphabet[sextet] as char
                } else {
                    self.padding
                }
            });

            encoded.extend(chars);
        }

        encoded
    }

    pub fn decode(&self, encoded: impl AsRef<[u8]>) -> Result<Vec<u8>, String> {
        let bytes = encoded.as_ref();
        if bytes.is_empty() {
            return Ok(Vec::from(b""));
        }

        let (unpadded, padded) = bytes.split_at(bytes.len() - 4);
        if unpadded.len() % 4 != 0 || padded.len() % 4 != 0 {
            let msg = "Invalid bytes of standard base64 with padding";
            return Err(String::from(msg));
        }

        let mut decoded = Vec::<u8>::with_capacity(bytes.len() * 3 / 4);
        for window in unpadded.chunks_exact(4) {
            // merge of 4 base64 characters of 6 bits each
            let merged = self.decode_u32_from_base64_block(window)?;
            for rsh in DECODE_RSH.iter() {
                decoded.push(((merged >> rsh) & DECODE_MASK) as u8);
            }
        }

        // merge of 4 base64 characters of 6 bits each
        let merged = self.decode_u32_from_base64_block(padded)?;
        let pad = padded.iter().rev().take_while(|&&b| b == b'=').count();
        let nbytes = match pad {
            0 => 3,
            1 => 2,
            2 => 1,
            _ => return Err("Invalid padding in base64 block".into()),
        };
        // transform from merged 24 bits into output bytes
        for rsh in DECODE_RSH.iter().take(nbytes) {
            decoded.push(((merged >> rsh) & DECODE_MASK) as u8);
        }

        Ok(decoded)
    }

    #[inline(always)]
    fn make_u32_from_parts(&self, first: u8, second: u8, third: u8) -> u32 {
        (u32::from(first) << 16) + (u32::from(second) << 8) + u32::from(third)
    }

    // block expected to have exactly 4 bytes
    fn decode_u32_from_base64_block(&self, block: &[u8]) -> Result<u32, String> {
        block.iter().enumerate().try_fold(0, |merged, (i, byte)| {
            let idx = match self.alphabet.iter().position(|b| b == byte) {
                Some(idx) => u32::try_from(idx).ok().unwrap_or_default(),
                None if *byte == b'=' => 0u32,
                None => return Err(format!("Invalid base64 byte: {byte:0x}")),
            };

            let lsh = 6 * (3 - i);
            Ok(merged + (idx << lsh))
        })
    }
}

#[cfg(test)]
mod tests {
    mod standard {
        use crate::Base64;

        #[test]
        fn test_basic_ascii() {
            let config = [
                ("", ""),
                ("f", "Zg=="),
                ("fo", "Zm8="),
                ("foo", "Zm9v"),
                ("foobar", "Zm9vYmFy"),
                ("light w", "bGlnaHQgdw=="),
                ("light wo", "bGlnaHQgd28="),
                ("light wor", "bGlnaHQgd29y"),
            ];

            let engine = Base64::standard();
            for (raw, encoded) in config {
                let out = engine.encode(raw);
                assert_eq!(encoded, out);

                let out = engine.decode(out).unwrap();
                assert_eq!(raw.as_bytes(), out);
            }
        }

        #[test]
        fn test_padding_edges() {
            let config = [
                (Vec::from([0x01]), "AQ=="),
                (Vec::from([0x01, 0x02]), "AQI="),
                (Vec::from([0x01, 0x02, 0x03]), "AQID"),
                (Vec::from([0x01, 0x02, 0x03, 0x04]), "AQIDBA=="),
                (Vec::from([0x01, 0x02, 0x03, 0x04, 0x05]), "AQIDBAU="),
            ];

            let engine = Base64::standard();
            for (raw, encoded) in config {
                let out = engine.encode(&raw);
                assert_eq!(out, encoded);

                let out = engine.decode(out).unwrap();
                assert_eq!(raw, out);
            }
        }

        #[test]
        fn test_ascii_cases() {
            let config = [
                ("hello", "aGVsbG8="),
                ("Hello, world!", "SGVsbG8sIHdvcmxkIQ=="),
                ("Base64 Encoding Test", "QmFzZTY0IEVuY29kaW5nIFRlc3Q="),
                (
                    "abcdefghijklmnopqrstuvwxyz",
                    "YWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXo=",
                ),
                (
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                    "QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVo=",
                ),
                ("0123456789", "MDEyMzQ1Njc4OQ=="),
            ];

            let engine = Base64::standard();
            for (raw, encoded) in config {
                let out = engine.encode(raw);
                assert_eq!(encoded, out);

                let out = engine.decode(out).unwrap();
                assert_eq!(raw.as_bytes(), out);
            }
        }

        #[test]
        fn test_binary_data() {
            let config = [(Vec::from([0, 1, 2]), "AAEC")];

            let engine = Base64::standard();
            for (raw, encoded) in config {
                let out = engine.encode(&raw);
                assert_eq!(encoded, out);

                let out = engine.decode(out).unwrap();
                assert_eq!(raw, out);
            }
        }

        #[test]
        fn test_utf8() {
            let config = [
                ("✓", "4pyT"),
                ("こんにちは", "44GT44KT44Gr44Gh44Gv"),
                ("😊", "8J+Yig=="),
                ("€", "4oKs"),
            ];

            let engine = Base64::standard();
            for (raw, encoded) in config {
                let out = engine.encode(raw);
                assert_eq!(encoded, out);

                let out = engine.decode(out).unwrap();
                assert_eq!(raw.as_bytes(), out);
            }
        }

        #[test]
        fn test_all_bytes() {
            let all: Vec<u8> = (0u8..=255).collect();
            let encoded = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7PD0+P0BBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWltcXV5fYGFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6e3x9fn+AgYKDhIWGh4iJiouMjY6PkJGSk5SVlpeYmZqbnJ2en6ChoqOkpaanqKmqq6ytrq+wsbKztLW2t7i5uru8vb6/wMHCw8TFxsfIycrLzM3Oz9DR0tPU1dbX2Nna29zd3t/g4eLj5OXm5+jp6uvs7e7v8PHy8/T19vf4+fr7/P3+/w==";

            let engine = Base64::standard();
            let out = engine.encode(&all);
            assert_eq!(encoded, out);

            let out = engine.decode(out).unwrap();
            assert_eq!(all, out);
        }
    }

    mod url_safe {
        use crate::Base64;

        #[test]
        fn test_basic_ascii() {
            let config = [
                ("", ""),
                ("f", "Zg=="),
                ("fo", "Zm8="),
                ("foo", "Zm9v"),
                ("foobar", "Zm9vYmFy"),
                ("light w", "bGlnaHQgdw=="),
                ("light wo", "bGlnaHQgd28="),
                ("light wor", "bGlnaHQgd29y"),
            ];

            let engine = Base64::url_safe();
            for (raw, encoded) in config {
                let out = engine.encode(raw);
                assert_eq!(encoded, out);

                let out = engine.decode(out).unwrap();
                assert_eq!(raw.as_bytes(), out);
            }
        }

        #[test]
        fn test_padding_edges() {
            let config = [
                (Vec::from([0x01]), "AQ=="),
                (Vec::from([0x01, 0x02]), "AQI="),
                (Vec::from([0x01, 0x02, 0x03]), "AQID"),
                (Vec::from([0x01, 0x02, 0x03, 0x04]), "AQIDBA=="),
                (Vec::from([0x01, 0x02, 0x03, 0x04, 0x05]), "AQIDBAU="),
            ];

            let engine = Base64::url_safe();
            for (raw, encoded) in config {
                let out = engine.encode(&raw);
                assert_eq!(out, encoded);

                let out = engine.decode(out).unwrap();
                assert_eq!(raw, out);
            }
        }

        #[test]
        fn test_ascii_cases() {
            let config = [
                ("hello", "aGVsbG8="),
                ("Hello, world!", "SGVsbG8sIHdvcmxkIQ=="),
                ("Base64 Encoding Test", "QmFzZTY0IEVuY29kaW5nIFRlc3Q="),
                (
                    "abcdefghijklmnopqrstuvwxyz",
                    "YWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXo=",
                ),
                (
                    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                    "QUJDREVGR0hJSktMTU5PUFFSU1RVVldYWVo=",
                ),
                ("0123456789", "MDEyMzQ1Njc4OQ=="),
            ];

            let engine = Base64::url_safe();
            for (raw, encoded) in config {
                let out = engine.encode(raw);
                assert_eq!(encoded, out);

                let out = engine.decode(out).unwrap();
                assert_eq!(raw.as_bytes(), out);
            }
        }

        #[test]
        fn test_binary_data() {
            let config = [(Vec::from([0, 1, 2]), "AAEC")];

            let engine = Base64::url_safe();
            for (raw, encoded) in config {
                let out = engine.encode(&raw);
                assert_eq!(encoded, out);

                let out = engine.decode(out).unwrap();
                assert_eq!(raw, out);
            }
        }

        #[test]
        fn test_utf8() {
            let config = [
                ("✓", "4pyT"),
                ("こんにちは", "44GT44KT44Gr44Gh44Gv"),
                ("😊", "8J-Yig=="),
                ("€", "4oKs"),
            ];

            let engine = Base64::url_safe();
            for (raw, encoded) in config {
                let out = engine.encode(raw);
                assert_eq!(encoded, out);

                let out = engine.decode(out).unwrap();
                assert_eq!(raw.as_bytes(), out);
            }
        }

        #[test]
        fn test_all_bytes() {
            let all: Vec<u8> = (0u8..=255).collect();
            let encoded = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7PD0-P0BBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWltcXV5fYGFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6e3x9fn-AgYKDhIWGh4iJiouMjY6PkJGSk5SVlpeYmZqbnJ2en6ChoqOkpaanqKmqq6ytrq-wsbKztLW2t7i5uru8vb6_wMHCw8TFxsfIycrLzM3Oz9DR0tPU1dbX2Nna29zd3t_g4eLj5OXm5-jp6uvs7e7v8PHy8_T19vf4-fr7_P3-_w==";

            let engine = Base64::url_safe();
            let out = engine.encode(&all);
            assert_eq!(encoded, out);

            let out = engine.decode(out).unwrap();
            assert_eq!(all, out);
        }
    }
}
