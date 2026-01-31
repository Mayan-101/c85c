// src/codegen.rs

use crate::parser::{Statement, BinaryOperator, Condition};
use std::collections::HashMap;

/// Generates 8085 assembly code from a list of Statements.
pub fn generate(statements: &[Statement]) -> String {
    let mut asm_code = String::new();
    let mut static_vars: HashMap<String, u16> = HashMap::new();
    let mut var_to_register: HashMap<String, String> = HashMap::new();
    let mut next_address = 0x8000u16;
    let mut label_counter = 0;
    let registers = vec!["A", "B", "C", "D", "E"];
    let mut register_idx = 0;

    // First pass: allocate addresses and assign registers for static variables
    allocate_static_vars(statements, &mut static_vars, &mut next_address, &mut var_to_register, &registers, &mut register_idx);

    // Second pass: generate code
    for statement in statements {
        generate_statement(statement, &static_vars, &var_to_register, &mut asm_code, &mut label_counter);
    }

    asm_code
}

/// First pass: allocate addresses and assign registers for static variables
fn allocate_static_vars(
    statements: &[Statement], 
    static_vars: &mut HashMap<String, u16>, 
    next_address: &mut u16,
    var_to_register: &mut HashMap<String, String>,
    registers: &[&str],
    register_idx: &mut usize
) {
    for statement in statements {
        match statement {
            Statement::StaticAssignment { variable, .. } => {
                if !static_vars.contains_key(variable) {
                    static_vars.insert(variable.clone(), *next_address);
                    *next_address += 1;
                    
                    // Assign to next available register
                    if *register_idx < registers.len() {
                        var_to_register.insert(variable.clone(), registers[*register_idx].to_string());
                        *register_idx += 1;
                    }
                }
            }
            Statement::If { body, .. } => {
                allocate_static_vars(body, static_vars, next_address, var_to_register, registers, register_idx);
            }
            _ => {}
        }
    }
}

/// Generate assembly for a single statement
fn generate_statement(
    statement: &Statement, 
    static_vars: &HashMap<String, u16>,
    var_to_register: &HashMap<String, String>,
    asm_code: &mut String, 
    label_counter: &mut i32
) {
    match statement {
        Statement::MoveImmediate { register, value } => {
            let numeric_val = value.trim_start_matches("0x").trim_start_matches("0X");
            asm_code.push_str(&format!("MVI {},{}H;\n", register, numeric_val.to_uppercase()));
        }
        Statement::LoadImmediateExtended { register_pair, address } => {
            let numeric_addr = address.trim_start_matches("0x").trim_start_matches("0X");
            asm_code.push_str(&format!("LXI {},{}H;\n", register_pair, numeric_addr.to_uppercase()));
        }
        Statement::StaticAssignment { variable, value, is_16bit } => {
            let addr = static_vars[variable];
            let numeric_val = value.trim_start_matches("0x").trim_start_matches("0X");
            
            if *is_16bit {
                // For 16-bit: LXI H, value; SHLD address
                asm_code.push_str(&format!("LXI H,{}H;\n", numeric_val.to_uppercase()));
                asm_code.push_str(&format!("SHLD {:04X}H;\n", addr));
                
                // If assigned to a register, load lower byte into that register
                if let Some(reg) = var_to_register.get(variable) {
                    asm_code.push_str(&format!("MOV {},L;\n", reg));
                }
            } else {
                // For 8-bit: MVI A, value; STA address
                asm_code.push_str(&format!("MVI A,{}H;\n", numeric_val.to_uppercase()));
                asm_code.push_str(&format!("STA {:04X}H;\n", addr));
                
                // If assigned to a register (and it's not A), move from A
                if let Some(reg) = var_to_register.get(variable) {
                    if reg != "A" {
                        asm_code.push_str(&format!("MOV {},A;\n", reg));
                    }
                }
            }
        }
        Statement::BinaryOp { register, operator } => {
            // All operations use register B as second operand
            let instruction = match operator {
                BinaryOperator::Add => "ADD B",
                BinaryOperator::Sub => "SUB B",
                BinaryOperator::And => "ANA B",
                BinaryOperator::Or => "ORA B",
                BinaryOperator::Xor => "XRA B",
            };
            
            // If register is not A, we need to move it to A first
            if register != "A" {
                asm_code.push_str(&format!("MOV A,{};\n", register));
            }
            asm_code.push_str(&format!("{};\n", instruction));
            // Result is in A, move back if needed
            if register != "A" {
                asm_code.push_str(&format!("MOV {},A;\n", register));
            }
        }
        Statement::PointerIncDec { register_pair, is_increment } => {
            let instruction = if *is_increment {
                format!("INX {}", register_pair)
            } else {
                format!("DCX {}", register_pair)
            };
            asm_code.push_str(&format!("{};\n", instruction));
        }
        Statement::If { left, condition, right, body } => {
            let label = *label_counter;
            *label_counter += 1;
            
            // Resolve left and right to actual registers
            let left_reg = var_to_register.get(left).unwrap_or(left).clone();
            let right_reg = var_to_register.get(right).unwrap_or(right).clone();
            
            // Move left operand to A if not already A
            if left_reg != "A" {
                asm_code.push_str(&format!("MOV A,{};\n", left_reg));
            }
            
            // Compare A with right operand
            if right_reg == "A" {
                // Comparing with itself, use CPI instead
                asm_code.push_str("CPI 00H;\n");
            } else {
                asm_code.push_str(&format!("CMP {};\n", right_reg));
            }
            
            // Jump based on condition
            let jump_instruction = match condition {
                Condition::Equal => format!("JNZ SKIP_{};\n", label),      // Jump if not zero (not equal)
                Condition::Greater => format!("JZ SKIP_{};\nJC SKIP_{};\n", label, label), // Jump if zero or carry (<=)
                Condition::Less => format!("JZ SKIP_{};\nJNC SKIP_{};\n", label, label),  // Jump if zero or no carry (>=)
            };
            asm_code.push_str(&jump_instruction);
            
            // Generate body
            for stmt in body {
                generate_statement(stmt, static_vars, var_to_register, asm_code, label_counter);
            }
            
            // Skip label
            asm_code.push_str(&format!("SKIP_{}:\n", label));
        }
    }
}