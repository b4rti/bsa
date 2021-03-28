use std::{error, fmt};

#[derive(Clone, Debug)]
pub enum ReadError {
    MissingHeader,
    UnknownVersion(u32),
    UnexpectedEOF,
    UnexpectedFolderRecordOffset,
    CompressionUnsupported,
    UndecodableCharacters,
    ExpectedNullByte,
    FailedToReadFileOffset,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingHeader => write!(f, "BSA file header is missing or invalid"),
            Self::UnknownVersion(value) => write!(f, "Unknown BSA version: {}", value),
            Self::UnexpectedEOF => write!(f, "Unexpected end of file"),
            Self::UnexpectedFolderRecordOffset => write!(f, "Unexpected folder record offset"),
            Self::CompressionUnsupported => write!(f, "Compression is not currently supported"),
            Self::UndecodableCharacters => write!(f, "Undecodable characters found"),
            Self::ExpectedNullByte => write!(f, "Expected a null byte"),
            Self::FailedToReadFileOffset => write!(f, "Failed to read file offset"),
        }
    }
}

impl error::Error for ReadError {
}

#[derive(Clone, Debug)]
pub enum WriteError {
    UnencodableCharacters,
    FileNameMoreThan255Characters,
    CompressionUnsupported,
    MissingFileName,
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnencodableCharacters => write!(f, "Unencodable characters found"),
            Self::CompressionUnsupported => write!(f, "Compression is not currently supported"),
            Self::FileNameMoreThan255Characters => write!(f, "File name is longer than 255 characters"),
            Self::MissingFileName => write!(f, "Missing file name"),
        }
    }
}

impl error::Error for WriteError {
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Version {
    Oblivion,
    Skyrim,
    SkyrimSpecialEdition
}

impl Version {
    fn serialize(self) -> u32 {
        match self {
            Self::Oblivion => 103,
            Self::Skyrim => 104,
            Self::SkyrimSpecialEdition => 105,
        }
    }

    fn deserialize(value: u32) -> Result<Self, ReadError> {
        Ok(match value {
            103 => Self::Oblivion,
            104 => Self::Skyrim,
            105 => Self::SkyrimSpecialEdition,
            other => return Err(ReadError::UnknownVersion(other)),
        })
    }
}

#[derive(Clone, Copy, Debug)]
struct ArchiveFlags {
    include_directory_names: bool,
    include_file_names: bool,
    compressed_archive: bool,
    retain_directory_names: bool,
    retain_file_names: bool,
    retain_file_name_offsets: bool,
    xbox360_archive: bool,
    retain_strings: bool,
    embed_file_names: bool,
    xmem_codec: bool,
}

impl ArchiveFlags {
    fn serialize(self) -> u32 {
        let mut res = 0;
        if self.include_directory_names { res |= 0x01; }
        if self.include_file_names { res |= 0x02; }
        if self.compressed_archive { res |= 0x04; }
        if self.retain_directory_names { res |= 0x08; }
        if self.retain_file_names { res |= 0x10; }
        if self.retain_file_name_offsets { res |= 0x20; }
        if self.xbox360_archive { res |= 0x40; }
        if self.retain_strings { res |= 0x80; }
        if self.embed_file_names { res |= 0x100; }
        if self.xmem_codec { res |= 0x200; }
        res
    }

