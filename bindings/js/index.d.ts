/**
 * FireLocal - Offline-first database with Firestore API compatibility
 */

/**
 * Compaction statistics
 */
export interface CompactionStats {
  files_before: number;
  files_after: number;
  entries_before: number;
  entries_after: number;
  tombstones_removed: number;
  size_before: number;
  size_after: number;
}

/**
 * Write batch for atomic operations
 */
export class WriteBatch {
  /**
   * Add a set operation to the batch
   * @param path - Document path
   * @param data - Document data as JSON string
   * @returns This WriteBatch instance for chaining
   */
  set(path: string, data: string): WriteBatch;

  /**
   * Add an update operation to the batch
   * @param path - Document path
   * @param data - Document data as JSON string
   * @returns This WriteBatch instance for chaining
   */
  update(path: string, data: string): WriteBatch;

  /**
   * Add a delete operation to the batch
   * @param path - Document path
   * @returns This WriteBatch instance for chaining
   */
  delete(path: string): WriteBatch;

  /**
   * Commit the batch atomically
   * All operations in the batch will be committed together.
   * If any operation fails, the entire batch will be rolled back.
   */
  commit(): void;
}

/**
 * FireLocal database instance
 * 
 * Example:
 * ```javascript
 * const db = new FireLocal('./data');
 * db.put('users/alice', JSON.stringify({ name: 'Alice', age: 30 }));
 * const user = JSON.parse(db.get('users/alice'));
 * ```
 */
export class FireLocal {
  /**
   * Create a new database instance
   * @param path - Path to the database directory
   */
  constructor(path: string);

  /**
   * Load security rules
   * @param rules - Firestore security rules string
   * @throws Error if rules are invalid
   */
  loadRules(rules: string): void;

  /**
   * Write a document
   * @param key - Document path (e.g., "users/alice")
   * @param value - Document data as JSON string
   * @throws Error if write fails or rules deny access
   */
  put(key: string, value: string): void;

  /**
   * Read a document
   * @param key - Document path
   * @returns Document data as JSON string, or null if not found
   * @throws Error if read fails or rules deny access
   */
  get(key: string): string | null;

  /**
   * Delete a document
   * @param key - Document path
   * @throws Error if delete fails or rules deny access
   */
  delete(key: string): void;

  /**
   * Create a new write batch
   * @returns WriteBatch instance
   */
  batch(): WriteBatch;

  /**
   * Run compaction to merge SST files and remove tombstones
   * @returns CompactionStats with before/after metrics
   * @throws Error if compaction fails
   */
  compact(): CompactionStats;

  /**
   * Flush memtable to SST file
   * @throws Error if flush fails
   */
  flush(): void;

  /**
   * Close the database
   */
  close(): void;
}

export default FireLocal;
