use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// File lock for exclusive access
pub struct FileLock {
    _file: File,
}

impl FileLock {
    /// Create a new file lock
    fn new(file: File) -> Self {
        Self { _file: file }
    }
}

/// Trait representing a file handle
pub trait FileHandle: Read + Write + Seek + Send + Sync {
    fn set_len(&mut self, size: u64) -> io::Result<()>;
    fn sync_all(&mut self) -> io::Result<()>;
}

/// Trait representing file system operations
pub trait Storage: Send + Sync + 'static {
    type File: FileHandle;

    fn open(&self, path: &Path) -> io::Result<Self::File>;
    fn create(&self, path: &Path) -> io::Result<Self::File>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    fn read_dir(&self, path: &Path) -> io::Result<Vec<(PathBuf, SystemTime)>>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn exists(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    
    /// Acquire an exclusive lock on a file for critical operations
    fn lock_exclusive(&self, path: &Path) -> io::Result<FileLock>;
    /// Try to acquire an exclusive lock without blocking
    fn try_lock_exclusive(&self, path: &Path) -> io::Result<Option<FileLock>>;
}

// --- Standard Filesystem Implementation (Native) ---

pub struct StdFile(std::fs::File);

impl FileHandle for StdFile {
    fn set_len(&mut self, size: u64) -> io::Result<()> {
        self.0.set_len(size)
    }
    fn sync_all(&mut self) -> io::Result<()> {
        self.0.sync_all()
    }
}

impl Read for StdFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for StdFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl Seek for StdFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}

#[derive(Clone)]
pub struct StdStorage;

impl Storage for StdStorage {
    type File = StdFile;

    fn open(&self, path: &Path) -> io::Result<Self::File> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        Ok(StdFile(file))
    }

    fn create(&self, path: &Path) -> io::Result<Self::File> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        Ok(StdFile(file))
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<(PathBuf, SystemTime)>> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            entries.push((
                entry.path(),
                meta.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            ));
        }
        Ok(entries)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        std::fs::create_dir_all(path)
    }

    fn lock_exclusive(&self, path: &Path) -> io::Result<FileLock> {
        let lock_path = path.with_extension("lock");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_path)?;
        
        // Use flock on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            let result = unsafe { libc::flock(fd, libc::LOCK_EX) };
            if result != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        
        // For Windows, we'd use LockFileEx
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::Storage::FileSystem::LockFileEx;
            use windows_sys::Win32::Storage::FileSystem::LOCKFILE_EXCLUSIVE_LOCK;
            
            let handle = file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
            let result = unsafe { 
                LockFileEx(
                    handle,
                    LOCKFILE_EXCLUSIVE_LOCK,
                    0,
                    0xFFFFFFFF,
                    0xFFFFFFFF,
                    std::ptr::null_mut(),
                )
            };
            if result == 0 {
                return Err(io::Error::last_os_error());
            }
        }
        
        Ok(FileLock::new(file))
    }

    fn try_lock_exclusive(&self, path: &Path) -> io::Result<Option<FileLock>> {
        let lock_path = path.with_extension("lock");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_path)?;
        
        // Use non-blocking flock on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            let result = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
            if result != 0 {
                let error = io::Error::last_os_error();
                if error.kind() == io::ErrorKind::WouldBlock {
                    return Ok(None);
                }
                return Err(error);
            }
        }
        
        // For Windows, we'd use LockFileEx with non-blocking flag
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::Storage::FileSystem::LockFileEx;
            use windows_sys::Win32::Storage::FileSystem::LOCKFILE_EXCLUSIVE_LOCK;
            
            let handle = file.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
            let result = unsafe { 
                LockFileEx(
                    handle,
                    LOCKFILE_EXCLUSIVE_LOCK,
                    0,
                    0xFFFFFFFF,
                    0xFFFFFFFF,
                    std::ptr::null_mut(),
                )
            };
            if result == 0 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() == Some(33) { // ERROR_LOCK_VIOLATION
                    return Ok(None);
                }
                return Err(error);
            }
        }
        
        Ok(Some(FileLock::new(file)))
    }
}

// --- In-Memory Implementation (WASM/Test) ---

// We need an interior mutability wrapper that can be cloned (for multiple file handles)
// but specific file handles need their own cursor.

struct MemFileData {
    content: Vec<u8>,
    mtime: SystemTime,
}

#[derive(Clone)]
struct MemFs {
    files: Arc<Mutex<HashMap<PathBuf, Arc<Mutex<MemFileData>>>>>,
}

