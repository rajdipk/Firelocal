using System;
using System.Collections.Generic;

namespace FireLocal
{
    /// <summary>
    /// FieldValue helpers for special Firestore-compatible operations
    /// </summary>
    public static class FieldValue
    {
        /// <summary>
        /// Get current server timestamp
        /// </summary>
        /// <returns>FieldValue for server timestamp</returns>
        public static Dictionary<string, object> ServerTimestamp()
        {
            return new Dictionary<string, object>
            {
                ["_firelocal_field_value"] = "serverTimestamp",
                ["value"] = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds()
            };
        }

        /// <summary>
        /// Increment a numeric field
        /// </summary>
        /// <param name="n">Amount to increment by</param>
        /// <returns>FieldValue for increment operation</returns>
        public static Dictionary<string, object> Increment(long n)
        {
            return new Dictionary<string, object>
            {
                ["_firelocal_field_value"] = "increment",
                ["value"] = n
            };
        }

        /// <summary>
        /// Add elements to an array (unique)
        /// </summary>
        /// <param name="elements">Elements to add</param>
        /// <returns>FieldValue for arrayUnion operation</returns>
        public static Dictionary<string, object> ArrayUnion(params object[] elements)
        {
            return new Dictionary<string, object>
            {
                ["_firelocal_field_value"] = "arrayUnion",
                ["value"] = elements
            };
        }

        /// <summary>
        /// Remove elements from an array
        /// </summary>
        /// <param name="elements">Elements to remove</param>
        /// <returns>FieldValue for arrayRemove operation</returns>
        public static Dictionary<string, object> ArrayRemove(params object[] elements)
        {
            return new Dictionary<string, object>
            {
                ["_firelocal_field_value"] = "arrayRemove",
                ["value"] = elements
            };
        }

        /// <summary>
        /// Delete a field from a document
        /// </summary>
        /// <returns>FieldValue for delete operation</returns>
        public static Dictionary<string, object> Delete()
        {
            return new Dictionary<string, object>
            {
                ["_firelocal_field_value"] = "delete"
            };
        }
    }
}
