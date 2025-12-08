using System;
using System.Collections.Generic;
using System.Text.Json;

namespace FireLocal
{
    /// <summary>
    /// Write batch for atomic multi-document operations
    /// </summary>
    public class WriteBatch : IDisposable
    {
        private readonly FireLocalDb _db;
        private readonly IntPtr _dbHandle;
        private IntPtr _handle;
        private bool _disposed = false;

        internal WriteBatch(FireLocalDb db, IntPtr dbHandle)
        {
            _db = db;
            _dbHandle = dbHandle;
            _handle = NativeMethods.firelocal_batch_new(dbHandle);
            if (_handle == IntPtr.Zero)
            {
                throw new Exception("Failed to create batch");
            }
        }

        /// <summary>
        /// Add a set operation to the batch
        /// </summary>
        /// <param name="path">Document path</param>
        /// <param name="data">Document data</param>
        /// <returns>This batch for chaining</returns>
        public WriteBatch Set(string path, Dictionary<string, object> data)
        {
            ThrowIfDisposed();
            var json = JsonSerializer.Serialize(data);
            int result = NativeMethods.firelocal_batch_set(_handle, path, json);
            if (result != 0)
            {
                throw new Exception($"Failed to add set operation: {path}");
            }
            return this;
        }

        /// <summary>
        /// Add an update operation to the batch
        /// </summary>
        /// <param name="path">Document path</param>
        /// <param name="data">Fields to update</param>
        /// <returns>This batch for chaining</returns>
        public WriteBatch Update(string path, Dictionary<string, object> data)
        {
            ThrowIfDisposed();
            var json = JsonSerializer.Serialize(data);
            int result = NativeMethods.firelocal_batch_update(_handle, path, json);
            if (result != 0)
            {
                throw new Exception($"Failed to add update operation: {path}");
            }
            return this;
        }

        /// <summary>
        /// Add a delete operation to the batch
        /// </summary>
        /// <param name="path">Document path</param>
        /// <returns>This batch for chaining</returns>
        public WriteBatch Delete(string path)
        {
            ThrowIfDisposed();
            int result = NativeMethods.firelocal_batch_delete(_handle, path);
            if (result != 0)
            {
                throw new Exception($"Failed to add delete operation: {path}");
            }
            return this;
        }

        /// <summary>
        /// Commit all operations atomically
        /// </summary>
        public void Commit()
        {
            ThrowIfDisposed();
            int result = NativeMethods.firelocal_batch_commit(_dbHandle, _handle);
            if (result != 0)
            {
                throw new Exception("Failed to commit batch");
            }
        }

        private void ThrowIfDisposed()
        {
            if (_disposed)
                throw new ObjectDisposedException(nameof(WriteBatch));
        }

        public void Dispose()
        {
            if (!_disposed)
            {
                if (_handle != IntPtr.Zero)
                {
                    NativeMethods.firelocal_batch_free(_handle);
                    _handle = IntPtr.Zero;
                }
                _disposed = true;
            }
            GC.SuppressFinalize(this);
        }

        ~WriteBatch()
        {
            Dispose();
        }
    }
}
