use std::cell::Cell;
use std::rc::Rc;
use std::fmt;
use std::collections::HashMap;

struct TokenizerImpl {
    input: &'static str,
    current_index: Cell<usize>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum Token {
    Int(i32),
    OpenParen,
    ClosedParen,
    Operator(&'static str),
    Identifier(String),
    InputEnd,
}

trait Tokenizer {
    fn new(name: &'static str) -> Self;
    fn next_token(&self) -> Token;
}

impl Tokenizer for TokenizerImpl {
    fn new(input: &'static str) -> TokenizerImpl {
        TokenizerImpl { input: input, current_index: Cell::new(0), }
    }
    fn next_token(&self) -> Token {
        match self.input.chars().nth(self.current_index.get()) {
            Some('(') => {
                self.current_index.set(self.current_index.get() + 1);
                Token::OpenParen
            },
            Some(')') => {
                self.current_index.set(self.current_index.get() + 1);
                Token::ClosedParen
            }
            Some('+') => {
                self.current_index.set(self.current_index.get() + 1);
                Token::Operator("+")
            },
            Some('-') => {
                self.current_index.set(self.current_index.get() + 1);
                Token::Operator("-")
            },
            Some('*') => {
                self.current_index.set(self.current_index.get() + 1);
                Token::Operator("*")
            },
            Some('/') => {
                self.current_index.set(self.current_index.get() + 1);
                Token::Operator("/")
            },
            Some('=') => {
                self.current_index.set(self.current_index.get() + 1);
                Token::Operator("=")
            },
            Some('a'..='z') | Some('A'..='Z') => {
                let mut identifier = String::new();
                while let Some(c) = self.input.chars().nth(self.current_index.get()) {
                    if c.is_alphabetic() {
                        identifier.push(c);
                        self.current_index.set(self.current_index.get() + 1);
                    } else {
                        break;
                    }
                }
                let result = identifier.clone().to_owned();
                Token::Identifier(result)
            },
            Some('0'..='9') => {
                let mut number = String::new();
                while let Some(c) = self.input.chars().nth(self.current_index.get()) {
                    if c.is_digit(10) {
                        number.push(c);
                        self.current_index.set(self.current_index.get() + 1);
                    } else {
                        break;
                    }
                }
                Token::Int(number.parse::<i32>().unwrap())
            },
            Some(' ' | '\r' | '\n' | '\t') => {
                self.current_index.set(self.current_index.get() + 1);
                self.next_token()
            },
            None => Token::InputEnd,
            x => panic!("Invalid character {}", x.unwrap()),
        }
    }
}

fn tokenize_all(input: &'static str) -> Vec<Token> {
    let tokenizer = TokenizerImpl::new(input);
    let mut tokens = Vec::new();
    loop {
        let token = tokenizer.next_token();
        tokens.push(token.clone());
        if let Token::InputEnd = token.clone() {
            break;
        }
    }
    tokens
}

fn parse(tokens: Vec<Token>) -> AstNode {
    let mut current_index = 0;
    fn parse_expression(tokens: &Vec<Token>, current_index: &mut usize) -> AstNode {
        let mut node = parse_additive(tokens, current_index);
        loop {
            match tokens[*current_index] {
                Token::Operator("=") => {
                    *current_index += 1;
                    node = AstNode::Assign(Rc::new(node), Rc::new(parse_expression(tokens, current_index)));
                },
                _ => break,
            }
        }
        node
    }
    fn parse_additive(tokens: &Vec<Token>, current_index: &mut usize) -> AstNode {
        let mut node = parse_term(tokens, current_index);
        loop {
            match tokens[*current_index] {
                Token::Operator("+") => {
                    *current_index += 1;
                    node = AstNode::Add(Rc::new(node), Rc::new(parse_term(tokens, current_index)));
                },
                Token::Operator("-") => {
                    *current_index += 1;
                    node = AstNode::Subtract(Rc::new(node), Rc::new(parse_term(tokens, current_index)));
                },
                _ => break,
            }
        }
        node
    }
    fn parse_term(tokens: &Vec<Token>, current_index: &mut usize) -> AstNode {
        let mut node = parse_factor(tokens, current_index);
        loop {
            match tokens[*current_index] {
                Token::Operator("*") => {
                    *current_index += 1;
                    node = AstNode::Multiply(Rc::new(node), Rc::new(parse_factor(tokens, current_index)));
                },
                Token::Operator("/") => {
                    *current_index += 1;
                    node = AstNode::Divide(Rc::new(node), Rc::new(parse_factor(tokens, current_index)));
                },
                _ => break,
            }
        }
        node
    }
    fn parse_factor(tokens: &Vec<Token>, current_index: &mut usize) -> AstNode {
        match tokens[*current_index].clone() {
            Token::Int(value) => {
                *current_index += 1;
                AstNode::IntLiteral(value)
            },
            Token::OpenParen => {
                *current_index += 1;
                let node = parse_expression(tokens, current_index);
                match tokens[*current_index] {
                    Token::ClosedParen => {
                        *current_index += 1;
                        node
                    },
                    _ => panic!("Expected closing parenthesis"),
                }
            },
            Token::Identifier(identifier) => {
                *current_index += 1;
                AstNode::Identifier(identifier)
            },
            _ => panic!("Expected integer or opening parenthesis")
        }
    }
    parse_expression(&tokens, &mut current_index)
}

fn interpret(input: &'static str) -> i32 {
    let tokens = tokenize_all(input);
    let ast = parse(tokens);
    evaluate(Rc::new(ast))
}

fn interpret_with_environment(input: &'static str, environment: &mut HashMap<String, i32>) -> i32 {
    let tokens = tokenize_all(input);
    let ast = parse(tokens);
    evaluate_expression(Rc::new(ast), environment)
}

fn interpret_expressions(inputs: Vec<&'static str>) -> i32 {
    let mut result = 0;
    let mut environment = HashMap::new();
    for input in inputs {
        result = interpret_with_environment(input, &mut environment);
    }
    result
}


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Int(value) => write!(f, "Int({})", value),
            Token::OpenParen => write!(f, "OpenParen"),
            Token::ClosedParen => write!(f, "ClosedParen"),
            Token::Operator(value) => write!(f, "Operator({})", value),
            Token::InputEnd => write!(f, "InputEnd"),
            Token::Identifier(value) => write!(f, "Identifier({})", value),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum AstNode {
    Add(Rc<AstNode>, Rc<AstNode>),
    Subtract(Rc<AstNode>, Rc<AstNode>),
    Multiply(Rc<AstNode>, Rc<AstNode>),
    Divide(Rc<AstNode>, Rc<AstNode>),
    Assign(Rc<AstNode>, Rc<AstNode>),
    Identifier(String),
    IntLiteral(i32),
}

fn evaluate_expression(node: Rc<AstNode>, environment: &mut HashMap<String, i32>) -> i32 {
    match *node {
        AstNode::Add(ref left, ref right) =>
            evaluate_expression(left.clone(), environment) + evaluate_expression(right.clone(), environment),
        AstNode::Subtract(ref left, ref right) =>
            evaluate_expression(left.clone(), environment) - evaluate_expression(right.clone(), environment),
        AstNode::Multiply(ref left, ref right) =>
            evaluate_expression(left.clone(), environment) * evaluate_expression(right.clone(), environment),
        AstNode::Divide(ref left, ref right) =>
            evaluate_expression(left.clone(), environment) / evaluate_expression(right.clone(), environment),
        AstNode::IntLiteral(value) =>
            value,
        AstNode::Assign(ref left, ref right) => {
            if let AstNode::Identifier(ref identifier) = **left {
                let value = evaluate_expression(right.clone(), environment);
                environment.insert(identifier.to_string(), value);
                value
            } else {
                panic!("Expected identifier on left side of assignment");
            }
        },
        AstNode::Identifier(ref identifier) => {
            if let Some(value) = environment.get(identifier) {
                *value
            } else {
                panic!("Unknown identifier {}", identifier);
            }
        },
    }
}

fn evaluate(node: Rc<AstNode>) -> i32 {
    let mut environment = HashMap::new();
    evaluate_expression(node, &mut environment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_literal() {
        let node = Rc::new(AstNode::IntLiteral(1));
        assert_eq!(evaluate(node), 1);
    }

    #[test]
    fn test_add() {
        let node = Rc::new(AstNode::Add(Rc::new(AstNode::IntLiteral(1)), Rc::new(AstNode::IntLiteral(2))));
        assert_eq!(evaluate(node), 3);
    }

    #[test]
    fn test_subtract() {
        let node = Rc::new(AstNode::Subtract(Rc::new(AstNode::IntLiteral(1)), Rc::new(AstNode::IntLiteral(2))));
        assert_eq!(evaluate(node), -1);
    }

    #[test]
    fn test_multiply() {
        let node = Rc::new(AstNode::Multiply(Rc::new(AstNode::IntLiteral(2)), Rc::new(AstNode::IntLiteral(3))));
        assert_eq!(evaluate(node), 6);
    }

    #[test]
    fn test_divide() {
        let node = Rc::new(AstNode::Divide(Rc::new(AstNode::IntLiteral(6)), Rc::new(AstNode::IntLiteral(2))));
        assert_eq!(evaluate(node), 3);
    }

    #[test]
    // (1 + 2) * (3 - 6) = -9
    fn test_complex_expression() {
        let node = Rc::new(AstNode::Multiply(
            Rc::new(AstNode::Add(Rc::new(AstNode::IntLiteral(1)), Rc::new(AstNode::IntLiteral(2)))),
            Rc::new(AstNode::Subtract(Rc::new(AstNode::IntLiteral(3)), Rc::new(AstNode::IntLiteral(6))))
        ));
        assert_eq!(evaluate(node), -9);
    }
}

#[cfg(test)]
mod all_tests {
    use super::*;

    #[test]
    fn test_tokenize_all() {
        let tokens = tokenize_all("(1 + 2) * (3 - 6)");
        assert_eq!(tokens.len(), 12);
        assert_eq!(tokens[0], Token::OpenParen);
        assert_eq!(tokens[1], Token::Int(1));
        assert_eq!(tokens[2], Token::Operator("+"));
        assert_eq!(tokens[3], Token::Int(2));
        assert_eq!(tokens[4], Token::ClosedParen);
        assert_eq!(tokens[5], Token::Operator("*"));
        assert_eq!(tokens[6], Token::OpenParen);
        assert_eq!(tokens[7], Token::Int(3));
        assert_eq!(tokens[8], Token::Operator("-"));
        assert_eq!(tokens[9], Token::Int(6));
        assert_eq!(tokens[10], Token::ClosedParen);
        assert_eq!(tokens[11], Token::InputEnd);
    }

    #[test]
    fn test_parse() {
        let tokens = tokenize_all("(1 + 2) * (3 - 6)");
        let ast = parse(tokens);
        assert_eq!(ast, AstNode::Multiply(
            Rc::new(AstNode::Add(Rc::new(AstNode::IntLiteral(1)), Rc::new(AstNode::IntLiteral(2)))),
            Rc::new(AstNode::Subtract(Rc::new(AstNode::IntLiteral(3)), Rc::new(AstNode::IntLiteral(6))))
        ));
    }

    #[test]
    fn test_interpret() {
        assert_eq!(interpret("(1 + 2) * (3 - 6)"), -9);
        assert_eq!(interpret("(1 + 3) * (4 * 2)"), 32);
        assert_eq!(interpret("x = (1 + 3) * (4 * 2)"), 32);
    }

    #[test]
    fn test_interpret_expressions() {
        assert_eq!(interpret_expressions(vec!["x = 1", "y = 2", "x + y"]), 3);
    }
}

fn main() {
    tokenize_all("(1 + 2) * (3 - 6)");
    println!("1 = {}", evaluate(Rc::new(AstNode::IntLiteral(1))));
}

