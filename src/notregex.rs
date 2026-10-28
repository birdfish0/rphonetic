use std::{error::Error, fmt::Display};

use self::Pattern::*;

#[derive(PartialEq, Eq, Clone, Debug)]
enum Pattern {
    Sequence(String),
    NotChar(Vec<char>),
    IsChar(Vec<char>),
    LineStart,
    LineEnd,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub struct RegexParseError {
    /// Contents of the erroneous input that failed to parse.
    failed_content: String,
    /// Description of the error.
    description: String,
}

impl Display for RegexParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<RegexParseError in \"{}\": {}>",
            self.failed_content, self.description
        )
    }
}

#[derive(Debug, Clone)]
pub struct Regex {
    patterns: Vec<Pattern>,
}

impl Display for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut outstr = String::new();
        for pat in &self.patterns {
            match pat {
                IsChar(chrs) => {
                    outstr = outstr + "[" + &chrs.iter().collect::<String>() + "]";
                }
                NotChar(chrs) => {
                    outstr = outstr + "[^" + &chrs.iter().collect::<String>() + "]";
                }
                LineStart => {
                    outstr += "^";
                }
                LineEnd => {
                    outstr += "$";
                }
                Sequence(seq) => {
                    outstr += &seq;
                }
            }
        }
        write!(f, "{}", outstr)
    }
}

impl Regex {
    pub fn check_match(&self, offset: usize, chars: &Vec<char>) -> bool {
        let mut idx = 0;
        for pat in &self.patterns {
            if (*pat != LineEnd || idx + offset > chars.len()) && idx + offset >= chars.len() {
                return false;
            }
            if !match pat {
                IsChar(is) => {
                    let res = is.iter().any(|c| *c == chars[idx + offset]);
                    idx += 1;
                    res
                }
                LineEnd => {
                    idx += 1;
                    if idx + offset - 1 > chars.len() {
                        false
                    } else if idx + offset - 1 == chars.len() {
                        true
                    } else {
                        chars[idx + offset - 1] == '\n'
                    }
                }
                LineStart => idx + offset == 0 || chars[idx + offset - 1] == '\n',
                NotChar(exclude) => {
                    let res = exclude.iter().all(|c| *c != chars[idx + offset]);
                    idx += 1;
                    res
                }
                Sequence(seq) => {
                    let mut seqchars = seq.chars();
                    let mut res = true;
                    while let Some(c) = seqchars.next() {
                        if idx + offset < chars.len() {
                            if chars[idx + offset] != c {
                                res = false;
                                break;
                            }
                            idx += 1;
                        } else {
                            res = false;
                            break;
                        }
                    }
                    res
                }
            } {
                return false;
            }
        }
        true
    }

    pub fn new(input: &str) -> Result<Self, RegexParseError> {
        let mut patterns = Vec::new();
        let mut chars = input.chars();
        let mut curr_pattern: Option<Pattern> = None;
        while let Some(cchar) = chars.next() {
            if let Some(ref mut pattern) = curr_pattern {
                match pattern {
                    IsChar(find) => {
                        if cchar == ']' {
                            patterns.push(pattern.to_owned());
                            curr_pattern = None;
                        } else {
                            find.push(cchar);
                        }
                        continue;
                    }
                    NotChar(find) => {
                        if cchar == ']' {
                            patterns.push(pattern.to_owned());
                            curr_pattern = None;
                        } else {
                            find.push(cchar);
                        }
                        continue;
                    }
                    _ => (),
                }
            }
            if cchar == '[' {
                if let Some(nextchar) = chars.next() {
                    if let Some(patt) = curr_pattern {
                        patterns.push(patt);
                    }
                    if nextchar == '^' {
                        curr_pattern = Some(NotChar(Vec::new()));
                    } else {
                        curr_pattern = Some(IsChar(vec![nextchar]));
                    }
                    continue;
                } else {
                    return Err(RegexParseError {
                        description: "Expected '^' or character after '[' in regex, got EOF."
                            .to_owned(),
                        failed_content: input.to_owned(),
                    });
                }
            }
            if cchar == '^' {
                if let Some(patt) = curr_pattern {
                    patterns.push(patt);
                    curr_pattern = None;
                }
                patterns.push(LineStart);
                continue;
            }
            if cchar == '$' {
                if let Some(patt) = curr_pattern {
                    patterns.push(patt);
                    curr_pattern = None;
                }
                patterns.push(LineEnd);
                continue;
            }
            if curr_pattern.is_none() {
                curr_pattern = Some(Sequence(cchar.to_string()));
            } else if let Some(ref mut patt) = curr_pattern {
                match patt {
                    Sequence(seq) => seq.push(cchar),
                    _ => (),
                }
            }
        }
        if let Some(p) = curr_pattern {
            match p {
                IsChar(include) => {
                    let pat_str = "[".to_owned() + &include.iter().collect::<String>();
                    return Err(RegexParseError {
                        description: format!("Expected rest of expression `{}`, got EOF.", pat_str),
                        failed_content: input.to_owned(),
                    });
                }
                NotChar(include) => {
                    let pat_str = "[^".to_owned() + &include.iter().collect::<String>();
                    return Err(RegexParseError {
                        description: format!("Expected rest of expression `{}`, got EOF.", pat_str),
                        failed_content: input.to_owned(),
                    });
                }
                Sequence(_) => patterns.push(p),
                _ => (),
            }
        }
        // println!("{:?}", patterns);
        Ok(Self { patterns })
    }

    pub fn is_match(&self, input: &str) -> bool {
        let chars = input.chars().collect::<Vec<char>>();
        let mut offset: usize = 0;
        let mut res = false;
        while offset < chars.len() {
            let found = self.check_match(offset, &chars);
            if found {
                res = true;
                break;
            }
            offset += 1;
        }
        res
    }
}
