namespace FireLocal
{
    /// <summary>
    /// Statistics from a compaction run
    /// </summary>
    public class CompactionStats
    {
        /// <summary>
        /// Number of SST files before compaction
        /// </summary>
        public int FilesBefore { get; set; }

        /// <summary>
        /// Number of SST files after compaction
        /// </summary>
        public int FilesAfter { get; set; }

        /// <summary>
        /// Number of entries before compaction
        /// </summary>
        public int EntriesBefore { get; set; }

        /// <summary>
        /// Number of entries after compaction
        /// </summary>
        public int EntriesAfter { get; set; }

        /// <summary>
        /// Number of tombstones removed
        /// </summary>
        public int TombstonesRemoved { get; set; }

        /// <summary>
        /// Total size in bytes before compaction
        /// </summary>
        public long SizeBefore { get; set; }

        /// <summary>
        /// Total size in bytes after compaction
        /// </summary>
        public long SizeAfter { get; set; }

        /// <summary>
        /// Bytes saved by compaction
        /// </summary>
        public long SizeReduction => Math.Max(0, SizeBefore - SizeAfter);

        /// <summary>
        /// Percentage of space saved
        /// </summary>
        public double SizeReductionPercent =>
            SizeBefore == 0 ? 0.0 : (SizeReduction / (double)SizeBefore) * 100.0;

        public override string ToString()
        {
            return $"CompactionStats(files: {FilesBefore}→{FilesAfter}, " +
                   $"tombstones: {TombstonesRemoved}, " +
                   $"reduction: {SizeReductionPercent:F1}%)";
        }
    }
}
