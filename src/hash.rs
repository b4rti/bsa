pub(crate) fn compute_hash(name: &str) -> u64 {
    let name = name.replace('/', r"\");
    if name.contains('\\') {
        // no file extension if we're looking as a directory containing dot chars
        return compute_hash_with_ext(name.as_bytes(), &[]);
    }
    if let Some(ext_idx) = name.rfind('.') {
        let (name, ext) = name.split_at(ext_idx);
        compute_hash_with_ext(name.as_bytes(), ext.as_bytes())
    } else {
        compute_hash_with_ext(name.as_bytes(), &[])
    }
}

fn compute_hash_with_ext(name: &[u8], ext: &[u8]) -> u64 {
    let name = name.to_ascii_lowercase();
    let ext = ext.to_ascii_lowercase();
    let hash_bytes = [
        if name.is_empty() {
            0x00
        } else {
            name[name.len() - 1]
        },
        if name.len() < 3 {
            0x00
        } else {
            name[name.len() - 2]
        },
        name.len() as u8,
        // not sure about this extra check
        if name.is_empty() { 0x00 } else { name[0] },
    ];
    let mut hash1 = u32::from_le_bytes(hash_bytes);
    match ext.as_slice() {
        b".kf" => hash1 |= 0x80,
        b".nif" => hash1 |= 0x8000,
        b".dds" => hash1 |= 0x8080,
        b".wav" => hash1 |= 0x8000_0000,
        _ => (),
    }
    let mut hash2 = 0_u32;
    // not sure about this extra check
    if name.len() >= 3 {
        for &n in &name[1..name.len() - 2] {
            hash2 = hash2.wrapping_mul(0x1003f).wrapping_add(u32::from(n));
        }
    }
    let mut hash3 = 0_u32;
    for &n in ext.as_slice() {
        hash3 = hash3.wrapping_mul(0x1003f).wrapping_add(u32::from(n));
    }
    (u64::from(hash2.wrapping_add(hash3)) << 32) + u64::from(hash1)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_hash_calculation() {
        assert_eq!(
            super::compute_hash("textures/terrain/skuldafnworld"),
            0x0fd0_dbef_741e_6c64
        );
        assert_eq!(
            super::compute_hash("textures/terrain/dlc2solstheimworld/objects"),
            0xe38e_0b87_742b_7473
        );
        assert_eq!(
            super::compute_hash("skuldafnworld.4.20.-5.dds"),
            0xa106_a998_7315_adb5
        );
        assert_eq!(
            super::compute_hash(r"meshes\actors\character\facegendata\facegeom\update.esm"),
            0x7e7d_d467_6d37_736d
        );
        assert_eq!(super::compute_hash("seq"), 0x7303_6571);
    }
}
