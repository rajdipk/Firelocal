use std::ffi::{CStr, CString};

use firelocal_core::ffi::{
    firelocal_destroy, firelocal_free_string, firelocal_get_resource, firelocal_load_rules,
    firelocal_open, firelocal_put_resource,
};

#[test]
fn test_ffi_lifecycle() {
    unsafe {
        let path = CString::new("tmp_ffi_test_db").unwrap();

        // 1. Open
        let db_ptr = firelocal_open(path.as_ptr());
        assert!(!db_ptr.is_null(), "Database pointer should not be null");

        // Load rules
        let rules = CString::new("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }").unwrap();
        let ret = firelocal_load_rules(db_ptr, rules.as_ptr());
        assert_eq!(ret, 0, "Load rules should return 0");

        // 2. Put
        let key = CString::new("doc1").unwrap();
        let val = CString::new(r#"{"foo":"bar"}"#).unwrap();
        let ret = firelocal_put_resource(db_ptr, key.as_ptr(), val.as_ptr());
        assert_eq!(ret, 0, "Put should return 0 on success");

        // 3. Get
        let res_ptr = firelocal_get_resource(db_ptr, key.as_ptr());
        assert!(!res_ptr.is_null(), "Get should return a pointer");

        let c_str = CStr::from_ptr(res_ptr);
        let s = c_str.to_str().expect("Should be valid UTF-8");
        assert_eq!(s, r#"{"foo":"bar"}"#);

        // 4. Free String
        firelocal_free_string(res_ptr);

        // 5. Destroy DB
        firelocal_destroy(db_ptr);

        // Cleanup filesystem
        let _ = std::fs::remove_dir_all("tmp_ffi_test_db");
    }
}
