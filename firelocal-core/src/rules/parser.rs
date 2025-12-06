use crate::rules::ast::{AllowStatement, MatchBlock, Ruleset};
use std::iter::Peekable;
use std::str::Chars;

pub struct RulesParser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> RulesParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    // Simplistic parser: specific structure expected
    // rule = "service" "cloud.firestore" "{" "match" path "{" [match_blocks] [allow_stmts] "}" "}"

    pub fn parse(&mut self) -> Result<Ruleset, String> {
        self.consume_whitespace();
        self.expect("service")?;
        self.consume_whitespace();
        self.expect("cloud.firestore")?;
        self.consume_whitespace();
        self.expect("{")?;

        let match_block = self.parse_match_block()?;

        self.consume_whitespace();
        self.expect("}")?;

        Ok(Ruleset {
            service_name: "cloud.firestore".to_string(),
            match_block,
        })
    }

    fn parse_match_block(&mut self) -> Result<MatchBlock, String> {
        self.consume_whitespace();
        self.expect("match")?;
        self.consume_whitespace();

        let pattern = self.parse_path()?;

        self.consume_whitespace();
        self.expect("{")?;

        let mut sub_matches = Vec::new();
        let mut allow_statements = Vec::new(); // M4 MVP: Only supporting allows for now

        loop {
            self.consume_whitespace();
            if self.check("match") {
                sub_matches.push(self.parse_match_block()?);
            } else if self.check("allow") {
                allow_statements.push(self.parse_allow()?);
            } else if self.check("}") {
                self.expect("}")?;
                break;
            } else {
                return Err("Unexpected token in match block".to_string());
            }
        }

        Ok(MatchBlock {
            path_pattern: pattern,
            sub_matches,
            allow_statements,
        })
    }

    fn parse_allow(&mut self) -> Result<AllowStatement, String> {
        self.consume_whitespace();
        self.expect("allow")?;
        self.consume_whitespace();

        // Parse operations: read, write, ...
        let mut operations = Vec::new();
        loop {
            let op = self.parse_identifier()?;
            operations.push(op);
            self.consume_whitespace();
            if self.check(",") {
                self.expect(",")?;
                self.consume_whitespace();
            } else {
                break;
            }
        }

        self.expect(":")?;
        self.consume_whitespace();
        self.expect("if")?;
        self.consume_whitespace();

        // Parse condition until semicolon
        let condition = self.parse_until(";")?;
        self.expect(";")?;

        Ok(AllowStatement {
            operations,
            condition: condition.trim().to_string(),
        })
    }

    fn parse_path(&mut self) -> Result<String, String> {
        // Read until whitespace or {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                break;
            }
            s.push(c);
            self.chars.next();
        }
        Ok(s)
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        let mut s = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_alphabetic() || c == '_' {
                s.push(c);
                self.chars.next();
            } else {
                break;
            }
        }
        if s.is_empty() {
            return Err("Expected identifier".to_string());
        }
        Ok(s)
    }

    fn parse_until(&mut self, delimiter: &str) -> Result<String, String> {
        let mut s = String::new();
        // Check for delimiter match simplisticly
        while let Some(&c) = self.chars.peek() {
            if c == delimiter.chars().next().unwrap() {
                // Peek ahead to verify?
                // M4 MVP: single char delimiter
                break;
            }
            s.push(c);
            self.chars.next();
        }
        Ok(s)
    }

    fn expect(&mut self, s: &str) -> Result<(), String> {
        for expected_char in s.chars() {
            if let Some(c) = self.chars.next() {
                if c != expected_char {
                    return Err(format!("Expected '{}', found '{}'", expected_char, c));
                }
            } else {
                return Err(format!("Expected '{}', found EOF", expected_char));
            }
        }
        Ok(())
    }

    fn check(&mut self, s: &str) -> bool {
        // Peek ahead without consuming
        // This is inefficient for long strings but fine for "match" / "allow" etc.
        // We'd need to clone iterator or buffer.
        // Helper: Clone, advance clone, check.
        let mut iter = self.chars.clone();
        for expected_char in s.chars() {
            if let Some(c) = iter.next() {
                if c != expected_char {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn consume_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }
}
