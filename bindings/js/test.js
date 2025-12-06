const { FireLocal } = require('./index.js');
const fs = require('fs');

const DB_PATH = 'tmp_js_test_db';
if (fs.existsSync(DB_PATH)) {
    fs.rmSync(DB_PATH, { recursive: true, force: true });
}

try {
    const db = new FireLocal(DB_PATH);
    db.loadRules("service cloud.firestore { match /databases/{database}/documents { match /{document=**} { allow read, write: if true; } } }");

    const key = "doc1";
    const value = JSON.stringify({ foo: "bar" });
    db.put(key, value);

    const val = db.get(key);
    console.log("Retrieved:", val);

    if (val !== value) {
        console.error("Test failed: mismatch");
        process.exit(1);
    }
    console.log("JS binding test passed!");
} catch (e) {
    console.error("Test failed with error:", e);
    process.exit(1);
} finally {
    if (fs.existsSync(DB_PATH)) {
        fs.rmSync(DB_PATH, { recursive: true, force: true });
    }
}
