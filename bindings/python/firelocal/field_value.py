"""
FieldValue helpers for special operations
"""

import json
import time
from typing import Any, List


def server_timestamp() -> dict:
    """
    Get current server timestamp
    
    Returns:
        FieldValue for server timestamp
    """
    return {
        "_firelocal_field_value": "serverTimestamp",
        "value": int(time.time() * 1000),  # milliseconds
    }


def increment(n: int) -> dict:
    """
    Increment a numeric field
    
    Args:
        n: Amount to increment by
        
    Returns:
        FieldValue for increment operation
    """
    return {
        "_firelocal_field_value": "increment",
        "value": n,
    }


def array_union(elements: List[Any]) -> dict:
    """
    Add elements to an array (unique)
    
    Args:
        elements: Elements to add
        
    Returns:
        FieldValue for arrayUnion operation
    """
    return {
        "_firelocal_field_value": "arrayUnion",
        "value": elements,
    }


def array_remove(elements: List[Any]) -> dict:
    """
    Remove elements from an array
    
    Args:
        elements: Elements to remove
        
    Returns:
        FieldValue for arrayRemove operation
    """
    return {
        "_firelocal_field_value": "arrayRemove",
        "value": elements,
    }


def delete_field() -> dict:
    """
    Delete a field from a document
    
    Returns:
        FieldValue for delete operation
    """
    return {
        "_firelocal_field_value": "delete",
    }
