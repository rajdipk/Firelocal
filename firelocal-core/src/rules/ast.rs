use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Ruleset {
    pub service_name: String,
    pub match_block: MatchBlock,
}

#[derive(Debug, Clone)]
pub struct MatchBlock {
    pub path_pattern: String, // e.g. "/databases/{database}/documents"
    pub sub_matches: Vec<MatchBlock>,
    pub allow_statements: Vec<AllowStatement>,
}

#[derive(Debug, Clone)]
pub struct AllowStatement {
    pub operations: Vec<String>,
    pub condition: String,
}

impl Ruleset {
    pub fn is_allowed(
        &self,
        path: &str,
        operation: &str,
        context: &HashMap<String, String>,
    ) -> bool {
        self.match_block.matches_recursive(path, operation, context)
    }
}

impl MatchBlock {
    // Returns true if this block or any sub-block allows the operation on the path
    pub fn matches_recursive(
        &self,
        remaining_path: &str,
        operation: &str,
        _context: &HashMap<String, String>,
    ) -> bool {
        // 1. Try to consume the current pattern from the remaining path
        if let Some(remainder) = self.consume_pattern(remaining_path) {
            // Match successful!

            // 2. If exact match (remainder empty or just /), check ALLOWS
            let is_exact = remainder.trim_matches('/').is_empty();
            if is_exact {
                for allow in &self.allow_statements {
                    if (allow.operations.contains(&operation.to_string())
                        || allow.operations.contains(&"match_all".to_string()))
                        && allow.condition.trim() == "true"
                    {
                        return true;
                    }
                }
            }

            // 3. Check sub-matches with the remainder
            for sub in &self.sub_matches {
                if sub.matches_recursive(remainder, operation, _context) {
                    return true;
                }
            }
        }

        false
    }

    fn consume_pattern<'a>(&self, path: &'a str) -> Option<&'a str> {
        let pattern_segments: Vec<&str> = self
            .path_pattern
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if path_segments.len() < pattern_segments.len() {
            return None;
        }

        // Check segments
        for (i, p_seg) in pattern_segments.iter().enumerate() {
            let doc_seg = path_segments[i];

            if p_seg.starts_with('{') && p_seg.ends_with('}') {
                if p_seg.contains("=**") {
                    // Recursive wildcard: Matches everything remaining.
                    // If this is the last pattern segment, we return "" as remainder (conceptually consumed all relevant for this block? Or matches rest?)
                    // In Firestore: match /{document=**} means document variable captures the REST of path.
                    // So we successfully match ALL of it.
                    // Remainder should be empty to trigger "exact match" allows?
                    // Yes. And we need to support sub-matches? Rarely used with **.
                    return Some("");
                }
                // Variable match: Matches any single segment
                continue;
            }

            if *p_seg != doc_seg {
                return None;
            }
        }

        // Reconstruct remainder
        // Skip the consumed segments.
        // We need to find where the consumed part ended in the original string?
        // Or simpler: rejoin the remaining segments.
        // But rejoining allocates. We want &str.
        // Let's approximate:
        // We matched `pattern_segments.len()` segments.
        // Total path segments available.
        // If we matched all, remainder is the rest.

        // Reconstructing from index is annoying with split.
        // Let's iterate original string splitting?

        let consumed_count = pattern_segments.len();

        // Find the byte offset of the Nth slash-separated segment end
        let _ = path; // usage

        // Simplification for M4: Reconstruct string and leak? No.
        // Return byte index?
        // Let's just use re-split assumption:
        // Pass the substring starting after the Nth non-empty segment.

        let mut current_matches = 0;
        let p_bytes = path.as_bytes();
        let mut i = 0;

        // Skip leading slashes
        while i < p_bytes.len() && p_bytes[i] == b'/' {
            i += 1;
        }

        let start_idx = i;

        if consumed_count == 0 {
            return Some(path);
        }

        while current_matches < consumed_count {
            if i >= p_bytes.len() {
                // If we ran out of string but matched segments, it means we consumed everything?
                // But we checked len earlier.
                break;
            }
            if p_bytes[i] == b'/' {
                current_matches += 1;
                while i < p_bytes.len() && p_bytes[i] == b'/' {
                    i += 1;
                } // skip multiple slashes
            } else {
                i += 1;
            }
        }

        // Check if we finished the last segment
        if current_matches < consumed_count {
            // We didn't find N separators.
            // That means the last segment goes to end of string?
            // Example: path "a/b", consume 2.
            // i goes to end. current_matches = 1 (sep after a).
            // separator count is segments - 1.
            // So if we consume N segments, we might pass N-1 separators.
            // Correct logic: Scan past N segments.
        }

        // Let's restart scan
        i = start_idx;
        for _ in 0..consumed_count {
            // Scan one segment
            while i < p_bytes.len() && p_bytes[i] != b'/' {
                i += 1;
            }
            // Consumed one segment.
            // Scan past separators
            while i < p_bytes.len() && p_bytes[i] == b'/' {
                i += 1;
            }
        }

        Some(&path[i..])
    }
}
