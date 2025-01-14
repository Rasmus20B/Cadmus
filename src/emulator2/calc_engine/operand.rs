use std::ops::{Mul, Add, Sub, Div};
use std::str::FromStr;

use super::calc_eval::EvaluateError;

#[derive(Debug)]
pub enum ParseErr {
    InvalidFormat
}

#[derive(PartialEq, Debug, Clone)]
pub enum Operand {
    Number(f64),
    Text(String),
    FieldName(String),
}

impl Operand {
    pub fn as_num(&self) -> f64 {
        match self {
            Operand::Number(n) => *n,
            Operand::Text(s) => {
                let string = s.chars()
                    .inspect(|c| println!("{}", c))
                    .scan(false, |dot_seen, c| {
                        if c.is_ascii_digit() { 
                            Some(Some(c)) 
                        } else if c == '.' && !*dot_seen {
                            *dot_seen = true;
                            Some(Some(c))
                        } else {
                            Some(None)
                        }
                    })
                    .filter_map(|c| c)
                    .collect::<String>();
                    println!("{:?}", string);
                if string.is_empty() {
                    0.0
                } else {
                    let mut start = String::from("0");
                    start.push_str(&string);
                    string.parse::<f64>().unwrap()
                }
            },
            _ => 0.0
        }
    }

    pub fn concat(self, rhs: Operand) -> Self {
        let lhs_s = match self {
            Operand::Text(s) => s.strip_suffix("\"").unwrap().to_string(),
            Operand::Number(n) => n.to_string(),
            _ => String::new()
        };

        let rhs_s = match rhs {
            Operand::Text(s) => s.strip_prefix("\"").unwrap().to_string(),
            Operand::Number(n) => n.to_string(),
            _ => String::new()
        };

        Operand::Text(lhs_s + &rhs_s)
    }
}

impl Mul for Operand {
    type Output = Operand;
    fn mul(self, rhs: Operand) -> <Self as Mul<Operand>>::Output {
        Operand::Number(self.as_num() * rhs.as_num())
    }
}

impl Add for Operand {
    type Output = Operand;
    fn add(self, rhs: Operand) -> <Self as Add<Operand>>::Output {
        Operand::Number(self.as_num() + rhs.as_num())
    }
}

impl Sub for Operand {
    type Output = Operand;
    fn sub(self, rhs: Operand) -> <Self as Sub<Operand>>::Output {
        Operand::Number(self.as_num() - rhs.as_num())
    }
}

impl Div for Operand {
    type Output = Result<Operand, EvaluateError>;
    fn div(self, rhs: Operand) -> <Self as Div<Operand>>::Output {
        let rhs_n = rhs.as_num();
        if rhs_n == 0.0 {
            return Err(EvaluateError::DivideByZero { operand_left: self.as_num().to_string(), operand_right: rhs_n.to_string() })
        }

        Ok(Operand::Number(self.as_num() / rhs.as_num()))
    }
}

impl ToString for Operand {
   fn to_string(&self) -> String {
       match self {
           Operand::Number(n) => n.to_string(),
           Operand::Text(s) => s.clone(),
           Operand::FieldName(s) => s.clone(),
       }
   }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn num_parse() {
        let op1 = Operand::Number(45.0);
        assert_eq!(op1.as_num(), 45.0);
        let op2 = Operand::Text(String::from("s.4.5"));
        assert_eq!(op2.as_num(), 0.45);
        assert_eq!(op1 + op2, Operand::Number(45.45));
        assert_eq!(Operand::Text(String::from("Hello")).as_num(), 0.0);
        assert_eq!(Operand::Number(20.0) * Operand::Number(10.0), Operand::Number(200.0));
        assert_eq!(Operand::Number(20.0) * Operand::Text(String::from("Hello")), Operand::Number(0.0));
        assert_eq!(Operand::Number(10.0) / Operand::Number(0.0), Err(EvaluateError::DivideByZero { operand_left: 10.0.to_string(), operand_right: 0.0.to_string() }));
    }
}






