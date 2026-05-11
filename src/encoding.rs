use crate::errors::ShsError;
use crate::shamir::Share;

// Encode a share as `"{index}-{hex}"`.
// Each u16 element is serialized as 2 big-endian bytes before hex-encoding.
pub fn encode_share(share: &Share) -> String {
    let bytes: Vec<u8> = share.data.iter()
        .flat_map(|&v| v.to_be_bytes())
        .collect();
    format!("{}-{}", share.index, hex::encode(bytes))
}

// Decode a `"{index}-{hex}"` string back into a Share.
pub fn decode_share(s: &str) -> Result<Share, ShsError> {
    let (idx_str, hex_str) = s.split_once('-').ok_or_else(|| ShsError::InvalidShareFormat {
        reason: "expected format 'index-hexdata'".into(),
    })?;

    let index: u8 = idx_str.parse().map_err(|_| ShsError::InvalidShareFormat {
        reason: format!("invalid index: '{}'", idx_str),
    })?;

    if index == 0 {
        return Err(ShsError::InvalidShareFormat {
            reason: "index must be >= 1".into(),
        });
    }

    let bytes = hex::decode(hex_str).map_err(|e| ShsError::InvalidShareFormat {
        reason: format!("bad hex: {}", e),
    })?;

    if bytes.len() % 2 != 0 {
        return Err(ShsError::InvalidShareFormat {
            reason: "hex data must have even byte count".into(),
        });
    }

    let data: Vec<u16> = bytes.chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();

    Ok(Share { index, data })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let share = Share { index: 3, data: vec![72, 101, 108] };
        let decoded = decode_share(&encode_share(&share)).unwrap();
        assert_eq!(decoded.index, 3);
        assert_eq!(decoded.data, vec![72, 101, 108]);
    }

    #[test]
    fn invalid_formats() {
        assert!(decode_share("nohyphen").is_err());
        assert!(decode_share("0-aabb").is_err());     // index 0
        assert!(decode_share("abc-aabb").is_err());   // non-numeric index
        assert!(decode_share("1-zzzz").is_err());     // bad hex
        assert!(decode_share("1-aab").is_err());       // odd hex length
    }
}