pub struct MemFile {
    inner: Arc<Mutex<MemFileData>>,
    pos: u64,
}

impl FileHandle for MemFile {
    fn set_len(&mut self, size: u64) -> io::Result<()> {
        let mut data = self.inner.lock().unwrap();
        data.content.resize(size as usize, 0);
        Ok(())
    }

    fn sync_all(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for MemFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let data = self.inner.lock().unwrap();
        let current_len = data.content.len() as u64;

        if self.pos >= current_len {
            return Ok(0);
        }

        let available = current_len - self.pos;
        let to_read = std::cmp::min(buf.len() as u64, available) as usize;

        buf[..to_read]
            .copy_from_slice(&data.content[self.pos as usize..(self.pos as usize + to_read)]);
        self.pos += to_read as u64;
        Ok(to_read)
    }
}

impl Write for MemFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut data = self.inner.lock().unwrap();

        let end_pos = self.pos + buf.len() as u64;
        if end_pos > data.content.len() as u64 {
            data.content.resize(end_pos as usize, 0);
        }

        data.content[self.pos as usize..end_pos as usize].copy_from_slice(buf);
        self.pos += buf.len() as u64;
        data.mtime = SystemTime::now();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for MemFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let data = self.inner.lock().unwrap();
        let len = data.content.len() as u64;

        let new_pos = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::End(p) => {
                if p < 0 {
                    // simple check
                    if p.unsigned_abs() > len {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Seek before start",
                        ));
                    }
                    len - p.unsigned_abs()
                } else {
                    len + (p as u64)
                }
            }
            SeekFrom::Current(p) => {
                if p < 0 {
                    let abs_p = p.unsigned_abs();
                    if abs_p > self.pos {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "Seek before start",
                        ));
                    }
                    self.pos - abs_p
                } else {
                    self.pos + (p as u64)
                }
            }
        };

        self.pos = new_pos;
        Ok(new_pos)
    }
}

#[derive(Clone)]
pub struct MemoryStorage {
    fs: MemFs,
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage {
            fs: MemFs {
                files: Arc::new(Mutex::new(HashMap::new())),
            },
        }
    }
}

impl Storage for MemoryStorage {
    type File = MemFile;

    fn open(&self, path: &Path) -> io::Result<Self::File> {
        let files = self.fs.files.lock().unwrap();
        if let Some(inner) = files.get(path) {
            Ok(MemFile {
                inner: inner.clone(),
                pos: 0,
            })
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }
    }

    fn create(&self, path: &Path) -> io::Result<Self::File> {
        let mut files = self.fs.files.lock().unwrap();
        let inner = Arc::new(Mutex::new(MemFileData {
            content: Vec::new(),
            mtime: SystemTime::now(),
        }));
        files.insert(path.to_path_buf(), inner.clone());
        Ok(MemFile { inner, pos: 0 })
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        let mut files = self.fs.files.lock().unwrap();
        files.remove(path);
        Ok(())
    }

    fn read_dir(&self, _path: &Path) -> io::Result<Vec<(PathBuf, SystemTime)>> {
        // Simple linear scan of all files (assuming flat or handling prefix)
        // For simplicity in this iteration, returning all files
        let files = self.fs.files.lock().unwrap();
        let mut entries = Vec::new();
        for (p, data) in files.iter() {
            let guard = data.lock().unwrap();
            entries.push((p.clone(), guard.mtime));
        }
        Ok(entries)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        let mut files = self.fs.files.lock().unwrap();
        if let Some(data) = files.remove(from) {
            files.insert(to.to_path_buf(), data);
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }
    }

    fn exists(&self, path: &Path) -> bool {
        let files = self.fs.files.lock().unwrap();
        files.contains_key(path)
    }

    fn create_dir_all(&self, _path: &Path) -> io::Result<()> {
        Ok(()) // "Folders" are implicit in mem fs
    }

    fn lock_exclusive(&self, path: &Path) -> io::Result<FileLock> {
        // For memory storage, we can't use real file locks
        // Create a dummy lock file in memory
        let lock_path = path.with_extension("lock");
        let _file = self.create(&lock_path)?;
        let _file = self.open(&lock_path)?;
        Ok(FileLock::new(std::fs::File::create("/dev/null").unwrap()))
    }

    fn try_lock_exclusive(&self, path: &Path) -> io::Result<Option<FileLock>> {
        // For memory storage, check if lock file exists
        let lock_path = path.with_extension("lock");
        if self.exists(&lock_path) {
            Ok(None)
        } else {
            self.lock_exclusive(path).map(Some)
        }
    }
}
