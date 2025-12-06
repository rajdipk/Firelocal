"""
FireLocal Python Bindings

Offline-first database with Firestore API compatibility
"""

from .core import FireLocal, WriteBatch, CompactionStats
from .field_value import (
    server_timestamp,
    increment,
    array_union,
    array_remove,
    delete_field,
)

__version__ = "0.1.0"
__all__ = [
    "FireLocal",
    "WriteBatch",
    "CompactionStats",
    "server_timestamp",
    "increment",
    "array_union",
    "array_remove",
    "delete_field",
]
