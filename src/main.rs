use std::{time::Instant, collections::HashMap, hash::Hash, ops::{BitAnd, Not}};

#[derive(Debug, Clone)]
enum Token {
    Chip(String),
    ChipIO(String, String), // CHIP_NAME[.CHIP_OUTPUT] - Defaults to the first output
    Input(String),
    IO(String, String),
    Output(String),
    True,
    False,
    Assign,
    LParen,
    RParen,
    Comma,
    Expression(Vec<Token>),
}


fn tokenize(code: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    let mut currentWord: String = String::new();
    let mut isComment = false;
    for c in code.chars() {
        if isComment {
            if c == '\n' {
                isComment = false;
            }
            continue;
        }
        if c.is_whitespace() {
            continue;
        }
        if c == '(' || c == ')' || c == '=' || c == ',' {
            if currentWord.len() > 0 {
                result.push(currentWord.clone());
            }
            currentWord.clear();
            result.push(c.into());
            continue;
        }
        currentWord += &c.to_string();
        if currentWord == "//" {
            isComment = true;
            currentWord.clear();
        }
    }
    if currentWord.len() > 0 {
        result.push(currentWord);
    }

    result
}

fn lex(tokens: &Vec<String>) -> Vec<Token>{
    let mut result: Vec<Token> = Vec::new();
    let mut hasOutput = false;
    let mut assigning = false;
    let mut parenCount = 0;

    for tok in tokens {
        if !hasOutput {
            if tok == "(" || tok == ")" || tok == "," {
                panic!("Syntax error. Unexpected token: {}, expected output name", tok);
            }
            result.push(Token::Output(tok.into()));
            hasOutput = true;
        }
        else {
            if !assigning {
                if tok != "=" {
                    panic!("Unexpected token {}, expected '='", tok);
                }
                result.push(Token::Assign);
                assigning = true
            }
            else {
                // RHS
                if tok == ")" {
                    result.push(Token::RParen);
                    parenCount -= 1;
                    if parenCount == 0 {
                        // End of current statement
                        assigning = false;
                        hasOutput = false;
                    }
                }
                else if tok == "(" {
                    // The previous token which was misidentified as an input is now a chip
                    if result.len() == 0 || match result.last().unwrap() {
                        Token::Chip(_) => true,
                        Token::Input(_) => false,
                        Token::Output(_) => true,
                        Token::True => true,
                        Token::False => true,
                        Token::Assign => true,
                        Token::LParen => true,
                        Token::RParen => true,
                        Token::Comma => true,
                        Token::Expression(_) => true,
                        Token::IO(_, _) => true,
                        Token::ChipIO(_, _) => true,
                    } {
                        panic!("Unexpected token: {}", tok);
                    }
                    
                    // Can actually convert previous from input to chip now
                    let last_token = result.last().unwrap().clone();
                    if let Token::Input(x) = last_token {
                        // This will always be the case
                        result.pop();
                        result.push(Token::Chip(x.into()));
                        result.push(Token::LParen);
                    }
                    parenCount += 1;
                }
                else if tok == "," {
                    result.push(Token::Comma);
                }
                else {
                    if tok.to_lowercase() == "true" || tok == "1" {
                        result.push(Token::True);
                    }
                    else if tok.to_ascii_lowercase() == "false" || tok == "0" {
                        result.push(Token::False);
                    }
                    else {
                        result.push(Token::Input(tok.into()));
                    }
                }
            }
        }
    }

    result
}

fn lex2(tokens: &[Token]) -> Vec<Token> {
    let mut result = Vec::<Token>::new();
    let mut current_tokens = vec![];

    for tok in tokens {
        if let Token::Output(_) = tok {
            if current_tokens.len() > 0 {
                // Flush current tokens as expression
                result.push(parse_expressions(&current_tokens));
                current_tokens.clear();
            }
            result.push(tok.clone());
        }
        else if let Token::Assign = tok {
            // result.push(tok.clone());
            // Clear the current tokens
            current_tokens.clear();
        }
        else {
            // Otherwise add to current tokens
            if let Token::Chip(x) = tok {
                if x.contains('.') {
                    current_tokens.push(Token::ChipIO(x.split('.').nth(0).unwrap().into(), x.split('.').nth(1).unwrap().into()));
                }
                else {
                    current_tokens.push(tok.clone());
                }
            }
            else {
                current_tokens.push(tok.clone());
            }
        }
    }
    if current_tokens.len() > 0 {
        // Flush current tokens as expression
        result.push(parse_expressions(&current_tokens));
    }

    result
}

