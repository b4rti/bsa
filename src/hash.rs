use crate::cp1252;

#[non_exhaustive]
pub(crate) enum Type {
    Directory,
    File,
}

pub(crate) fn compute_hash(name: &str, t: Type) -> Result<u64, cp1252::EncodingError> {
    let name = name.replace('/', r"\");
    Ok(match t {
        Type::Directory => compute_hash_with_ext(&cp1252::encode_str(&name)?, &[]),
        Type::File => {
            if let Some(ext_idx) = name.rfind('.') {
                let (name, ext) = name.split_at(ext_idx);
                compute_hash_with_ext(&cp1252::encode_str(&name)?, &cp1252::encode_str(&ext)?)
            } else {
                compute_hash_with_ext(&cp1252::encode_str(&name)?, &[])
            }
        }
    })
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
    use super::{compute_hash, Type};

    #[test]
    fn test_hash_calculation() -> Result<(), crate::cp1252::EncodingError> {
        assert_eq!(
            compute_hash("textures/terrain/skuldafnworld", Type::Directory)?,
            0x0fd0_dbef_741e_6c64
        );
        assert_eq!(
            compute_hash(
                "textures/terrain/dlc2solstheimworld/objects",
                Type::Directory
            )?,
            0xe38e_0b87_742b_7473
        );
        assert_eq!(
            compute_hash("skuldafnworld.4.20.-5.dds", Type::File)?,
            0xa106_a998_7315_adb5
        );
        assert_eq!(
            compute_hash(
                r"meshes\actors\character\facegendata\facegeom\update.esm",
                Type::Directory
            )?,
            0x7e7d_d467_6d37_736d
        );
        assert_eq!(compute_hash("seq", Type::Directory)?, 0x7303_6571);
        assert_eq!(
            compute_hash("dlc2mq05__0003c745_1\u{a0}.fuz", Type::File)?,
            0x9482_28c0_6415_31a0
        );
        Ok(())
    }
}
