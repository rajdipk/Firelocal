#![allow(clippy::missing_safety_doc)]

use crate::FireLocal;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

/// # Safety
///
/// All FFI functions in this module are unsafe because they:
/// - Dereference raw pointers passed from C code
/// - Assume valid UTF-8 strings for C string parameters
/// - Return raw pointers that must be properly freed by the caller
///
/// Callers must ensure:
/// - All pointer arguments are non-null (unless explicitly allowed)
/// - All string pointers point to valid, null-terminated UTF-8 strings
/// - Returned strings are freed using the provided firelocal_free_string function
/// - Database and batch pointers are not used after being freed
#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_open(path: *const c_char) -> *mut FireLocal {
    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match FireLocal::new(path_str) {
        Ok(db) => Box::into_raw(Box::new(db)),
        Err(_) => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_destroy(db: *mut FireLocal) {
    if !db.is_null() {
        unsafe {
            drop(Box::from_raw(db));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_load_rules(db: *mut FireLocal, rules: *const c_char) -> i32 {
    let db = unsafe {
        if db.is_null() {
            return -1;
        }
        &mut *db
    };

    let rules_str = unsafe { CStr::from_ptr(rules) }.to_string_lossy();
    if db.load_rules(&rules_str).is_ok() {
        return 0;
    }
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_put_resource(
    db: *mut FireLocal,
    key: *const c_char,
    val: *const c_char,
) -> i32 {
    let db = unsafe {
        if db.is_null() {
            return -1;
        }
        &mut *db
    };

    let key_str = unsafe { CStr::from_ptr(key) }
        .to_string_lossy()
        .into_owned();
    let val_bytes = unsafe { CStr::from_ptr(val) }.to_bytes().to_vec();

    if db.put(key_str, val_bytes).is_ok() {
        return 0;
    }
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_get_resource(
    db: *mut FireLocal,
    key: *const c_char,
) -> *mut c_char {
    let db = unsafe {
        if db.is_null() {
            return std::ptr::null_mut();
        }
        &*db
    };

    let key_str = unsafe { CStr::from_ptr(key) }.to_string_lossy();

    if let Ok(Some(val)) = db.get(&key_str) {
        if let Ok(s) = std::str::from_utf8(&val) {
            if let Ok(c_str) = CString::new(s) {
                return c_str.into_raw();
            }
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_delete(db: *mut FireLocal, key: *const c_char) -> i32 {
    let db = unsafe {
        if db.is_null() {
            return -1;
        }
        &mut *db
    };

    let key_str = unsafe { CStr::from_ptr(key) }
        .to_string_lossy()
        .into_owned();

    if db.delete(key_str).is_ok() {
        return 0;
    }
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_batch_new(
    db: *mut FireLocal,
) -> *mut crate::transaction::WriteBatch {
    let db = unsafe {
        if db.is_null() {
            return std::ptr::null_mut();
        }
        &*db
    };

    let batch = db.batch();
    Box::into_raw(Box::new(batch))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_batch_set(
    batch: *mut crate::transaction::WriteBatch,
    key: *const c_char,
    val: *const c_char,
) -> i32 {
    let batch = unsafe {
        if batch.is_null() {
            return -1;
        }
        &mut *batch
    };

    let path_str = unsafe { CStr::from_ptr(key) }
        .to_string_lossy()
        .into_owned();
    let data_bytes = unsafe { CStr::from_ptr(val) }.to_bytes().to_vec();

    batch.set(path_str, data_bytes);
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_batch_update(
    batch: *mut crate::transaction::WriteBatch,
    path: *const c_char,
    data: *const c_char,
) -> i32 {
    let batch = unsafe {
        if batch.is_null() {
            return -1;
        }
        &mut *batch
    };

    let path_str = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .into_owned();
    let data_bytes = unsafe { CStr::from_ptr(data) }.to_bytes().to_vec();

    batch.update(path_str, data_bytes);
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_batch_delete(
    batch: *mut crate::transaction::WriteBatch,
    path: *const c_char,
) -> i32 {
    let batch = unsafe {
        if batch.is_null() {
            return -1;
        }
        &mut *batch
    };

    let path_str = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .into_owned();

    batch.delete(path_str);
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_batch_commit(
    db: *mut FireLocal,
    batch: *mut crate::transaction::WriteBatch,
) -> i32 {
    let db = unsafe {
        if db.is_null() {
            return -1;
        }
        &mut *db
    };

    let batch = unsafe {
        if batch.is_null() {
            return -1;
        }
        &*batch
    };

    if db.commit_batch(batch).is_ok() {
        return 0;
    }
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_batch_free(batch: *mut crate::transaction::WriteBatch) {
    if !batch.is_null() {
        unsafe {
            drop(Box::from_raw(batch));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_compact(db: *mut FireLocal) -> *mut c_char {
    let db = unsafe {
        if db.is_null() {
            return std::ptr::null_mut();
        }
        &*db
    };

    if let Ok(stats) = db.compact() {
        // Return JSON string with stats
        let json = format!(
            r#"{{"files_before":{},"files_after":{},"entries_before":{},"entries_after":{},"tombstones_removed":{},"size_before":{},"size_after":{}}}"#,
            stats.files_before,
            stats.files_after,
            stats.entries_before,
            stats.entries_after,
            stats.tombstones_removed,
            stats.size_before,
            stats.size_after
        );
        if let Ok(c_str) = CString::new(json) {
            return c_str.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn firelocal_flush(db: *mut FireLocal) -> i32 {
    let db = unsafe {
        if db.is_null() {
            return -1;
        }
        &mut *db
    };

    if db.flush().is_ok() {
        return 0;
    }
    -1
}