fn lex_final(tokens: &[Token]) -> Vec<Token> {
    let mut result = Vec::<Token>::new();
    for tok in tokens {
        match tok {
            Token::Chip(x) => {
                if x.contains('.') {
                    result.push(Token::ChipIO(x.split('.').nth(0).unwrap().into(), x.split('.').nth(1).unwrap().into()));
                }
                else {
                    result.push(tok.clone())
                }
            },
            Token::ChipIO(_, _) => result.push(tok.clone()),
            Token::Input(_) => result.push(tok.clone()),
            Token::IO(_, _) => result.push(tok.clone()),
            Token::Output(_) => result.push(tok.clone()),
            Token::True => result.push(tok.clone()),
            Token::False => result.push(tok.clone()),
            Token::Assign => result.push(tok.clone()),
            Token::LParen => result.push(tok.clone()),
            Token::RParen => result.push(tok.clone()),
            Token::Comma => result.push(tok.clone()),
            Token::Expression(_) => result.push(tok.clone()),
        }
    }
    result
}

fn parse_expressions(tokens: &[Token]) -> Token {
    // Base cases, we have just an input, or true, or false
    if tokens.len() == 1 {
        let tok = tokens.first().unwrap();
        match tok {
            Token::Chip(_) => {},
            Token::Input(x) => return Token::IO(x.split(':').nth(0).unwrap().into(), x.split(':').nth(1).unwrap().into()),
            Token::Output(_) => {},
            Token::True => return tok.clone(),
            Token::False => return tok.clone(),
            Token::Assign => {},
            Token::LParen => {},
            Token::RParen => {},
            Token::Comma => {},
            Token::Expression(_) => return tok.clone(),
            Token::IO(_, _) => {},
            Token::ChipIO(_, _) => {},
        }
    }
    // TODO: Ensure parens match closing
    // TODO: Check the number of tokens etc
    let this_chip = tokens.first().unwrap();
    let mut input_expressions: Vec<Token> = vec![this_chip.clone()];
    let mut p_count = 0;
    let mut current_expression = Vec::<Token>::new();

    for tok in tokens {
        if let Token::LParen = tok {
            p_count += 1;
            current_expression.push(tok.clone());
            if p_count == 1 {
                // Refresh the current expression
                current_expression.clear();
            }
            continue;
        }

        if let Token::RParen = tok {
            p_count -= 1;
            current_expression.push(tok.clone());
            if p_count == 0 {
                // Refresh the current expression
                input_expressions.push(parse_expressions(&current_expression));
                current_expression.clear();
            }
            continue;
        }

        if p_count == 1 {
            // We are on the current chip's input level
            if let Token::Comma = tok {
                // We can flush the current expression
                input_expressions.push(parse_expressions(&current_expression));
                current_expression.clear();
                continue;
            }
        }

        // Otherwise we can add to the current expression if it is inside the chip's parentheses (p_count > 0)
        if p_count > 0 {
            current_expression.push(tok.clone());
        }
    }

    if input_expressions.len() == 1 {
        let tok = input_expressions.first().unwrap();
        if let Token::Input(x) = tok {
            return Token::IO(x.split(':').nth(0).unwrap().into(), x.split(':').nth(1).unwrap().into());
        }
    }

    // We can now return an expression in the form <CHIP, Inputs>
    return Token::Expression(input_expressions);
}

fn parse(code: &str) -> Vec<Token> {
    lex_final(&lex2(&lex(&tokenize(&code))))
}

fn get_first_output(out: &HashMap<String, u8>) -> u8 {
    *out.get(out.keys().nth(0).unwrap()).unwrap_or(&0)
}

trait Executable {
    fn eval(&self, code: Vec<Token>, inputs: &HashMap<String, u8>) -> HashMap<String, u8>;
}

struct ChipEvaluator {
    chips: HashMap<String, Vec<Token>>
}

impl ChipEvaluator {
    fn new() -> Self {
        Self {chips: HashMap::new()}
    }

    fn load_chip(&mut self, chip_name: &str, code: &Vec<Token>)  {
        self.chips.insert(chip_name.to_string(), code.clone());
    }
}

fn NAND(inputs: &HashMap<String, u8>) ->  HashMap<String,u8> {
    let mut output: HashMap<String,u8> = HashMap::new();
    let result = !(*inputs.get("a").unwrap_or(&0) & *inputs.get("b").unwrap_or(&0));
    output.insert(String::from("out"), result);
    output
}