    fn deserialize(value: u32) -> Self {
        let mut res = Self {
            include_directory_names: false,
            include_file_names: false,
            compressed_archive: false,
            retain_directory_names: false,
            retain_file_names: false,
            retain_file_name_offsets: false,
            xbox360_archive: false,
            retain_strings: false,
            embed_file_names: false,
            xmem_codec: false,
        };
        if (value & 0x01) != 0 { res.include_directory_names = true; }
        if (value & 0x02) != 0 { res.include_file_names = true; }
        if (value & 0x04) != 0 { res.compressed_archive = true; }
        if (value & 0x08) != 0 { res.retain_directory_names = true; }
        if (value & 0x10) != 0 { res.retain_file_names = true; }
        if (value & 0x20) != 0 { res.retain_file_name_offsets = true; }
        if (value & 0x40) != 0 { res.xbox360_archive = true; }
        if (value & 0x80) != 0 { res.retain_strings = true; }
        if (value & 0x100) != 0 { res.embed_file_names = true; }
        if (value & 0x200) != 0 { res.xmem_codec = true; }
        res
    }
}

#[derive(Clone, Copy, Debug)]
struct FileFlags {
    meshes: bool,
    textures: bool,
    menus: bool,
    sounds: bool,
    voices: bool,
    shaders: bool,
    trees: bool,
    fonts: bool,
    miscellaneous: bool,
}

impl FileFlags {
    fn serialize(self) -> u32 {
        let mut res = 0;
        if self.meshes { res |= 0x01; }
        if self.textures { res |= 0x02; }
        if self.menus { res |= 0x04; }
        if self.sounds { res |= 0x08; }
        if self.voices { res |= 0x10; }
        if self.shaders { res |= 0x20; }
        if self.trees { res |= 0x40; }
        if self.fonts { res |= 0x80; }
        if self.miscellaneous { res |= 0x100; }
        res
    }

    fn deserialize(value: u32) -> Self {
        let mut res = Self {
            meshes: false,
            textures: false,
            menus: false,
            sounds: false,
            voices: false,
            shaders: false,
            trees: false,
            fonts: false,
            miscellaneous: false,
        };
        if (value & 0x01) != 0 { res.meshes = true; }
        if (value & 0x02) != 0 { res.textures = true; }
        if (value & 0x04) != 0 { res.menus = true; }
        if (value & 0x08) != 0 { res.sounds = true; }
        if (value & 0x10) != 0 { res.voices = true; }
        if (value & 0x20) != 0 { res.shaders = true; }
        if (value & 0x40) != 0 { res.trees = true; }
        if (value & 0x80) != 0 { res.fonts = true; }
        if (value & 0x100) != 0 { res.miscellaneous = true; }
        res
    }
}

#[derive(Clone)]
pub struct File<'a> {
    name: Option<String>,
    data: &'a [u8],
}

impl fmt::Debug for File<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "filename: {:?}, size: {}", self.name, self.data.len())
    }
}

fn serialize_bstring(s: &str, zero: bool, vec: &mut Vec<u8>) -> Result<(), WriteError> {
    let (encoded_str, _, errors) = encoding_rs::WINDOWS_1252.encode(s);
    let length = if zero {
        encoded_str.as_ref().len() + 1
    } else {
        encoded_str.as_ref().len()
    };
    if length > 255 {
        return Err(WriteError::FileNameMoreThan255Characters);
    }
    vec.push(length as u8);
    for b in encoded_str.as_ref() {
        vec.push(*b);
    }
    if zero {
        vec.push(0);
    }
    if errors {
        return Err(WriteError::UnencodableCharacters);
    }
    Ok(())
}

// returns (resulting string, bytes read)
fn deserialize_bstring(bytes: &[u8], zero: bool) -> Result<(String, usize), ReadError> {
    let name_length = if zero {
        bytes[0] as usize - 1
    } else {
        bytes[0] as usize
    };
    let encoded_filename = &bytes[1..(name_length + 1)];
    let (decoded_name, _, errors) = encoding_rs::WINDOWS_1252.decode(encoded_filename);
    if errors {
        return Err(ReadError::UndecodableCharacters);
    }
    if zero {
        if bytes[name_length + 1] != 0 {
            return Err(ReadError::ExpectedNullByte)
        }
    }
    Ok((decoded_name.into_owned(), 1 + name_length + if zero { 1 } else { 0 }))
}

// returns (resulting string, bytes read)
fn deserialize_null_terminated_string(bytes: &[u8]) -> Result<(String, usize), ReadError> {
    let mut name_length = None;
    for i in 0..bytes.len() {
        if bytes[i] == 0 {
            name_length = Some(i);
            break;
        }
    }
    if let Some(name_length) = name_length {
        let encoded_filename = &bytes[..name_length];
        let (decoded_name, _, errors) = encoding_rs::WINDOWS_1252.decode(encoded_filename);
        if errors {
            return Err(ReadError::UndecodableCharacters);
        }
        Ok((decoded_name.into_owned(), 1 + name_length))
    } else {
        Err(ReadError::ExpectedNullByte)
    }
}

