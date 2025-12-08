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
try:
    _lib = ctypes.CDLL(_get_library_path())
    
    # Define function signatures
    _lib.firelocal_open.argtypes = [ctypes.c_char_p]
    _lib.firelocal_open.restype = ctypes.c_void_p
    
    _lib.firelocal_destroy.argtypes = [ctypes.c_void_p]
    _lib.firelocal_destroy.restype = None
    
    _lib.firelocal_load_rules.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    _lib.firelocal_load_rules.restype = ctypes.c_int
    
    _lib.firelocal_put_resource.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    _lib.firelocal_put_resource.restype = ctypes.c_int
    
    _lib.firelocal_get_resource.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    _lib.firelocal_get_resource.restype = ctypes.c_void_p
    
    _lib.firelocal_delete.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    _lib.firelocal_delete.restype = ctypes.c_int
    
    _lib.firelocal_free_string.argtypes = [ctypes.c_void_p]
    _lib.firelocal_free_string.restype = None
    
    _lib.firelocal_batch_new.argtypes = [ctypes.c_void_p]
    _lib.firelocal_batch_new.restype = ctypes.c_void_p
    
    _lib.firelocal_batch_set.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    _lib.firelocal_batch_set.restype = ctypes.c_int
    
    _lib.firelocal_batch_update.argtypes = [ctypes.c_void_p, ctypes.c_char_p, ctypes.c_char_p]
    _lib.firelocal_batch_update.restype = ctypes.c_int
    
    _lib.firelocal_batch_delete.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
    _lib.firelocal_batch_delete.restype = ctypes.c_int
    
    _lib.firelocal_batch_commit.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
    _lib.firelocal_batch_commit.restype = ctypes.c_int
    
    _lib.firelocal_batch_free.argtypes = [ctypes.c_void_p]
    _lib.firelocal_batch_free.restype = None
    
    _lib.firelocal_compact.argtypes = [ctypes.c_void_p]
    _lib.firelocal_compact.restype = ctypes.c_void_p
    
    _lib.firelocal_flush.argtypes = [ctypes.c_void_p]
    _lib.firelocal_flush.restype = ctypes.c_int
    
    _LIB_LOADED = True
except (OSError, FileNotFoundError) as e:
    _LIB_LOADED = False
    _lib = None
    print(f"Warning: Could not load FireLocal library: {e}")


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
        if not _LIB_LOADED:
            raise RuntimeError("FireLocal library not loaded")
            
        self.path = path
        self._handle = _lib.firelocal_open(path.encode('utf-8'))
        if not self._handle:
            raise RuntimeError(f"Failed to open database at {path}")
    
    def load_rules(self, rules: str) -> None:
        """Load security rules"""
        if _lib.firelocal_load_rules(self._handle, rules.encode('utf-8')) != 0:
            raise RuntimeError("Failed to load rules")
    
    def put(self, key: str, value: Dict[str, Any]) -> None:
        """
        Write a document
        
        Args:
            key: Document path (e.g., "users/alice")
            value: Document data as dictionary
        """
        json_str = json.dumps(value)
        result = _lib.firelocal_put_resource(
            self._handle,
            key.encode('utf-8'),
            json_str.encode('utf-8')
        )
        if result != 0:
            raise RuntimeError(f"Failed to put document: {key}")
    
    def get(self, key: str) -> Optional[Dict[str, Any]]:
        """
        Read a document
        
        Args:
            key: Document path
            
        Returns:
            Document data or None if not found
        """
        result_ptr = _lib.firelocal_get_resource(self._handle, key.encode('utf-8'))
        if not result_ptr:
            return None
        
        try:
            json_str = ctypes.c_char_p(result_ptr).value.decode('utf-8')
            return json.loads(json_str)
        finally:
            _lib.firelocal_free_string(result_ptr)
    
    def delete(self, key: str) -> None:
        """
        Delete a document
        
        Args:
            key: Document path
        """
        result = _lib.firelocal_delete(self._handle, key.encode('utf-8'))
        if result != 0:
            raise RuntimeError(f"Failed to delete document: {key}")
    
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
        result_ptr = _lib.firelocal_compact(self._handle)
        if not result_ptr:
            raise RuntimeError("Compaction failed")
        
        try:
            json_str = ctypes.c_char_p(result_ptr).value.decode('utf-8')
            data = json.loads(json_str)
            return CompactionStats(
                files_before=data['files_before'],
                files_after=data['files_after'],
                entries_before=data['entries_before'],
                entries_after=data['entries_after'],
                tombstones_removed=data['tombstones_removed'],
                size_before=data['size_before'],
                size_after=data['size_after'],
            )
        finally:
            _lib.firelocal_free_string(result_ptr)
    
    def flush(self) -> None:
        """Flush memtable to SST file"""
        result = _lib.firelocal_flush(self._handle)
        if result != 0:
            raise RuntimeError("Flush failed")
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
    
    def close(self):
        """Close the database and free resources"""
        if self._handle:
            _lib.firelocal_destroy(self._handle)
            self._handle = None
    
    def __del__(self):
        self.close()


class WriteBatch:
    """
    Atomic write batch
    
    Example:
        >>> batch = db.batch()
        >>> batch.set("users/alice", {"name": "Alice"})
        >>> batch.set("users/bob", {"name": "Bob"})
        >>> batch.delete("users/charlie")
        >>> batch.commit()
    """
    
    def __init__(self, db: FireLocal):
        self.db = db
        self._handle = _lib.firelocal_batch_new(db._handle)
        if not self._handle:
            raise RuntimeError("Failed to create batch")
    
    def set(self, path: str, data: Dict[str, Any]) -> 'WriteBatch':
        """Add a set operation to the batch"""
        json_str = json.dumps(data)
        result = _lib.firelocal_batch_set(
            self._handle,
            path.encode('utf-8'),
            json_str.encode('utf-8')
        )
        if result != 0:
            raise RuntimeError(f"Failed to add set operation: {path}")
        return self
    
    def update(self, path: str, data: Dict[str, Any]) -> 'WriteBatch':
        """Add an update operation to the batch"""
        json_str = json.dumps(data)
        result = _lib.firelocal_batch_update(
            self._handle,
            path.encode('utf-8'),
            json_str.encode('utf-8')
        )
        if result != 0:
            raise RuntimeError(f"Failed to add update operation: {path}")
        return self
    
    def delete(self, path: str) -> 'WriteBatch':
        """Add a delete operation to the batch"""
        result = _lib.firelocal_batch_delete(self._handle, path.encode('utf-8'))
        if result != 0:
            raise RuntimeError(f"Failed to add delete operation: {path}")
        return self
    
    def commit(self) -> None:
        """Commit the batch atomically"""
        result = _lib.firelocal_batch_commit(self.db._handle, self._handle)
        if result != 0:
            raise RuntimeError("Failed to commit batch")
    
    def __del__(self):
        if self._handle:
            _lib.firelocal_batch_free(self._handle)
            self._handle = None


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
