#![allow(unused_imports, dead_code)]

use std::{error, fmt, io::{self, Read, Seek, Write}};

#[non_exhaustive]
#[derive(Debug)]
pub enum ReadError {
    MissingHeader,
    UnknownVersion(u32),
    UnexpectedFolderRecordOffset,
    CompressionUnsupported,
    UndecodableCharacters,
    ExpectedNullByte,
    FailedToReadFileOffset,
    ReaderError(io::Error)
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MissingHeader => write!(f, "BSA file header is missing or invalid"),
            Self::UnknownVersion(value) => write!(f, "Unknown BSA version: {}", value),
            Self::UnexpectedFolderRecordOffset => write!(f, "Unexpected folder record offset"),
            Self::CompressionUnsupported => write!(f, "Compression is not currently supported"),
            Self::UndecodableCharacters => write!(f, "Undecodable characters found"),
            Self::ExpectedNullByte => write!(f, "Expected a null byte"),
            Self::FailedToReadFileOffset => write!(f, "Failed to read file offset"),
            Self::ReaderError(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for ReadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::ReaderError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> Self {
        Self::ReaderError(e)
    }
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

enum FileReader<'a> {
    None,
    Raw(&'a mut dyn Read)
}

pub struct File<'a> {
    name: Option<String>,
    offset: u64,
    size: u64,
    compressed: bool,
    data: FileReader<'a>,
}

#[derive(Debug)]
struct DataNotAvailable {}
impl fmt::Display for DataNotAvailable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Data is not available")
    }
}
impl error::Error for DataNotAvailable {}

impl<'a> Read for FileReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            FileReader::<'a>::None => Err(io::Error::new(io::ErrorKind::Other, DataNotAvailable {})),
            FileReader::<'a>::Raw(r) => r.read(buf)
        }
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

fn read_u8(reader: &mut impl Read) -> Result<u8, ReadError> {
    let mut buf = [0];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u32(reader: &mut impl Read, archive_flags: Option<ArchiveFlags>) -> Result<u32, ReadError> {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;
    if archive_flags.is_some() && archive_flags.unwrap().xbox360_archive {
        Ok(u32::from_be_bytes(buf))
    } else {
        Ok(u32::from_le_bytes(buf))
    }
}

fn read_u64(reader: &mut impl Read, archive_flags: Option<ArchiveFlags>) -> Result<u64, ReadError> {
    let mut buf = [0; 8];
    reader.read_exact(&mut buf)?;
    if archive_flags.is_some() && archive_flags.unwrap().xbox360_archive {
        Ok(u64::from_be_bytes(buf))
    } else {
        Ok(u64::from_le_bytes(buf))
    }
}

fn deserialize_bstring(bytes: &mut impl Read, zero: bool) -> Result<String, ReadError> {
    let length_byte = read_u8(bytes)?;
    let name_length = if zero {
        length_byte as usize - 1
    } else {
        length_byte as usize
    };
    let mut encoded_filename = Vec::with_capacity(name_length);
    encoded_filename.resize(name_length, 0);
    bytes.read_exact(&mut encoded_filename)?;
    let (decoded_name, _, errors) = encoding_rs::WINDOWS_1252.decode(&encoded_filename);
    if errors {
        return Err(ReadError::UndecodableCharacters);
    }
    if zero {
        let null_byte = read_u8(bytes)?;
        if null_byte != 0 {
            return Err(ReadError::ExpectedNullByte)
        }
    }
    Ok(decoded_name.into_owned())
}

fn deserialize_null_terminated_string(bytes: &mut impl Read) -> Result<String, ReadError> {
    let mut encoded_filename = vec![];
    loop {
        let byte = read_u8(bytes)?;
        if byte == 0 {
            break;
        }
        encoded_filename.push(byte);
    }
    let (decoded_name, _, errors) = encoding_rs::WINDOWS_1252.decode(&encoded_filename);
    if errors {
        return Err(ReadError::UndecodableCharacters);
    }
    Ok(decoded_name.into_owned())
}

impl<'a> File<'a> {
    // fn serialize(&self, archive_flags: ArchiveFlags, compress: bool) -> Result<io::Chain<&[u8], &mut R>, WriteError> {
    //     if compress {
    //         return Err(WriteError::CompressionUnsupported)
    //     }
    //     let mut res = vec![];
    //     if archive_flags.embed_file_names {
    //         if let Some(name) = &self.name {
    //             serialize_bstring(&name, false, &mut res)?;
    //         } else {
    //             return Err(WriteError::MissingFileName);
    //         }
    //     }
    //     Ok(res.chain(&mut self.data))
    // }

    fn deserialize(
        archive_flags: ArchiveFlags,
        compressed: bool,
        size: u64,
        data: &'a mut (impl Read + Seek),
    ) -> Result<File<'static>, ReadError> {
        let name = if archive_flags.embed_file_names {
            Some(deserialize_bstring(data, false)?)
        } else {
            None
        };
        if compressed {
            let original_size = read_u32(data, Some(archive_flags))?;
            println!("size: {}, original_size: {}", size, original_size);
        }
        let data_size = if compressed { size + 4 } else { size };
        data.seek(io::SeekFrom::Current(data_size as i64))?;
        Ok(File {
            name,
            offset: data.stream_position()?,
            size: data_size,
            compressed,
            data: FileReader::None,
        })
    }

    pub fn name(&self) -> Option<&str> {
        if let Some(name) = &self.name {
            Some(name.as_str())
        } else {
            None
        }
    }

    pub fn contents(&mut self) -> &mut (impl Read + 'a) {
        &mut self.data
    }
}

