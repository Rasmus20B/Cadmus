
use chrono::Local;
use super::{calc_parser::*, calc_lexer::*};
use crate::emulator::emulator::Emulator;
use super::calc_tokens::TokenType;

#[derive(PartialEq, Debug)]
enum OperandType {
    Number,
    Text,
    FieldName,
}

#[derive(PartialEq, Debug)]
struct Operand<'a> {
    value: &'a str,
    otype: OperandType
}

#[derive(Debug)]
enum EvaluateError {
    DivideByZero { operand_left: String, operand_right: String },
    InvalidOperation { operand_left: String, operator: String, operand_right: String },
    InvalidArgument { function: String, argument: String },
    UnimplementedFunction { function: String },
}

pub fn eval_calculation(calculation: &str, emulator: &Emulator) -> String {
    let tokens = lex(calculation).expect("Unable to lex calculation.");
    let ast = Parser::new(tokens).parse().expect("unable to parse tokens.");
    evaluate(*ast, emulator).expect("Unable to evaluate expression.")
}

fn get_operand_val<'a>(val: &'a str, emulator: &'a Emulator) -> Result<Operand<'a>, String> {
    let r = val.parse::<f64>();
    if r.is_ok() {
        return Ok(Operand {
            otype: OperandType::Number,
            value: val
        })
    }

    if val.starts_with("\"") && val.ends_with("\"") {
        return Ok(Operand {
            otype: OperandType::Text,
            value: val
        })
    }

    let fieldname = val.split("::").collect::<Vec<&str>>();
    if fieldname.len() == 2 {
        let val = emulator.get_active_database().get_found_set_record_field(fieldname[0], fieldname[1]);
        return get_operand_val(val, emulator);
    } else {
        let scope = emulator.scr_state.instruction_ptr.len() - 1;
        let var_val = emulator.scr_state.variables[scope]
            .get(val);

        if var_val.is_none() {
            return Err("Unable to evaluate script step argument.".to_string());
        }
        return get_operand_val(&var_val.unwrap().value, emulator);
    }

}


