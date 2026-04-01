//! Tar.gz archive read/write utilities

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tar::{Builder, Header};
use tracing::{debug, info};

/// Reader for tar.gz archives
pub struct TarGzReader {
    entries: HashMap<String, Vec<u8>>,
}

impl TarGzReader {
    /// Open and read a tar.gz archive into memory.
    /// Uses a custom raw tar parser that handles C# OpenSim's GNU LongLink format
    /// (././@LongLink type 'L' entries for paths > 100 chars).
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open archive: {:?}", path.as_ref()))?;

        let decoder = GzDecoder::new(file);
        let entries = Self::read_tar_raw(decoder)?;

        let asset_count = entries.keys().filter(|k| k.starts_with("assets/")).count();
        let inv_count = entries.keys().filter(|k| k.starts_with("inventory/")).count();
        let obj_count = entries.keys().filter(|k| k.starts_with("objects/")).count();
        let other_count = entries.len() - asset_count - inv_count - obj_count;
        info!("Loaded {} entries from archive (assets={}, inventory={}, objects={}, other={})",
            entries.len(), asset_count, inv_count, obj_count, other_count);
        for key in entries.keys() {
            info!("[TAR] entry key: '{}' (size={})", key, entries[key].len());
        }

        Ok(Self { entries })
    }

    fn read_tar_raw<R: Read>(mut reader: R) -> Result<HashMap<String, Vec<u8>>> {
        let mut entries = HashMap::new();
        let mut header_buf = [0u8; 512];
        let mut long_link_entries = 0u64;

        loop {
            if reader.read_exact(&mut header_buf).is_err() {
                break;
            }

            if header_buf.iter().all(|&b| b == 0) {
                break;
            }

            let entry_type = header_buf[156];
            let file_size = Self::parse_octal(&header_buf[124..135]);

            if entry_type == b'L' {
                let long_path = Self::read_data_string(&mut reader, file_size)?;
                long_link_entries += 1;

                if reader.read_exact(&mut header_buf).is_err() {
                    break;
                }

                let real_size = Self::parse_octal(&header_buf[124..135]);
                let real_type = header_buf[156];

                if real_type == b'5' || real_type == b'D' {
                    Self::skip_data(&mut reader, real_size)?;
                    entries.insert(long_path, Vec::new());
                } else {
                    let data = Self::read_data(&mut reader, real_size)?;
                    entries.insert(long_path, data);
                }
            } else {
                let file_path = Self::parse_path(&header_buf[0..100]);

                if file_path.is_empty() {
                    Self::skip_data(&mut reader, file_size)?;
                    continue;
                }

                if entry_type == b'5' || entry_type == b'D' {
                    Self::skip_data(&mut reader, file_size)?;
                    entries.insert(file_path, Vec::new());
                } else {
                    let data = Self::read_data(&mut reader, file_size)?;
                    entries.insert(file_path, data);
                }
            }
        }

        if long_link_entries > 0 {
            info!("Processed {} GNU LongLink entries for paths > 100 chars", long_link_entries);
        }

        Ok(entries)
    }

    fn parse_octal(bytes: &[u8]) -> usize {
        let s = std::str::from_utf8(bytes).unwrap_or("");
        let trimmed = s.trim_matches(|c: char| c == '\0' || c == ' ');
        usize::from_str_radix(trimmed, 8).unwrap_or(0)
    }

    fn parse_path(bytes: &[u8]) -> String {
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).to_string()
    }

    fn read_data<R: Read>(reader: &mut R, size: usize) -> Result<Vec<u8>> {
        if size == 0 {
            return Ok(Vec::new());
        }
        let padded = (size + 511) & !511;
        let mut buf = vec![0u8; padded];
        reader.read_exact(&mut buf)?;
        buf.truncate(size);
        Ok(buf)
    }

    fn read_data_string<R: Read>(reader: &mut R, size: usize) -> Result<String> {
        let data = Self::read_data(reader, size)?;
        let s = String::from_utf8_lossy(&data);
        Ok(s.trim_matches('\0').to_string())
    }

    fn skip_data<R: Read>(reader: &mut R, size: usize) -> Result<()> {
        if size == 0 {
            return Ok(());
        }
        let padded = (size + 511) & !511;
        let mut buf = vec![0u8; padded];
        reader.read_exact(&mut buf)?;
        Ok(())
    }

    /// Get entry by path
    pub fn get(&self, path: &str) -> Option<&[u8]> {
        self.entries.get(path).map(|v| v.as_slice())
    }

    /// Get all entries with a given prefix
    pub fn get_entries_with_prefix(&self, prefix: &str) -> Vec<(&String, &Vec<u8>)> {
        self.entries
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .collect()
    }

    /// Get all entry paths
    pub fn paths(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if archive is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all asset entries (files in assets/ directory)
    pub fn get_asset_entries(&self) -> Vec<(&String, &Vec<u8>)> {
        self.get_entries_with_prefix("assets/")
    }

    /// Get all inventory entries (files in inventory/ directory)
    pub fn get_inventory_entries(&self) -> Vec<(&String, &Vec<u8>)> {
        self.get_entries_with_prefix("inventory/")
    }

    /// Get all object entries (files in objects/ directory)
    pub fn get_object_entries(&self) -> Vec<(&String, &Vec<u8>)> {
        self.get_entries_with_prefix("objects/")
    }

    /// Get archive.xml content
    pub fn get_archive_xml(&self) -> Option<&[u8]> {
        self.get("archive.xml")
    }
}