impl<'a> File<'a> {
    fn serialize(&self, archive_flags: ArchiveFlags, compress: bool) -> Result<Vec<u8>, WriteError> {
        if compress {
            return Err(WriteError::CompressionUnsupported)
        }
        let mut res = vec![];
        if archive_flags.embed_file_names {
            if let Some(name) = &self.name {
                serialize_bstring(&name, false, &mut res)?;
            } else {
                return Err(WriteError::MissingFileName);
            }
        }
        for b in self.data {
            res.push(*b);
        }
        Ok(res)
    }

    fn deserialize(
        archive_flags: ArchiveFlags,
        compress: bool,
        size: usize,
        mut data: &'a [u8]
    ) -> Result<(Self, &[u8]), ReadError> {
        if compress {
            return Err(ReadError::CompressionUnsupported);
        }
        let name = if archive_flags.embed_file_names {
            let (name, bytes_read) = deserialize_bstring(data, false)?;
            data = &data[bytes_read..];
            Some(name)
        } else {
            None
        };
        Ok((Self {
            name,
            data: &data[..size]
        }, &data[size..]))
    }
}

#[derive(Debug, Clone)]
pub struct Folder<'a> {
    name: Option<String>,
    files: Vec<File<'a>>,
}

#[derive(Debug, Clone)]
pub struct Bsa<'a> {
    version: Version,
    archive_flags: ArchiveFlags,
    folder_count: u32,
    file_count: u32,
    total_folder_name_length: u32,
    total_file_name_length: u32,
    file_flags: FileFlags,
    folders: Vec<Folder<'a>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Hash(u64);

#[derive(Debug, Clone)]
struct FolderRecord {
    name_hash: u64,
    name: Option<String>,
    file_count: u32,
    offset: u64,
    file_records: Vec<FileRecord>,
}

#[derive(Debug, Clone)]
struct FileRecord {
    name_hash: u64,
    size: u32,
    offset: u32,
    name: Option<String>,
}

impl<'a> Bsa<'a> {
    pub fn folders(&self) -> &[Folder] {
        &self.folders
    }

    fn read_u32(data: &[u8], archive_flags: Option<ArchiveFlags>) -> Result<(u32, &[u8]), ReadError> {
        if data.len() < 4 {
            Err(ReadError::UnexpectedEOF)
        } else {
            let bytes = [data[0], data[1], data[2], data[3]];
            if archive_flags.is_some() && archive_flags.unwrap().xbox360_archive {
                Ok((u32::from_be_bytes(bytes), &data[4..]))
            } else {
                Ok((u32::from_le_bytes(bytes), &data[4..]))
            }
        }
    }

