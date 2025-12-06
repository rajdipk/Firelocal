using System;
using System.Collections.Generic;

namespace FireLocal
{
    /// <summary>
    /// Write batch for atomic multi-document operations
    /// </summary>
    public class WriteBatch
    {
        private readonly FireLocalDb _db;
        internal List<BatchOperation> Operations { get; } = new();

        internal WriteBatch(FireLocalDb db)
        {
            _db = db;
        }

        /// <summary>
        /// Add a set operation to the batch
        /// </summary>
        /// <param name="path">Document path</param>
        /// <param name="data">Document data</param>
        /// <returns>This batch for chaining</returns>
        public WriteBatch Set(string path, Dictionary<string, object> data)
        {
            Operations.Add(new BatchOperation
            {
                Type = OperationType.Set,
                Path = path,
                Data = data
            });
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
            Operations.Add(new BatchOperation
            {
                Type = OperationType.Update,
                Path = path,
                Data = data
            });
            return this;
        }

        /// <summary>
        /// Add a delete operation to the batch
        /// </summary>
        /// <param name="path">Document path</param>
        /// <returns>This batch for chaining</returns>
        public WriteBatch Delete(string path)
        {
            Operations.Add(new BatchOperation
            {
                Type = OperationType.Delete,
                Path = path
            });
            return this;
        }

        /// <summary>
        /// Commit all operations atomically
        /// </summary>
        public void Commit()
        {
            _db.CommitBatch(this);
        }
    }

    internal enum OperationType
    {
        Set,
        Update,
        Delete
    }

    internal class BatchOperation
    {
        public OperationType Type { get; set; }
        public string Path { get; set; } = string.Empty;
        public Dictionary<string, object>? Data { get; set; }
    }
}