impl Executable for ChipEvaluator {
    fn eval(&self, code: Vec<Token>, inputs: &HashMap<String, u8>) -> HashMap<String, u8> {
        let mut output = HashMap::<String, u8>::new();
        
        let mut token_iter = code.iter();
        let mut current_token = token_iter.next();
        let mut current_out_name: String = String::new();
        while !current_token.is_none() {
            // Handle case of outputting
            match current_token.unwrap()  {
                Token::Chip(_) => {},
                Token::ChipIO(_, _) => {},
                Token::Input(_) => todo!(),
                Token::IO(_, _) => todo!(),
                Token::Output(out) => {current_out_name = out.clone();},
                Token::True => {output.insert(String::from("OUT"), 1 as u8);},
                Token::False => {output.insert(String::from("OUT"), 0 as u8);},
                Token::Assign => {},
                Token::LParen => {},
                Token::RParen => {},
                Token::Comma => {},
                Token::Expression(e_codes) => {
                    let mut ec_iter = e_codes.iter();
                    let e_chip =  ec_iter.next().unwrap();
                    let mut e_inputs = HashMap::<String,u8>::new();
                    let mut current_input_param = 'a' as u8;
                    for input_token in ec_iter {
                        // Handle IO
                        if let Token::IO(x, y) = input_token {
                            e_inputs.insert(x.clone(), *inputs.get(y).unwrap_or(&0));
                        }
                        // Handle expressions as inputs to current expression
                        if let Token::Expression(i_toks)   = input_token {
                            e_inputs.insert((current_input_param as char).to_string(), get_first_output(&self.eval(i_toks.to_vec(), &inputs)));
                        }

                        current_input_param += 1;
                    }

                    // Handle normal CHIPs
                    if let Token::Chip(chip_name) = e_chip {
                        // Handle NAND CHIP
                        if chip_name == "NAND" {
                            let e_result = get_first_output(&NAND(&e_inputs));
                            if current_out_name.len() > 0 {
                                output.insert(current_out_name.clone(), e_result);
                                current_out_name.clear();
                            }
                            else  {
                                output.insert(String::from("out"), e_result);
                            }
                        }
                        // Handle other chips
                        else {
                            let chip_instructions = self.chips.get(chip_name).unwrap();
                            let e_result =  get_first_output(&self.eval(chip_instructions.to_vec(), &e_inputs));
                            if current_out_name.len() > 0 {
                                output.insert(current_out_name.clone(), e_result);
                                current_out_name.clear();
                            }
                            else  {
                                output.insert(String::from("out"), e_result);
                            }
                        }
                    }

                    // Handle CHIPIO chips
                    else if let Token::ChipIO(chip_name, chip_out) = e_chip {
                        // Handle NAND CHIP
                        if chip_name == "NAND" {
                            let e_result = *NAND(&e_inputs).get(chip_out).unwrap_or(&0);
                            if current_out_name.len() > 0 {
                                output.insert(current_out_name.clone(), e_result);
                                current_out_name.clear();
                            }
                            else  {
                                output.insert(String::from("out"), e_result);
                            }
                        }
                        // Handle other chips
                        else {
                            let chip_instructions = self.chips.get(chip_name).unwrap();
                            let e_result =  *self.eval(chip_instructions.to_vec(), &e_inputs).get(chip_out).unwrap();
                            if current_out_name.len() > 0 {
                                output.insert(current_out_name.clone(), e_result);
                                current_out_name.clear();
                            }
                            else  {
                                output.insert(String::from("out"), e_result);
                            }
                        }
                    }
                },
            }


            current_token = token_iter.next();
        }

        output
    }
}


fn main() {
    // println!("{:#?}", lex2(&lex(&tokenize("// This is a comment\nOUT1 = NAND(a, b)\nXOR=AND(OR(A,B), NAND(A,B))"))));
    // println!("{:?}", parse("OUT = NAND.out1(a: a, OR(b: b,c: c))\nOUT2=XOR(a: x,b: y)"));
    let cpu = ChipEvaluator::new();
    let mut inputs = HashMap::<String, u8>::new();
    inputs.insert(String::from("a"), 0b11111111);
    inputs.insert(String::from("b"), 0b01010101);
    println!("Result of NAND on 1 and 2: {:#08b}", get_first_output(&cpu.eval(parse("OUT = NAND(a: a, b: b)"), &inputs)));
}

/*
CHIP_A
INPUTS: I1, I2, ..., IN
OUTPUTS: O1, O2, ..., ON

CHIP_B
INPUTS: X1, X2, ..., XN
OUTPUTS: Y1, Y2, ..., YN

CHIP_B Example Def: Y1 = CHIP_A.O2(I1:X1, I2:X2, ...)
*/