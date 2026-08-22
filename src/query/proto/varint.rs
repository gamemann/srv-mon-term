use crate::query::types::error::QueryError;

/// Encodes a Minecraft protocol VarInt.
pub fn write_varint(value: i32, out: &mut Vec<u8>) {
    let mut val = value as u32;

    loop {
        if val & !0x7F == 0 {
            out.push(val as u8);

            return;
        }

        out.push(((val & 0x7F) as u8) | 0x80);

        val >>= 7;
    }
}

/// Decodes a Minecraft protocol VarInt, returning the value and the amount of bytes consumed.
pub fn read_varint(buf: &[u8]) -> Result<(i32, usize), QueryError> {
    let mut result: u32 = 0;
    let mut read = 0usize;

    loop {
        let byte = *buf
            .get(read)
            .ok_or_else(|| QueryError::InvalidResponse("truncated varint".to_string()))?;

        result |= ((byte & 0x7F) as u32) << (7 * read);

        read += 1;

        if byte & 0x80 == 0 {
            return Ok((result as i32, read));
        }

        if read >= 5 {
            return Err(QueryError::InvalidResponse("varint too large".to_string()));
        }
    }
}

/// Encodes a length-prefixed UTF-8 string as used by the Minecraft protocol.
pub fn write_string(value: &str, out: &mut Vec<u8>) {
    write_varint(value.len() as i32, out);

    out.extend_from_slice(value.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_varints() {
        for value in [0, 1, 127, 128, 255, 2097151, 2147483647, -1] {
            let mut buf = Vec::new();

            write_varint(value, &mut buf);

            let (decoded, read) = read_varint(&buf).expect("failed to decode");

            assert_eq!(decoded, value);
            assert_eq!(read, buf.len());
        }
    }

    #[test]
    fn known_encodings() {
        let mut buf = Vec::new();
        write_varint(300, &mut buf);

        assert_eq!(buf, vec![0xAC, 0x02]);
    }

    #[test]
    fn rejects_truncated_varint() {
        assert!(read_varint(&[0x80]).is_err());
        assert!(read_varint(&[]).is_err());
    }
}
