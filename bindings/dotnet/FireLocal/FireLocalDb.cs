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
            _handle = NativeMethods.firelocal_open(path);
            if (_handle == IntPtr.Zero)
            {
                throw new Exception($"Failed to open database at {path}");
            }
        }

        /// <summary>
        /// Load security rules
        /// </summary>
        /// <param name="rules">Firestore security rules string</param>
        public void LoadRules(string rules)
        {
            ThrowIfDisposed();
            int result = NativeMethods.firelocal_load_rules(_handle, rules);
            if (result != 0)
            {
                throw new Exception("Failed to load rules");
            }
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
            int result = NativeMethods.firelocal_put_resource(_handle, key, json);
            if (result != 0)
            {
                throw new Exception($"Failed to put document: {key}");
            }
        }

        /// <summary>
        /// Read a document from the database
        /// </summary>
        /// <param name="key">Document path</param>
        /// <returns>Document data or null if not found</returns>
        public Dictionary<string, object>? Get(string key)
        {
            ThrowIfDisposed();
            IntPtr jsonPtr = NativeMethods.firelocal_get_resource(_handle, key);
            if (jsonPtr == IntPtr.Zero)
            {
                return null;
            }

            try
            {
                var json = Marshal.PtrToStringUTF8(jsonPtr);
                if (string.IsNullOrEmpty(json))
                {
                    return null;
                }
                return JsonSerializer.Deserialize<Dictionary<string, object>>(json);
            }
            finally
            {
                NativeMethods.firelocal_free_string(jsonPtr);
            }
        }

        /// <summary>
        /// Delete a document from the database
        /// </summary>
        /// <param name="key">Document path</param>
        public void Delete(string key)
        {
            ThrowIfDisposed();
            int result = NativeMethods.firelocal_delete(_handle, key);
            if (result != 0)
            {
                throw new Exception($"Failed to delete document: {key}");
            }
        }

        /// <summary>
        /// Create a new write batch for atomic operations
        /// </summary>
        /// <returns>WriteBatch instance</returns>
        public WriteBatch Batch()
        {
            ThrowIfDisposed();
            return new WriteBatch(this, _handle);
        }

        /// <summary>
        /// Run compaction to merge SST files and remove tombstones
        /// </summary>
        /// <returns>Compaction statistics</returns>
        public CompactionStats Compact()
        {
            ThrowIfDisposed();
            IntPtr jsonPtr = NativeMethods.firelocal_compact(_handle);
            if (jsonPtr == IntPtr.Zero)
            {
                throw new Exception("Compaction failed");
            }

            try
            {
                var json = Marshal.PtrToStringUTF8(jsonPtr);
                if (string.IsNullOrEmpty(json))
                {
                    throw new Exception("Invalid compaction result");
                }

                var data = JsonSerializer.Deserialize<Dictionary<string, object>>(json);
                if (data == null)
                {
                    throw new Exception("Failed to parse compaction stats");
                }

                return new CompactionStats
                {
                    FilesBefore = Convert.ToInt32(data["files_before"]),
                    FilesAfter = Convert.ToInt32(data["files_after"]),
                    EntriesBefore = Convert.ToInt32(data["entries_before"]),
                    EntriesAfter = Convert.ToInt32(data["entries_after"]),
                    TombstonesRemoved = Convert.ToInt32(data["tombstones_removed"]),
                    SizeBefore = Convert.ToInt64(data["size_before"]),
                    SizeAfter = Convert.ToInt64(data["size_after"])
                };
            }
            finally
            {
                NativeMethods.firelocal_free_string(jsonPtr);
            }
        }

        /// <summary>
        /// Flush memtable to SST file
        /// </summary>
        public void Flush()
        {
            ThrowIfDisposed();
            int result = NativeMethods.firelocal_flush(_handle);
            if (result != 0)
            {
                throw new Exception("Flush failed");
            }
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
                    NativeMethods.firelocal_destroy(_handle);
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

    // Native methods
    internal static class NativeMethods
    {
        private const string LibName = "firelocal_core";

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern IntPtr firelocal_open([MarshalAs(UnmanagedType.LPUTF8Str)] string path);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void firelocal_destroy(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern int firelocal_load_rules(IntPtr handle, [MarshalAs(UnmanagedType.LPUTF8Str)] string rules);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern int firelocal_put_resource(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string key,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string value);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern IntPtr firelocal_get_resource(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string key);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern int firelocal_delete(
            IntPtr handle,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string key);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void firelocal_free_string(IntPtr str);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr firelocal_batch_new(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern int firelocal_batch_set(
            IntPtr batch,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string data);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern int firelocal_batch_update(
            IntPtr batch,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string path,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string data);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)]
        internal static extern int firelocal_batch_delete(
            IntPtr batch,
            [MarshalAs(UnmanagedType.LPUTF8Str)] string path);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int firelocal_batch_commit(IntPtr dbHandle, IntPtr batchHandle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern void firelocal_batch_free(IntPtr batch);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern IntPtr firelocal_compact(IntPtr handle);

        [DllImport(LibName, CallingConvention = CallingConvention.Cdecl)]
        internal static extern int firelocal_flush(IntPtr handle);
    }
}
