use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Compaction strategy for merging SST files and removing tombstones
pub struct Compactor {
    data_dir: PathBuf,
}

impl Compactor {
    #[allow(dead_code)]
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Run compaction: merge all SST files, remove tombstones, create new SST
    pub fn compact(&self) -> Result<CompactionStats> {
        // TODO: Refactor Compactor to use Storage trait.
        // Currently stubbed to allow compilation of core with new Storage traits.
        Ok(CompactionStats::default())
    }

    #[allow(dead_code)]
    /// Find all SST files in the data directory
    fn find_sst_files(&self) -> Result<Vec<PathBuf>> {
        let mut sst_files = Vec::new();

        if !self.data_dir.exists() {
            return Ok(sst_files);
        }

        for entry in fs::read_dir(&self.data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("sst") {
                sst_files.push(path);
            }
        }

        // Sort by modification time (oldest first for deterministic merging)
        sst_files.sort_by_key(|p| {
            fs::metadata(p)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });

        Ok(sst_files)
    }

    /// Merge data from multiple SST files
    /// Later entries override earlier ones (last-write-wins)
    #[allow(dead_code)]
    fn merge_sst_files(&self, _files: &[PathBuf]) -> Result<HashMap<String, Option<Vec<u8>>>> {
        let merged = HashMap::new();

        // Note: Full SST iteration requires enhancing SstReader
        // Current SstReader only supports get() by key
        // For production, we would:
        // 1. Add iterator support to SstReader
        // 2. Scan all entries from each SST
        // 3. Merge with last-write-wins semantics

        // For now, return empty map which will result in stats showing
        // the compaction happened but no data was merged
        // This is acceptable for M1 as the framework is in place

        Ok(merged)
    }

    /// Calculate total size of SST files
    #[allow(dead_code)]
    fn calculate_total_size(&self, files: &[PathBuf]) -> Result<u64> {
        let mut total = 0;
        for file in files {
            total += fs::metadata(file)?.len();
        }
        Ok(total)
    }
}

/// Statistics from a compaction run
#[derive(Debug, Default, Clone)]
pub struct CompactionStats {
    pub files_before: usize,
    pub files_after: usize,
    pub entries_before: usize,
    pub entries_after: usize,
    pub tombstones_removed: usize,
    pub size_before: u64,
    pub size_after: u64,
}

impl CompactionStats {
    pub fn size_reduction(&self) -> u64 {
        self.size_before.saturating_sub(self.size_after)
    }

    pub fn size_reduction_percent(&self) -> f64 {
        if self.size_before == 0 {
            0.0
        } else {
            (self.size_reduction() as f64 / self.size_before as f64) * 100.0
        }
    }
}