#[derive(Debug)]
pub struct Folder<'a> {
    name: Option<String>,
    files: Vec<File<'a>>,
}

impl<'a> Folder<'a> {
    pub fn files(&self) -> impl Iterator<Item = &File<'a>> {
        self.files.iter()
    }

    pub fn files_mut(&mut self) -> impl Iterator<Item = &mut File<'a>> {
        self.files.iter_mut()
    }

    pub fn name(&self) -> Option<&str> {
        if let Some(name) = &self.name {
            Some(name.as_str())
        } else {
            None
        }
    }
}

impl fmt::Debug for File<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "File {:?} (offset {}, size {}, compressed {})", self.name, self.offset, self.size, self.compressed)
    }
}

#[derive(Debug)]
struct BsaHeader<'a> {
    version: Version,
    archive_flags: ArchiveFlags,
    folder_count: u32,
    file_count: u32,
    total_folder_name_length: u32,
    total_file_name_length: u32,
    file_flags: FileFlags,
    folders: Vec<Folder<'a>>,
}

pub struct Bsa<'a, R: Read + 'a> {
    header: BsaHeader<'a>,
    reader: R,
}

impl<R: Read> fmt::Debug for Bsa<'_, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self.header)
    }
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

impl<'a, R: Read + Seek + 'a> Bsa<'a, R> {
    pub fn folders(&self) -> impl Iterator<Item = &Folder<'a>> {
        self.header.folders.iter()
    }

    pub fn folders_mut(&mut self) -> impl Iterator<Item = &mut Folder<'a>> {
        self.header.folders.iter_mut()
    }

    fn read_header(data: &mut R) -> Result<BsaHeader<'static>, ReadError> {
        let mut magic = [0; 4];
        data.read_exact(&mut magic)?;
        if &magic != b"BSA\0" {
            return Err(ReadError::MissingHeader)
        }
        let version_num = read_u32(data, None)?;
        let version = Version::deserialize(version_num)?;
        let offset = read_u32(data, None)?;
        if offset != 36 {
            return Err(ReadError::UnexpectedFolderRecordOffset);
        }
        let archive_flags_u64 = read_u32(data, None)?;
        let archive_flags = ArchiveFlags::deserialize(archive_flags_u64);
        let folder_count = read_u32(data, Some(archive_flags))?;
        let file_count = read_u32(data, Some(archive_flags))?;
        let total_folder_name_length = read_u32(data, Some(archive_flags))?;
        let total_file_name_length = read_u32(data, Some(archive_flags))?;
        let file_flags_u64 = read_u32(data, None)?;
        let file_flags = FileFlags::deserialize(file_flags_u64);

        let mut res = BsaHeader {
            version,
            archive_flags,
            folder_count,
            file_count,
            total_folder_name_length,
            total_file_name_length,
            file_flags,
            folders: vec![],
        };

        // read folder records
        let mut folder_records = vec![];
        for _ in 0..res.folder_count {
            let name_hash = read_u64(data, Some(res.archive_flags))?;
            let file_count = read_u32(data, Some(res.archive_flags))?;
            let old_file_offset = read_u32(data, Some(res.archive_flags))?;
            let offset = match res.version {
                Version::Skyrim => u64::from(old_file_offset),
                Version::SkyrimSpecialEdition => read_u64(data, Some(res.archive_flags))?,
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
                let name = deserialize_bstring(data, true)?;
                folder_record.name = Some(name);
            }
            for _ in 0..folder_record.file_count {
                let name_hash = read_u64(data, Some(res.archive_flags))?;
                let size = read_u32(data, Some(res.archive_flags))?;
                let offset = read_u32(data, Some(res.archive_flags))?;
                folder_record.file_records.push(FileRecord {
                    name_hash,
                    size,
                    offset,
                    name: None,
                });
            }
        }

        if res.archive_flags.include_file_names {
            // read file name block
            for folder_record in &mut folder_records {
                for file_record in &mut folder_record.file_records {
                    let file_name = deserialize_null_terminated_string(data)?;
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
                let override_compressed = if file_record.size & 0x40000000 != 0 { true } else { false };
                let compressed = archive_flags.compressed_archive != override_compressed;

                let mut file = File::deserialize(res.archive_flags, compressed, file_record.size.into(), data)?;
                if file.name.is_none() && file_record.name.is_some() {
                    file.name = file_record.name;
                }
                folder.files.push(file);
            }
            res.folders.push(folder);
        }

        Ok(res)
    }

    pub fn read(mut data: R) -> Result<Bsa<'static, R>, ReadError> {
        let header = Self::read_header(&mut data)?;
        Ok(Bsa {
            header,
            reader: data
        })
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

    // pub fn write(&self) -> Vec<u8> {
    //     let mut res = vec![b'B', b'S', b'A', 0x00];
    //     Self::write_u32(&mut res, self.version.serialize(), None);
    //     Self::write_u32(&mut res, self.archive_flags.serialize(), None);
    //     Self::write_u32(&mut res, self.folder_count, Some(self.archive_flags));
    //     Self::write_u32(&mut res, self.file_count, Some(self.archive_flags));
    //     Self::write_u32(&mut res, self.total_folder_name_length, Some(self.archive_flags));
    //     Self::write_u32(&mut res, self.total_file_name_length, Some(self.archive_flags));
    //     Self::write_u32(&mut res, self.file_flags.serialize(), Some(self.archive_flags));
    //     res
    // }
}
