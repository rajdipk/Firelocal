use crate::FireLocal;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn firelocal_open(path: *const c_char) -> *mut FireLocal {
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
pub extern "C" fn firelocal_destroy(db: *mut FireLocal) {
    if !db.is_null() {
        unsafe {
            drop(Box::from_raw(db));
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn firelocal_load_rules(db: *mut FireLocal, rules: *const c_char) -> i32 {
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
pub extern "C" fn firelocal_put_resource(
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
pub extern "C" fn firelocal_get_resource(db: *mut FireLocal, key: *const c_char) -> *mut c_char {
    let db = unsafe {
        if db.is_null() {
            return std::ptr::null_mut();
        }
        &*db
    };

    let key_str = unsafe { CStr::from_ptr(key) }.to_string_lossy();

    if let Some(val) = db.get(&key_str) {
        if let Ok(s) = std::str::from_utf8(&val) {
            if let Ok(c_str) = CString::new(s) {
                return c_str.into_raw();
            }
        }
    }
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "C" fn firelocal_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}
