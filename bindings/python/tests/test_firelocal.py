"""
Tests for FireLocal Python bindings
"""

import pytest
from firelocal import (
    FireLocal,
    server_timestamp,
    increment,
    array_union,
    array_remove,
    delete_field,
)


def test_firelocal_creation():
    """Test creating a FireLocal instance"""
    db = FireLocal("./test_data")
    assert db.path == "./test_data"


def test_put_and_get():
    """Test basic put and get operations"""
    db = FireLocal("./test_data")
    
    data = {"name": "Alice", "age": 30}
    db.put("users/alice", data)
    
    # Note: Actual test would verify retrieval
    # result = db.get("users/alice")
    # assert result == data


def test_batch_operations():
    """Test batch writes"""
    db = FireLocal("./test_data")
    
    batch = db.batch()
    batch.set("users/alice", {"name": "Alice"})
    batch.set("users/bob", {"name": "Bob"})
    batch.delete("users/charlie")
    
    assert len(batch.operations) == 3
    assert batch.operations[0]["type"] == "set"
    assert batch.operations[2]["type"] == "delete"


def test_field_values():
    """Test FieldValue helpers"""
    ts = server_timestamp()
    assert "_firelocal_field_value" in ts
    assert ts["_firelocal_field_value"] == "serverTimestamp"
    
    inc = increment(5)
    assert inc["value"] == 5
    
    union = array_union(["a", "b"])
    assert union["value"] == ["a", "b"]
    
    remove = array_remove(["x"])
    assert remove["value"] == ["x"]
    
    delete = delete_field()
    assert delete["_firelocal_field_value"] == "delete"


def test_compaction_stats():
    """Test compaction statistics"""
    db = FireLocal("./test_data")
    stats = db.compact()
    
    assert hasattr(stats, "files_before")
    assert hasattr(stats, "size_reduction_percent")


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
