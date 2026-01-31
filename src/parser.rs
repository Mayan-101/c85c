// src/parser.rs

use crate::lexer::Token;

/// A more precise Abstract Syntax Tree (AST) node.
#[derive(Debug, PartialEq)]
pub enum Statement {
    // For MVI A, 0x08
    MoveImmediate {
        register: String,
        value: String,
    },
    // For LXI HL, 0x6000
    LoadImmediateExtended {
        register_pair: String,
        address: String,
    },
    // For counter = 0x06; (static allocation)
    StaticAssignment {
        variable: String,
        value: String,
        is_16bit: bool,
    },
    // For A + B; (A = A + B with second operand always B)
    BinaryOp {
        register: String,
        operator: BinaryOperator,
    },
    // For HL++; or HL--;
    PointerIncDec {
        register_pair: String,
        is_increment: bool,
    },
    // For if(counter > result) { ... } or if(A > B) { ... }
    If {
        left: String,       // register or variable name
        condition: Condition,
        right: String,      // register or variable name
        body: Vec<Statement>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperator {
    Add,    // +
    Sub,    // -
    And,    // &
    Or,     // |
    Xor,    // ^
}

#[derive(Debug, PartialEq, Clone)]
pub enum Condition {
    Greater,     // >
    Less,        // <
    Equal,       // ==
}

/// Validates hex literal bounds
fn validate_hex(value: &str, expected_16bit: bool) -> Result<(), String> {
    let hex_str = value.trim_start_matches("0x").trim_start_matches("0X");
    let num = u64::from_str_radix(hex_str, 16)
        .map_err(|_| format!("Invalid hex literal: {}", value))?;
    
    if expected_16bit {
        if num > 0xFFFF {
            return Err(format!("16-bit value {} exceeds maximum (0xFFFF)", value));
        }
    } else {
        if num > 0xFF {
            return Err(format!("8-bit value {} exceeds maximum (0xFF)", value));
        }
    }
    Ok(())
}

/// Checks if a register is 16-bit
fn is_16bit_register(reg: &str) -> bool {
    matches!(reg, "HL" | "BC" | "DE" | "SP")
}

/// Infers if value needs 16-bit storage
fn is_16bit_value(value: &str) -> bool {
    let hex_str = value.trim_start_matches("0x").trim_start_matches("0X");
    u64::from_str_radix(hex_str, 16).unwrap_or(0) > 0xFF
}

/// Parses a slice of Tokens into a list of Statements (our AST).
pub fn parse(tokens: &[Token]) -> Result<Vec<Statement>, String> {
    let mut statements = Vec::new();
    let mut i = 0;

    // Expect main { ... }
    if tokens.get(i) != Some(&Token::Main) || tokens.get(i+1) != Some(&Token::LBrace) {
        return Err("Expected 'main{' at the beginning of the file.".to_string());
    }
    i += 2; // Consume "main" and "{"

    parse_block(tokens, &mut i, &mut statements)?;

    Ok(statements)
}

/// Parse a block of statements (handles nested blocks for if statements)
fn parse_block(tokens: &[Token], i: &mut usize, statements: &mut Vec<Statement>) -> Result<(), String> {
    while *i < tokens.len() && tokens[*i] != Token::RBrace {
        // Check what kind of statement this is
        match tokens.get(*i) {
            Some(Token::Reg) => {
                // Register assignment: reg A = 0x08; or reg HL = malloc(0x6000);
                let register = match tokens.get(*i + 1) {
                    Some(Token::Identifier(name)) => name.clone(),
                    _ => return Err("Expected a register name after 'reg'.".to_string()),
                };

                if tokens.get(*i + 2) != Some(&Token::Equal) {
                    return Err("Expected '=' after register name.".to_string());
                }

                match tokens.get(*i + 3) {
                    // Direct value assignment: reg A = 0x08;
                    Some(Token::HexLiteral(value)) => {
                        let is_16bit = is_16bit_register(&register);
                        validate_hex(value, is_16bit)?;
                        
                        statements.push(Statement::MoveImmediate {
                            register,
                            value: value.clone(),
                        });
                        *i += 4; // Consumed: reg, A, =, 0x08
                    }
                    // Malloc call: reg HL = malloc(0x6000);
                    Some(Token::Malloc) => {
                        if !is_16bit_register(&register) {
                            return Err(format!("malloc() requires a 16-bit register pair, got {}", register));
                        }
                        
                        let address = match tokens.get(*i + 5) {
                            Some(Token::HexLiteral(addr)) => addr.clone(),
                            _ => return Err("Expected a hex address inside malloc().".to_string()),
                        };

                        validate_hex(&address, true)?;

                        if tokens.get(*i + 4) != Some(&Token::LParen) || tokens.get(*i + 6) != Some(&Token::RParen) {
                            return Err("Malformed malloc() call. Expected malloc(ADDRESS).".to_string());
                        }

                        statements.push(Statement::LoadImmediateExtended {
                            register_pair: register,
                            address,
                        });
                        *i += 7; // Consumed: reg, HL, =, malloc, (, 0x6000, )
                    }
                    _ => return Err("Invalid expression after '='.".to_string()),
                }

                // Expect semicolon
                if tokens.get(*i) != Some(&Token::Semicolon) {
                    return Err("Expected ';' at the end of the statement.".to_string());
                }
                *i += 1; // Consume ";"
            }
            Some(Token::Identifier(name)) => {
                let identifier = name.clone();
                
                // Check what follows: =, +, -, &, |, ^, ++, --
                match tokens.get(*i + 1) {
                    Some(Token::Equal) => {
                        // Static allocation: counter = 0x06;
                        let value = match tokens.get(*i + 2) {
                            Some(Token::HexLiteral(v)) => v.clone(),
                            _ => return Err(format!("Expected hex value after '=' for variable '{}'.", identifier)),
                        };

                        let is_16bit = is_16bit_value(&value);
                        validate_hex(&value, is_16bit)?;

                        statements.push(Statement::StaticAssignment {
                            variable: identifier,
                            value,
                            is_16bit,
                        });
                        *i += 3; // Consumed: identifier, =, value
                    }
                    Some(Token::Plus) | Some(Token::Minus) | Some(Token::And) | Some(Token::Or) | Some(Token::Xor) => {
                        // Binary operation: A + B;
                        let operator = match tokens.get(*i + 1) {
                            Some(Token::Plus) => BinaryOperator::Add,
                            Some(Token::Minus) => BinaryOperator::Sub,
                            Some(Token::And) => BinaryOperator::And,
                            Some(Token::Or) => BinaryOperator::Or,
                            Some(Token::Xor) => BinaryOperator::Xor,
                            _ => unreachable!(),
                        };

                        // Second operand must be B
                        if tokens.get(*i + 2) != Some(&Token::Identifier("B".to_string())) {
                            return Err("Second operand must be register B.".to_string());
                        }

                        statements.push(Statement::BinaryOp {
                            register: identifier,
                            operator,
                        });
                        *i += 3; // Consumed: A, +, B
                    }
                    Some(Token::PlusPlus) => {
                        // Pointer increment: HL++;
                        if !is_16bit_register(&identifier) {
                            return Err(format!("Increment/decrement requires a 16-bit register pair, got {}", identifier));
                        }

                        statements.push(Statement::PointerIncDec {
                            register_pair: identifier,
                            is_increment: true,
                        });
                        *i += 2; // Consumed: HL, ++
                    }
                    Some(Token::MinusMinus) => {
                        // Pointer decrement: HL--;
                        if !is_16bit_register(&identifier) {
                            return Err(format!("Increment/decrement requires a 16-bit register pair, got {}", identifier));
                        }

                        statements.push(Statement::PointerIncDec {
                            register_pair: identifier,
                            is_increment: false,
                        });
                        *i += 2; // Consumed: HL, --
                    }
                    _ => return Err(format!("Unexpected token after identifier '{}'.", identifier)),
                }

                // Expect semicolon
                if tokens.get(*i) != Some(&Token::Semicolon) {
                    return Err("Expected ';' at the end of the statement.".to_string());
                }
                *i += 1; // Consume ";"
            }
            Some(Token::If) => {
                // If statement: if(A > B) { ... } or if(counter > result) { ... }
                *i += 1; // Consume "if"

                if tokens.get(*i) != Some(&Token::LParen) {
                    return Err("Expected '(' after 'if'.".to_string());
                }
                *i += 1; // Consume "("

                let left = match tokens.get(*i) {
                    Some(Token::Identifier(name)) => name.clone(),
                    _ => return Err("Expected register or variable name in condition.".to_string()),
                };
                *i += 1;

                let condition = match tokens.get(*i) {
                    Some(Token::Greater) => Condition::Greater,
                    Some(Token::Less) => Condition::Less,
                    Some(Token::EqualEqual) => Condition::Equal,
                    _ => return Err("Expected condition: '>', '<', or '=='.".to_string()),
                };
                *i += 1;

                let right = match tokens.get(*i) {
                    Some(Token::Identifier(name)) => name.clone(),
                    _ => return Err("Expected register or variable name in condition.".to_string()),
                };
                *i += 1;

                if tokens.get(*i) != Some(&Token::RParen) {
                    return Err("Expected ')' after condition.".to_string());
                }
                *i += 1; // Consume ")"

                if tokens.get(*i) != Some(&Token::LBrace) {
                    return Err("Expected '{' after condition.".to_string());
                }
                *i += 1; // Consume "{"

                let mut body = Vec::new();
                parse_block(tokens, i, &mut body)?;

                if tokens.get(*i) != Some(&Token::RBrace) {
                    return Err("Expected '}' to close if block.".to_string());
                }
                *i += 1; // Consume "}"

                statements.push(Statement::If {
                    left,
                    condition,
                    right,
                    body,
                });
            }
            _ => return Err(format!("Expected statement, found {:?}", tokens.get(*i))),
        }
    }

    Ok(())
}