    fn read_u64(data: &[u8], archive_flags: Option<ArchiveFlags>) -> Result<(u64, &[u8]), ReadError> {
        if data.len() < 8 {
            Err(ReadError::UnexpectedEOF)
        } else {
            let bytes = [data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]];
            if archive_flags.is_some() && archive_flags.unwrap().xbox360_archive {
                Ok((u64::from_be_bytes(bytes), &data[8..]))
            } else {
                Ok((u64::from_le_bytes(bytes), &data[8..]))
            }
        }
    }

    fn read_header(mut data: &[u8]) -> Result<(Bsa<'static>, &[u8]), ReadError> {
        if &data[0..4] != b"BSA\0" {
            return Err(ReadError::MissingHeader)
        } else {
            data = &data[4..];
        }
        let (version_num, data) = Self::read_u32(data, None)?;
        let version = Version::deserialize(version_num)?;
        let (offset, data) = Self::read_u32(data, None)?;
        if offset != 36 {
            return Err(ReadError::UnexpectedFolderRecordOffset);
        }
        let (archive_flags_u64, data) = Self::read_u32(data, None)?;
        let archive_flags = ArchiveFlags::deserialize(archive_flags_u64);
        let (folder_count, data) = Self::read_u32(data, Some(archive_flags))?;
        let (file_count, data) = Self::read_u32(data, Some(archive_flags))?;
        let (total_folder_name_length, data) = Self::read_u32(data, Some(archive_flags))?;
        let (total_file_name_length, data) = Self::read_u32(data, Some(archive_flags))?;
        let (file_flags_u64, data) = Self::read_u32(data, None)?;
        let file_flags = FileFlags::deserialize(file_flags_u64);
        Ok((Bsa {
            version,
            archive_flags,
            folder_count,
            file_count,
            total_folder_name_length,
            total_file_name_length,
            file_flags,
            folders: vec![],
        }, data))
    }

    pub fn read(data: &'a [u8]) -> Result<Self, ReadError> {
        let (mut res, mut data) = Self::read_header(data)?;

        // read folder records
        let mut folder_records = vec![];
        for _ in 0..res.folder_count {
            let (name_hash, remaining) = Self::read_u64(data, Some(res.archive_flags))?;
            let (file_count, remaining) = Self::read_u32(remaining, Some(res.archive_flags))?;
            let (old_file_offset, remaining) = Self::read_u32(remaining, Some(res.archive_flags))?;
            data = remaining;
            let offset = match res.version {
                Version::Skyrim => u64::from(old_file_offset),
                Version::SkyrimSpecialEdition => {
                    let (offset, remaining) = Self::read_u64(data, Some(res.archive_flags))?;
                    data = remaining;
                    offset
                }
                _ => return Err(ReadError::FailedToReadFileOffset),
            };
            folder_records.push(FolderRecord {
                name_hash,
                file_count,
                offset,
                file_records: vec![],
                name: None,
            });
        }

        // read file record blocks
        for folder_record in &mut folder_records {
            if res.archive_flags.include_directory_names {
                let (name, len) = deserialize_bstring(data, true)?;
                folder_record.name = Some(name);
                data = &data[len..];
            }
            for _ in 0..folder_record.file_count {
                let (name_hash, remaining) = Self::read_u64(data, Some(res.archive_flags))?;
                let (size, remaining) = Self::read_u32(remaining, Some(res.archive_flags))?;
                let (offset, remaining) = Self::read_u32(remaining, Some(res.archive_flags))?;
                folder_record.file_records.push(FileRecord {
                    name_hash,
                    size,
                    offset,
                    name: None,
                });
                data = remaining;
            }
        }

        if res.archive_flags.include_file_names {
            // read file name block
            for folder_record in &mut folder_records {
                for file_record in &mut folder_record.file_records {
                    let (file_name, bytes_read) = deserialize_null_terminated_string(data)?;
                    data = &data[bytes_read..];
                    file_record.name = Some(file_name);
                }
            }
        }

        for folder_record in folder_records {
            let mut folder = Folder {
                name: folder_record.name,
                files: vec![],
            };
            for file_record in folder_record.file_records {
                let (mut file, remaining) = File::deserialize(res.archive_flags, false, file_record.size as usize, data)?;
                if file.name.is_none() && file_record.name.is_some() {
                    file.name = file_record.name;
                }
                folder.files.push(file);
                data = remaining;
            }
            res.folders.push(folder);
        }

        Ok(res)
    }

    fn write_u32(v: &mut Vec<u8>, value: u32, archive_flags: Option<ArchiveFlags>) {
        let bytes = if archive_flags.is_some() && archive_flags.unwrap().xbox360_archive {
            value.to_be_bytes()
        } else {
            value.to_le_bytes()
        };
        for b in std::array::IntoIter::new(bytes) {
            v.push(b);
        }
    }

    pub fn write(&self) -> Vec<u8> {
        let mut res = vec![b'B', b'S', b'A', 0x00];
        Self::write_u32(&mut res, self.version.serialize(), None);
        Self::write_u32(&mut res, self.archive_flags.serialize(), None);
        Self::write_u32(&mut res, self.folder_count, Some(self.archive_flags));
        Self::write_u32(&mut res, self.file_count, Some(self.archive_flags));
        Self::write_u32(&mut res, self.total_folder_name_length, Some(self.archive_flags));
        Self::write_u32(&mut res, self.total_file_name_length, Some(self.archive_flags));
        Self::write_u32(&mut res, self.file_flags.serialize(), Some(self.archive_flags));
        res
    }
}
