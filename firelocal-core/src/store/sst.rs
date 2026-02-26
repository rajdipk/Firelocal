use crate::store::io::Storage;
use crate::store::memtable::{Entry, Memtable};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

// Simple SST format:
// Record: [flag: u8] [k_len: u32] [key_bytes] [v_len: u32] [val_bytes]
// flag: 0 = Put, 1 = Delete

const FLAG_PUT: u8 = 0;
const FLAG_DELETE: u8 = 1;

pub struct SstBuilder<F: Write> {
    writer: BufWriter<F>,
}

impl<F: Write> SstBuilder<F> {
    pub fn new<S: Storage<File = F>>(storage: &S, path: impl AsRef<Path>) -> io::Result<Self> {
        let file = storage.create(path.as_ref())?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn build(mut self, memtable: &Memtable) -> io::Result<()> {
        for (key, entry) in memtable.iter() {
            let key_bytes = key.as_bytes();
            let k_len = key_bytes.len() as u32;

            match entry {
                Entry::Put(val) => {
                    self.writer.write_all(&[FLAG_PUT])?;
                    self.writer.write_all(&k_len.to_le_bytes())?;
                    self.writer.write_all(key_bytes)?;

                    let v_len = val.len() as u32;
                    self.writer.write_all(&v_len.to_le_bytes())?;
                    self.writer.write_all(val)?;
                }
                Entry::Delete => {
                    self.writer.write_all(&[FLAG_DELETE])?;
                    self.writer.write_all(&k_len.to_le_bytes())?;
                    self.writer.write_all(key_bytes)?;
                    // No value for delete
                    self.writer.write_all(&0u32.to_le_bytes())?; // v_len = 0 generic
                }
            }
        }
        self.writer.flush()?;
        Ok(())
    }
}

pub struct SstReader<F: Read + Seek> {
    file: BufReader<F>,
}

impl<F: Read + Seek> SstReader<F> {
    pub fn open<S: Storage<File = F>>(storage: &S, path: impl AsRef<Path>) -> io::Result<Self> {
        let file = storage.open(path.as_ref())?;
        Ok(Self {
            file: BufReader::new(file),
        })
    }

    // Very inefficient linear scan for M1
    pub fn get(&mut self, search_key: &str) -> io::Result<SstSearchResult> {
        self.file.seek(SeekFrom::Start(0))?; // Reset to start

        let mut flag_buf = [0u8; 1];
        let mut len_buf = [0u8; 4];

        loop {
            // Read flag
            if self.file.read(&mut flag_buf)? == 0 {
                return Ok(SstSearchResult::NotFound); // EOF
            }
            let flag = flag_buf[0];

            // Read k_len
            self.file.read_exact(&mut len_buf)?;
            let k_len = u32::from_le_bytes(len_buf) as usize;

            // Read key
            let mut key_buf = vec![0u8; k_len];
            self.file.read_exact(&mut key_buf)?;
            let key = String::from_utf8_lossy(&key_buf);

            // Read v_len
            self.file.read_exact(&mut len_buf)?;
            let v_len = u32::from_le_bytes(len_buf) as usize;

            if key == search_key {
                if flag == FLAG_DELETE {
                    self.file.seek(SeekFrom::Current(v_len as i64))?;
                    return Ok(SstSearchResult::Deleted);
                } else {
                    let mut val_buf = vec![0u8; v_len];
                    self.file.read_exact(&mut val_buf)?;
                    return Ok(SstSearchResult::Found(val_buf));
                }
            } else {
                // Skip value
                self.file.seek(SeekFrom::Current(v_len as i64))?;
            }
        }
    }
}

pub enum SstSearchResult {
    Found(Vec<u8>),
    Deleted,
    NotFound,
}