/// Builder for tar.gz archives
pub struct TarGzWriter {
    builder: Builder<GzEncoder<File>>,
    entry_count: u32,
}

impl TarGzWriter {
    /// Create a new tar.gz archive for writing
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path.as_ref())
            .with_context(|| format!("Failed to create archive: {:?}", path.as_ref()))?;

        let encoder = GzEncoder::new(file, Compression::default());
        let builder = Builder::new(encoder);

        Ok(Self {
            builder,
            entry_count: 0,
        })
    }

    /// Add a file entry to the archive
    pub fn add_file(&mut self, path: &str, data: &[u8]) -> Result<()> {
        let mut header = Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());
        header.set_entry_type(tar::EntryType::Regular);

        if path.len() <= 100 {
            header.set_path(path)?;
            header.set_cksum();
            self.builder.append(&header, data)?;
        } else {
            self.builder.append_data(&mut header, path, data)?;
        }
        self.entry_count += 1;

        debug!("Added archive entry: {} ({} bytes)", path, data.len());

        Ok(())
    }

    /// Add a directory entry (empty directory marker)
    pub fn add_directory(&mut self, path: &str) -> Result<()> {
        let path = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{}/", path)
        };

        let mut header = Header::new_gnu();
        header.set_size(0);
        header.set_mode(0o755);
        header.set_entry_type(tar::EntryType::Directory);
        header.set_mtime(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());

        if path.len() <= 100 {
            header.set_path(&path)?;
            header.set_cksum();
            self.builder.append(&header, &[] as &[u8])?;
        } else {
            self.builder.append_data(&mut header, &path, &[] as &[u8])?;
        }

        Ok(())
    }

    /// Finalize and close the archive
    pub fn finish(self) -> Result<u32> {
        let encoder = self.builder.into_inner()?;
        encoder.finish()?;
        info!("Archive created with {} entries", self.entry_count);
        Ok(self.entry_count)
    }

    /// Get current entry count
    pub fn entry_count(&self) -> u32 {
        self.entry_count
    }
}

/// Streaming reader for large archives (doesn't load all into memory).
/// Uses same raw tar parser as TarGzReader to handle C# OpenSim's GNU LongLink format.
pub struct TarGzStreamReader<R: Read> {
    reader: GzDecoder<R>,
}

impl TarGzStreamReader<File> {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())?;
        let decoder = GzDecoder::new(file);
        Ok(Self { reader: decoder })
    }
}

impl<R: Read> TarGzStreamReader<R> {
    pub fn new(reader: R) -> Self {
        let decoder = GzDecoder::new(reader);
        Self { reader: decoder }
    }

    pub fn for_each_entry<F>(&mut self, mut handler: F) -> Result<u32>
    where
        F: FnMut(&str, &[u8]) -> Result<()>,
    {
        let mut count = 0;
        let mut header_buf = [0u8; 512];

        loop {
            if self.reader.read_exact(&mut header_buf).is_err() {
                break;
            }

            if header_buf.iter().all(|&b| b == 0) {
                break;
            }

            let entry_type = header_buf[156];
            let file_size = TarGzReader::parse_octal(&header_buf[124..135]);

            if entry_type == b'L' {
                let long_path = TarGzReader::read_data_string(&mut self.reader, file_size)?;

                if self.reader.read_exact(&mut header_buf).is_err() {
                    break;
                }

                let real_size = TarGzReader::parse_octal(&header_buf[124..135]);
                let data = TarGzReader::read_data(&mut self.reader, real_size)?;
                handler(&long_path, &data)?;
                count += 1;
            } else {
                let file_path = TarGzReader::parse_path(&header_buf[0..100]);

                if file_path.is_empty() {
                    TarGzReader::skip_data(&mut self.reader, file_size)?;
                    continue;
                }

                let data = TarGzReader::read_data(&mut self.reader, file_size)?;
                handler(&file_path, &data)?;
                count += 1;
            }
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_create_and_read_archive() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Create archive
        {
            let mut writer = TarGzWriter::create(path).unwrap();
            writer.add_file("test.txt", b"Hello, World!").unwrap();
            writer.add_file("folder/nested.txt", b"Nested content").unwrap();
            writer.finish().unwrap();
        }

        // Read archive
        {
            let reader = TarGzReader::open(path).unwrap();
            assert_eq!(reader.len(), 2);
            assert_eq!(reader.get("test.txt"), Some(b"Hello, World!".as_slice()));
            assert_eq!(reader.get("folder/nested.txt"), Some(b"Nested content".as_slice()));
        }
    }
}
