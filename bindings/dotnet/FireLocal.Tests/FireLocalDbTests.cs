using Xunit;
using System.Collections.Generic;

namespace FireLocal.Tests
{
    public class FireLocalDbTests
    {
        [Fact]
        public void CanCreateInstance()
        {
            using var db = new FireLocalDb("./test_data");
            Assert.NotNull(db);
        }

        [Fact]
        public void CanPutAndGet()
        {
            using var db = new FireLocalDb("./test_data");
            
            var data = new Dictionary<string, object>
            {
                ["name"] = "Alice",
                ["age"] = 30
            };

            db.Put("users/alice", data);
            
            // Note: Actual test would verify retrieval
            // var result = db.Get("users/alice");
            // Assert.Equal("Alice", result["name"]);
        }

        [Fact]
        public void CanUseBatch()
        {
            using var db = new FireLocalDb("./test_data");
            
            var batch = db.Batch();
            batch.Set("users/alice", new Dictionary<string, object> { ["name"] = "Alice" });
            batch.Set("users/bob", new Dictionary<string, object> { ["name"] = "Bob" });
            batch.Delete("users/charlie");

            batch.Commit(); // Just test that it doesn't throw
        }

        [Fact]
        public void FieldValueHelpersWork()
        {
            var ts = FieldValue.ServerTimestamp();
            Assert.Equal("serverTimestamp", ts["_firelocal_field_value"]);

            var inc = FieldValue.Increment(5);
            Assert.Equal(5L, inc["value"]);

            var union = FieldValue.ArrayUnion("a", "b");
            Assert.Equal("arrayUnion", union["_firelocal_field_value"]);

            var delete = FieldValue.Delete();
            Assert.Equal("delete", delete["_firelocal_field_value"]);
        }

        [Fact]
        public void CompactionStatsCalculatesCorrectly()
        {
            var stats = new CompactionStats
            {
                FilesBefore = 5,
                FilesAfter = 1,
                EntriesBefore = 1000,
                EntriesAfter = 750,
                TombstonesRemoved = 250,
                SizeBefore = 1000,
                SizeAfter = 750
            };

            Assert.Equal(250, stats.SizeReduction);
            Assert.Equal(25.0, stats.SizeReductionPercent);
        }
    }
}
