use super::types::RenderMaterialOverrideEntry;

pub fn mat_ovrd_to_bin(overrides: &[(u8, String)]) -> Vec<u8> {
    if overrides.is_empty() {
        return vec![0u8];
    }

    let mut buf = Vec::with_capacity(1 + overrides.len() * 64);
    buf.push(overrides.len() as u8);

    for (te_index, data) in overrides {
        let bytes = data.as_bytes();
        let len = bytes.len() as u16;
        buf.push(*te_index);
        buf.push((len & 0xff) as u8);
        buf.push(((len >> 8) & 0xff) as u8);
        buf.extend_from_slice(bytes);
    }

    buf
}

pub fn mat_ovrd_from_bin(data: &[u8]) -> Vec<RenderMaterialOverrideEntry> {
    if data.is_empty() || data[0] == 0 {
        return Vec::new();
    }

    let count = data[0] as usize;
    let mut entries = Vec::with_capacity(count);
    let mut offset = 1;

    for _ in 0..count {
        if offset + 3 > data.len() {
            break;
        }
        let te_index = data[offset];
        let len_lo = data[offset + 1] as u16;
        let len_hi = data[offset + 2] as u16;
        let len = (len_lo | (len_hi << 8)) as usize;
        offset += 3;

        if offset + len > data.len() {
            break;
        }
        let s = String::from_utf8_lossy(&data[offset..offset + len]).to_string();
        offset += len;

        entries.push(RenderMaterialOverrideEntry { te_index, data: s });
    }

    entries
}

pub fn overrides_to_tuples(entries: &[RenderMaterialOverrideEntry]) -> Vec<(u8, String)> {
    entries.iter().map(|e| (e.te_index, e.data.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_empty() {
        let bin = mat_ovrd_to_bin(&[]);
        assert_eq!(bin, vec![0u8]);
        let entries = mat_ovrd_from_bin(&bin);
        assert!(entries.is_empty());
    }

    #[test]
    fn test_roundtrip_single() {
        let overrides = vec![(0u8, r#"{"bc":[1,0,0,1],"mf":0.5}"#.to_string())];
        let bin = mat_ovrd_to_bin(&overrides);
        let entries = mat_ovrd_from_bin(&bin);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].te_index, 0);
        assert_eq!(entries[0].data, r#"{"bc":[1,0,0,1],"mf":0.5}"#);
    }

    #[test]
    fn test_roundtrip_multi() {
        let overrides = vec![
            (0u8, r#"{"bc":[1,0,0,1]}"#.to_string()),
            (2u8, r#"{"mf":0.8,"rf":0.2}"#.to_string()),
        ];
        let bin = mat_ovrd_to_bin(&overrides);
        let entries = mat_ovrd_from_bin(&bin);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].te_index, 0);
        assert_eq!(entries[1].te_index, 2);
        assert_eq!(entries[1].data, r#"{"mf":0.8,"rf":0.2}"#);
    }
}
