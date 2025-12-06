using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Collections.Generic;

namespace FireLocal
{
    /// <summary>
    /// FireLocal database instance for offline-first data storage
    /// </summary>
    public class FireLocalDb : IDisposable
    {
        private IntPtr _handle;
        private bool _disposed = false;

        /// <summary>
        /// Create a new FireLocal database instance
        /// </summary>
        /// <param name="path">Directory path for database storage</param>
        public FireLocalDb(string path)
        {
            // Note: Actual P/Invoke implementation would require C wrapper functions
            // For now, this is a placeholder showing the intended API
            _handle = IntPtr.Zero;
        }

        /// <summary>
        /// Write a document to the database
        /// </summary>
        /// <param name="key">Document path (e.g., "users/alice")</param>
        /// <param name="value">Document data as dictionary</param>
        public void Put(string key, Dictionary<string, object> value)
        {
            ThrowIfDisposed();
            var json = JsonSerializer.Serialize(value);
            
            // P/Invoke call would go here
            // NativeMethods.firelocal_put(_handle, key, json);
        }

        /// <summary>
        /// Read a document from the database
        /// </summary>
        /// <param name="key">Document path</param>
        /// <returns>Document data or null if not found</returns>
        public Dictionary<string, object>? Get(string key)
        {
            ThrowIfDisposed();
            
            // P/Invoke call would go here
            // var jsonPtr = NativeMethods.firelocal_get(_handle, key);
            // if (jsonPtr == IntPtr.Zero) return null;
            // var json = Marshal.PtrToStringUTF8(jsonPtr);
            // return JsonSerializer.Deserialize<Dictionary<string, object>>(json);
            
            return null;
        }

        /// <summary>
        /// Delete a document from the database
        /// </summary>
        /// <param name="key">Document path</param>
        public void Delete(string key)
        {
            ThrowIfDisposed();
            
            // P/Invoke call would go here
            // NativeMethods.firelocal_delete(_handle, key);
        }

        /// <summary>
        /// Create a new write batch for atomic operations
        /// </summary>
        /// <returns>WriteBatch instance</returns>
        public WriteBatch Batch()
        {
            ThrowIfDisposed();
            return new WriteBatch(this);
        }

        /// <summary>
        /// Commit a write batch atomically
        /// </summary>
        /// <param name="batch">The batch to commit</param>
        public void CommitBatch(WriteBatch batch)
        {
            ThrowIfDisposed();
            
            // P/Invoke call would go here to commit all operations
            foreach (var op in batch.Operations)
            {
                // Process each operation
            }
        }

        /// <summary>
        /// Run compaction to merge SST files and remove tombstones
        /// </summary>
        /// <returns>Compaction statistics</returns>
        public CompactionStats Compact()
        {
            ThrowIfDisposed();
            
            // P/Invoke call would go here
            return new CompactionStats
            {
                FilesBefore = 0,
                FilesAfter = 0,
                EntriesBefore = 0,
                EntriesAfter = 0,
                TombstonesRemoved = 0,
                SizeBefore = 0,
                SizeAfter = 0
            };
        }

        /// <summary>
        /// Flush memtable to SST file
        /// </summary>
        public void Flush()
        {
            ThrowIfDisposed();
            
            // P/Invoke call would go here
            // NativeMethods.firelocal_flush(_handle);
        }

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(FireLocalDb));
        }

        public void Dispose()
        {
            if (!_disposed)
            {
                if (_handle != IntPtr.Zero)
                {
                    // P/Invoke cleanup would go here
                    // NativeMethods.firelocal_free(_handle);
                    _handle = IntPtr.Zero;
                }
                _disposed = true;
            }
            GC.SuppressFinalize(this);
        }

        ~FireLocalDb()
        {
            Dispose();
        }
    }

    // Native methods would be defined here
    // internal static class NativeMethods
    // {
    //     [DllImport("firelocal_core", CallingConvention = CallingConvention.Cdecl)]
    //     internal static extern IntPtr firelocal_new(string path);
    //     
    //     [DllImport("firelocal_core", CallingConvention = CallingConvention.Cdecl)]
    //     internal static extern void firelocal_put(IntPtr handle, string key, string value);
    //     
    //     // ... more P/Invoke declarations
    // }
}
