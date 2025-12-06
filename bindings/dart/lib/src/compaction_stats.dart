/// Statistics from a compaction run
class CompactionStats {
  final int filesBefore;
  final int filesAfter;
  final int entriesBefore;
  final int entriesAfter;
  final int tombstonesRemoved;
  final int sizeBefore;
  final int sizeAfter;

  CompactionStats({
    required this.filesBefore,
    required this.filesAfter,
    required this.entriesBefore,
    required this.entriesAfter,
    required this.tombstonesRemoved,
    required this.sizeBefore,
    required this.sizeAfter,
  });

  /// Bytes saved
  int get sizeReduction => (sizeBefore - sizeAfter).clamp(0, sizeBefore);

  /// Percentage of space saved
  double get sizeReductionPercent {
    if (sizeBefore == 0) return 0.0;
    return (sizeReduction / sizeBefore) * 100.0;
  }

  @override
  String toString() {
    return 'CompactionStats(files: $filesBefore→$filesAfter, '
        'tombstones: $tombstonesRemoved, '
        'reduction: ${sizeReductionPercent.toStringAsFixed(1)}%)';
  }
}