pub fn evaluate(ast: Node, emulator: &Emulator) -> Result<String, EvaluateError> {
    match ast {
        Node::Unary { value, child } => {
            let val = get_operand_val(value.as_str(), emulator).unwrap()
                .value.to_string();

            if child.is_none() {
                return Ok(val);
            } 
            Ok(val.to_string())
        },
        Node::Grouping { left, operation, right } => {
            let lhs_wrap = evaluate(*left.clone(), emulator)?;
            let rhs_wrap = evaluate(*right, emulator)?;

            let mut lhs = match *left {
                Node::Number(val) => Operand { value: &val.to_string(), otype: OperandType::Number },
                _ => get_operand_val(&lhs_wrap, emulator).unwrap()
            };
            let mut rhs = get_operand_val(&rhs_wrap, emulator).unwrap();

            match operation {
                TokenType::Multiply => {
                    Ok((lhs.value.parse::<f32>().unwrap() * rhs.value.parse::<f32>().unwrap()).to_string())
                }
                TokenType::Plus => {
                    Ok((lhs.value.parse::<f32>().unwrap() + rhs.value.parse::<f32>().unwrap()).to_string())
                }
                TokenType::Ampersand => {
                    if lhs.otype == OperandType::Text {
                        lhs.value = lhs.value
                        .strip_prefix('"').unwrap()
                        .strip_suffix('"').unwrap();
                    }
                    if rhs.otype == OperandType::Text {
                        rhs.value = rhs.value
                        .strip_prefix('"').unwrap()
                        .strip_suffix('"').unwrap();
                    }
                    Ok(("\"".to_owned() + lhs.value + rhs.value + "\"").to_string())
                }
                _ => Ok(format!("Invalid operation {:?}.", operation).to_string())
            }
        }
        Node::Call { name, args } => {
            match name.as_str() {
                "Abs" => { Ok(evaluate(args[0].clone(), emulator)?
                    .parse::<f32>().expect("unable to perform Abs() on non-numeric")
                    .abs().to_string())
                }
                "Acos" => { Ok(evaluate(args[0].clone(), emulator)?.parse::<f64>().unwrap().acos().to_string()) }
                "Asin" => { Ok(std::cmp::min(evaluate(args[0].clone(), emulator)?, evaluate(args[1].clone(), emulator)?))}
                "Char" => { 
                    let mut res = String::new();
                    res.push('\"');
                    let c = char::from_u32(evaluate(args[0].clone(), emulator).expect("Unable to evalue argument.").parse::<u32>().unwrap()).unwrap();
                    res.push(c);
                    res.push('\"');
                    Ok(res)
                }
                "Min" => { 
                    Ok(args
                        .into_iter()
                        .map(|x| {
                            evaluate(x.clone(), emulator).expect("Unable to evaluate argument.")
                        })
                        .min_by(|x, y| {
                            let lhs = get_operand_val(x, emulator).unwrap();
                            let rhs = get_operand_val(y, emulator).unwrap();

                            if lhs.otype == OperandType::Number && rhs.otype == OperandType::Number {
                                    lhs.value.parse::<f64>().expect("lhs is not a valid number")
                                        .partial_cmp(
                                    &rhs.value.parse::<f64>().expect("rhs is not a number.")
                                    ).unwrap()
                            } else {
                                lhs.value.to_string().cmp(&rhs.value.to_string())
                            }
                        }).unwrap())
                },
                "Get" | "get" => {
                    match args[0] {
                        Node::Unary { ref value, child: _ } | Node::Variable(ref value) => {
                            match value.as_str() {
                                "AccountName" => Ok("\"Admin\"".to_string()),
                                "CurrentTime" => Ok(Local::now().timestamp().to_string()),
                                "FoundCount" => Ok(emulator.get_active_database().get_current_occurrence().found_set.len().to_string()),
                                _ => { Err(EvaluateError::InvalidArgument { function: name, argument: value.to_string() }) }
                            }
                        }
                        // TODO: Evaluate expression into string and then do the match.
                        _ => unimplemented!()
                    }
                },
                "Count" => {
                    let (table, field) = match &args[0] {
                        Node::Unary { value, child: _ } => {
                            let parts = value.split("::").collect::<Vec<_>>();
                            (parts[0], parts[1])
                        },
                        Node::Field(inner) => {
                            let parts = inner.split("::").collect::<Vec<_>>();
                            (parts[0], parts[1])
                        },
                        _ => { unimplemented!() }
                    };
                    Ok(emulator.get_active_database()
                        .get_related_records(table).unwrap_or_default()
                        .len().to_string())
                }
                _ => { Err(EvaluateError::UnimplementedFunction { function: name }) }
            }
        },
        Node::Binary { left, operation, right } => {
            let lhs_wrap = &evaluate(*left, emulator).unwrap();
            let rhs_wrap = &evaluate(*right, emulator).unwrap();
            let mut lhs = get_operand_val(lhs_wrap, emulator).unwrap();
            let mut rhs = get_operand_val(rhs_wrap, emulator).unwrap();

            match operation {
                TokenType::Plus => { 
                    if lhs.otype == OperandType::Text { lhs.value = "0" }
                    if rhs.otype == OperandType::Text { rhs.value = "0" }
                    Ok((lhs.value.parse::<f64>().unwrap()
                     + 
                     rhs.value.parse::<f64>().unwrap()
                     ).to_string())
                },
                TokenType::Minus => { 
                    if lhs.otype == OperandType::Text { lhs.value = "0" }
                    if rhs.otype == OperandType::Text { rhs.value = "0" }
                    Ok((lhs.value.parse::<f64>().unwrap()
                     - 
                     rhs.value.parse::<f64>().unwrap()
                     ).to_string())
                },
                TokenType::Multiply => { 
                    if lhs.otype == OperandType::Text { lhs.value = "0" }
                    if rhs.otype == OperandType::Text { rhs.value = "0" }
                    Ok((lhs.value.parse::<f64>().unwrap()
                     * 
                     rhs.value.parse::<f64>().unwrap()
                     ).to_string())
                },
                TokenType::Divide => { 
                    if lhs.otype == OperandType::Text { lhs.value = "0" }
                    if rhs.otype == OperandType::Text { rhs.value = "0" }

                    let checked_rhs = rhs.value.parse::<f64>().unwrap();
                    if checked_rhs == 0.0 { 
                        return Err(EvaluateError::DivideByZero { 
                            operand_left: lhs.value.to_string(), 
                            operand_right: checked_rhs.to_string() 
                        }) 
                    }
                    Ok((lhs.value.parse::<f64>().unwrap()
                     / 
                     rhs.value.parse::<f64>().unwrap()
                     ).to_string())
                },
                _ => { unreachable!()}
            }
        },
        Node::Comparison { left, operation, right } => {
            Ok(match operation {
                TokenType::Eq => { (evaluate(*left, emulator)? == evaluate(*right, emulator)?).to_string() },
                TokenType::Neq => { (evaluate(*left, emulator)? != evaluate(*right, emulator)?).to_string() },
                TokenType::Gt => { (evaluate(*left, emulator)? > evaluate(*right, emulator)?).to_string() },
                TokenType::Gtq => { (evaluate(*left, emulator)? >= evaluate(*right, emulator)?).to_string() },
                TokenType::Lt => { (evaluate(*left, emulator)? < evaluate(*right, emulator)?).to_string() },
                TokenType::Ltq => { (evaluate(*left, emulator)? <= evaluate(*right, emulator)?).to_string() },
                _ => unreachable!()
            })
        }
        Node::Concatenation { left, right } => { 
            let lhs = evaluate(*left, emulator)?;
            let rhs = evaluate(*right, emulator)?;
            let lhs = lhs.replace('"', "");
            let rhs = rhs.replace('"', "");
            Ok(format!("\"{lhs}{rhs}\""))
        }
        Node::Number(val) => Ok(val.to_string()),
        Node::Variable(val) => Ok(get_operand_val(&val, emulator).unwrap().value.to_string()),
        Node::Field(val) => Ok(get_operand_val(&val, emulator).unwrap().value.to_string()),
        Node::StringLiteral(val) => Ok(val.to_string()),
    }
}
