// src/lexer.rs

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Main,        // "main"
    Reg,         // "reg"
    Malloc,      // "malloc"
    If,          // "if"

    // Symbols
    LBrace,      // "{"
    RBrace,      // "}"
    LParen,      // "("
    RParen,      // ")"
    Equal,       // "="
    Semicolon,   // ";"
    
    // Operators
    Plus,        // "+"
    Minus,       // "-"
    And,         // "&"
    Or,          // "|"
    Xor,         // "^"
    PlusPlus,    // "++"
    MinusMinus,  // "--"
    
    // Comparisons
    Greater,     // ">"
    Less,        // "<"
    EqualEqual,  // "=="

    // Literals
    Identifier(String), // e.g., "A", "HL"
    HexLiteral(String), // e.g., "0x08", "0x6000"
}

/// A simple, manual lexer. It turns source code into a Vec<Token>.
pub fn lex(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' | '\r' | '\n' => continue, // Skip whitespace
            '{' => tokens.push(Token::LBrace),
            '}' => tokens.push(Token::RBrace),
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            ';' => tokens.push(Token::Semicolon),
            '&' => tokens.push(Token::And),
            '|' => tokens.push(Token::Or),
            '^' => tokens.push(Token::Xor),
            '>' => tokens.push(Token::Greater),
            '<' => tokens.push(Token::Less),
            '=' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::EqualEqual);
                } else {
                    tokens.push(Token::Equal);
                }
            }
            '+' => {
                if chars.peek() == Some(&'+') {
                    chars.next();
                    tokens.push(Token::PlusPlus);
                } else {
                    tokens.push(Token::Plus);
                }
            }
            '-' => {
                if chars.peek() == Some(&'-') {
                    chars.next();
                    tokens.push(Token::MinusMinus);
                } else {
                    tokens.push(Token::Minus);
                }
            }
            '/' => {
                // Check for comments
                if chars.peek() == Some(&'/') {
                    // Single-line comment: skip until newline
                    while let Some(&next_c) = chars.peek() {
                        chars.next();
                        if next_c == '\n' {
                            break;
                        }
                    }
                    continue;
                } else {
                    return Err(format!("Unexpected character: {}", c));
                }
            }
            'a'..='z' | 'A'..='Z' => {
                let mut identifier = String::new();
                identifier.push(c);
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphanumeric() {
                        identifier.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                match identifier.as_str() {
                    "main" => tokens.push(Token::Main),
                    "reg" => tokens.push(Token::Reg),
                    "malloc" => tokens.push(Token::Malloc),
                    "if" => tokens.push(Token::If),
                    _ => {
                        // Could be a register (A, HL) or a variable name later
                        tokens.push(Token::Identifier(identifier))
                    }
                }
            }
            '0' => {
                // Check for 0x prefix
                if chars.peek() == Some(&'x') || chars.peek() == Some(&'X') {
                    chars.next(); // Consume 'x' or 'X'
                    let mut hex_literal = String::from("0x");
                    while let Some(&next_c) = chars.peek() {
                        if next_c.is_ascii_hexdigit() {
                            hex_literal.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    if hex_literal.len() <= 2 {
                        return Err(format!("Invalid hex literal: '{}'. Expected digits after 0x.", hex_literal));
                    }
                    tokens.push(Token::HexLiteral(hex_literal));
                } else {
                    return Err(format!("Invalid number literal. Use 0x prefix for hex values."));
                }
            }
            '1'..='9' => {
                return Err(format!("Invalid number literal starting with '{}'. Use 0x prefix for hex values.", c));
            }
            _ => return Err(format!("Unexpected character: {}", c)),
        }
    }
    Ok(tokens)
}