"""
Core FireLocal Python bindings using ctypes
"""

import ctypes
import json
import os
import platform
from pathlib import Path
from typing import Optional, Dict, Any, List


def _get_library_path():
    """Find the FireLocal core library"""
    system = platform.system()
    if system == "Windows":
        lib_name = "firelocal_core.dll"
    elif system == "Darwin":
        lib_name = "libfirelocal_core.dylib"
    else:
        lib_name = "libfirelocal_core.so"
    
    # Try to find in standard locations
    search_paths = [
        Path(__file__).parent.parent.parent.parent / "target" / "release",
        Path(__file__).parent.parent.parent.parent / "target" / "debug",
        Path("/usr/local/lib"),
        Path("/usr/lib"),
    ]
    
    for path in search_paths:
        lib_path = path / lib_name
        if lib_path.exists():
            return str(lib_path)
    
    raise FileNotFoundError(f"Could not find {lib_name}")


# Load the library
_lib = ctypes.CDLL(_get_library_path())


class FireLocal:
    """
    FireLocal database instance
    
    Example:
        >>> db = FireLocal("./data")
        >>> db.put("users/alice", {"name": "Alice", "age": 30})
        >>> doc = db.get("users/alice")
        >>> print(doc)
        {'name': 'Alice', 'age': 30}
    """
    
    def __init__(self, path: str):
        """
        Create a new FireLocal instance
        
        Args:
            path: Directory path for database storage
        """
        self.path = path
        # Note: Actual FFI implementation would require C wrapper functions
        # For now, this is a placeholder showing the intended API
        self._handle = None
    
    def put(self, key: str, value: Dict[str, Any]) -> None:
        """
        Write a document
        
        Args:
            key: Document path (e.g., "users/alice")
            value: Document data as dictionary
        """
        json_str = json.dumps(value)
        # FFI call would go here
        # _lib.firelocal_put(self._handle, key.encode(), json_str.encode())
        pass
    
    def get(self, key: str) -> Optional[Dict[str, Any]]:
        """
        Read a document
        
        Args:
            key: Document path
            
        Returns:
            Document data or None if not found
        """
        # FFI call would go here
        # result = _lib.firelocal_get(self._handle, key.encode())
        # if result:
        #     return json.loads(result.decode())
        return None
    
    def delete(self, key: str) -> None:
        """
        Delete a document
        
        Args:
            key: Document path
        """
        # FFI call would go here
        # _lib.firelocal_delete(self._handle, key.encode())
        pass
    
    def batch(self) -> 'WriteBatch':
        """
        Create a new write batch
        
        Returns:
            WriteBatch instance
        """
        return WriteBatch(self)
    
    def compact(self) -> 'CompactionStats':
        """
        Run compaction to merge SST files and remove tombstones
        
        Returns:
            CompactionStats with before/after metrics
        """
        # FFI call would go here
        return CompactionStats(
            files_before=0,
            files_after=0,
            entries_before=0,
            entries_after=0,
            tombstones_removed=0,
            size_before=0,
            size_after=0,
        )
    
    def flush(self) -> None:
        """Flush memtable to SST file"""
        # FFI call would go here
        pass


class WriteBatch:
    """
    Atomic write batch
    
    Example:
        >>> batch = db.batch()
        >>> batch.set("users/alice", {"name": "Alice"})
        >>> batch.set("users/bob", {"name": "Bob"})
        >>> batch.delete("users/charlie")
        >>> db.commit_batch(batch)
    """
    
    def __init__(self, db: FireLocal):
        self.db = db
        self.operations: List[Dict[str, Any]] = []
    
    def set(self, path: str, data: Dict[str, Any]) -> 'WriteBatch':
        """Add a set operation to the batch"""
        self.operations.append({
            "type": "set",
            "path": path,
            "data": data,
        })
        return self
    
    def update(self, path: str, data: Dict[str, Any]) -> 'WriteBatch':
        """Add an update operation to the batch"""
        self.operations.append({
            "type": "update",
            "path": path,
            "data": data,
        })
        return self
    
    def delete(self, path: str) -> 'WriteBatch':
        """Add a delete operation to the batch"""
        self.operations.append({
            "type": "delete",
            "path": path,
        })
        return self
    
    def commit(self) -> None:
        """Commit the batch atomically"""
        # FFI call would go here
        # _lib.firelocal_commit_batch(self.db._handle, ...)
        pass


class CompactionStats:
    """Statistics from a compaction run"""
    
    def __init__(self, files_before: int, files_after: int,
                 entries_before: int, entries_after: int,
                 tombstones_removed: int, size_before: int, size_after: int):
        self.files_before = files_before
        self.files_after = files_after
        self.entries_before = entries_before
        self.entries_after = entries_after
        self.tombstones_removed = tombstones_removed
        self.size_before = size_before
        self.size_after = size_after
    
    @property
    def size_reduction(self) -> int:
        """Bytes saved"""
        return max(0, self.size_before - self.size_after)
    
    @property
    def size_reduction_percent(self) -> float:
        """Percentage of space saved"""
        if self.size_before == 0:
            return 0.0
        return (self.size_reduction / self.size_before) * 100.0
    
    def __repr__(self):
        return (f"CompactionStats(files: {self.files_before}→{self.files_after}, "
                f"tombstones: {self.tombstones_removed}, "
                f"reduction: {self.size_reduction_percent:.1f}%)")
