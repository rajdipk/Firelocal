import os
import shutil
from firelocal import FireLocal

DB_PATH = "tmp_py_test_db"

def teardown_module():
    if os.path.exists(DB_PATH):
        shutil.rmtree(DB_PATH)

def test_lifecycle():
    if os.path.exists(DB_PATH):
        shutil.rmtree(DB_PATH)
        
    with FireLocal(DB_PATH) as db:
        # Load rules
        db.load_rules("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }")
        
        # Put
        data = {"foo": 123, "bar": "baz"}
        db.put("doc1", data)
        
        # Get
        res = db.get("doc1")
        assert res == data
        
        # Get missing
        res = db.get("missing")
        assert res is None

if __name__ == "__main__":
    test_lifecycle()
    print("Python binding test passed!")
