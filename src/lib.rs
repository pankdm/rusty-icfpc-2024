use once_cell::sync::Lazy;
use regex::Regex;

use reqwest::Client;
use reqwest::Error;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::{fmt, rc::Rc};
use std::fs;
use std::io::{self, Write};

#[derive(Clone, PartialEq, Eq, Debug)]
enum Token {
    Boolean(bool),
    Integer(i64),
    String(String),
    Unary(char),
    Binary(char),
    If,
    Lambda(i64),
    Var(i64),
}

type ExprPtr = Rc<RefCell<Expr>>;

fn as_ptr(e: Expr) -> ExprPtr {
    return Rc::new(RefCell::new(e));
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Expr {
    Boolean(bool),
    Integer(i64),
    String(String),
    Unary(char, ExprPtr),
    Binary(char, ExprPtr, ExprPtr),
    If(ExprPtr, ExprPtr, ExprPtr),
    Lambda(i64, ExprPtr),
    Var(i64),
}

fn short_str(expr: &Expr) -> String {
    if is_basic(expr) {
        return format!("{:?}", expr);
    }
    match expr {
        Expr::Unary(op, _) => format!("Unary({})", op),
        Expr::If(_, _ , _) => format!("If"),
        Expr::Var(x) => format!("Var(x{})", x),
        Expr::Lambda(x, _) => format!("Lambda(x{})", x),
        Expr::Binary(op, _, _) => format!("Binary({})", op),
        _ => "N/A expr".to_string(),
    }
}

fn unwrap_bool(e: &Expr) -> bool {
    if let Expr::Boolean(x) = e {
        return *x;
    }
    panic!("Expected Expr::Boolean from expression, got {:?}", e);
}

fn unwrap_i64(e: &Expr) -> i64 {
    if let Expr::Integer(x) = e {
        return *x;
    }
    print_ast(as_ptr(e.clone()));
    panic!("Expected Expr::Integer from expression, got {:?}", e);
}

fn unwrap_string(e: &Expr) -> String {
    if let Expr::String(x) = e {
        return x.clone();
    }
    panic!("Expected Expr::String from expression, got {:?}", e);
}

fn to_chars(s: String) -> Vec<char> {
    s.chars().collect()
}

fn to_string(s: &[char]) -> String {
    s.iter().collect()
}

fn is_basic(e: &Expr) -> bool {
    match e {
        Expr::Boolean(_) | Expr::Integer(_) | Expr::String(_) => true,
        _ => false,
    }
}

fn apply(expr_ptr: ExprPtr, target_x: i64, value: ExprPtr) -> ExprPtr {
    println!("Apply Impl, f={}, x{}, v={}", short_str(&expr_ptr.borrow()), target_x, short_str(&value.borrow()));
    {
        let mut expr = expr_ptr.as_ref().borrow_mut();
        if is_basic(&*expr) {
            return expr_ptr.clone();
        } else if let Expr::Var(x) = *expr {
            if x == target_x {
                return value;
            }
        } else if let Expr::Unary(op, ref mut a) = expr.clone() {
            let new_a = apply(a.clone(), target_x, value);
            // *a = new_a;
            // println!("updated: {:?}", expr);
            return as_ptr(Expr::Unary(op, new_a));
        } else if let Expr::Binary(op, ref a, ref b) = expr.clone() {
            let new_a = apply(a.clone(), target_x, value.clone());
            let new_b = apply(b.clone(), target_x, value.clone());

            // *a = new_a;
            // *b = new_b;
            return as_ptr(Expr::Binary(op, new_a, new_b))
            // println!("updated: {:?}", expr);
        } else if let Expr::If(ref mut a, ref mut b, ref mut c) = expr.clone() {
            let new_a = apply(a.clone(), target_x, value.clone());
            let new_b = apply(b.clone(), target_x, value.clone());
            let new_c = apply(c.clone(), target_x, value.clone());

            // *a = new_a;
            // *b = new_b;
            // *c = new_c;
            return as_ptr(Expr::If(new_a, new_b, new_c));
        } else if let Expr::Lambda(x, ref mut a) = *expr {
            if x == target_x {
                // Don't go further if lambda captures the same variable
                return expr_ptr.clone();
            }
            let new_a = apply(a.clone(), target_x, value.clone());
            return as_ptr(Expr::Lambda(x, new_a));
        } else {
            panic!("[apply] Unimplemented: {:?}", expr);
        }
    }
    // println!("  Result >> {:?}", expr_ptr);

    expr_ptr
}

fn eval(expr_ptr: ExprPtr) -> ExprPtr {
    let ref e = *expr_ptr.borrow();
    if is_basic(e) {
        return expr_ptr.clone();
    }

    match e {
        Expr::Unary(op, expr_a) => {
            let res = {
                let a_ptr = eval(expr_a.clone());
                let ref a = *a_ptr.borrow();
                match op {
                    '-' => Expr::Integer(-unwrap_i64(a)),
                    '!' => Expr::Boolean(!unwrap_bool(a)),
                    '#' => {
                        let s = unwrap_string(a);
                        let chars = encode_string(s);
                        let x = base94_string_to_int(&chars);
                        Expr::Integer(x)
                    }
                    '$' => {
                        let x = unwrap_i64(a);
                        let s = int_to_base94_string(x);
                        let s_chars: Vec<char> = s.chars().collect();
                        let s_decoded = decode_string(&s_chars);
                        Expr::String(s_decoded)
                    }
                    _ => panic!("Unexpected op: {}", op),
                }
            };
            as_ptr(res)
        }
        Expr::Binary(op, expr_a, expr_b) => {
            let res = {
                if *op == '$' {
                    let ref a = *expr_a.borrow();
                    if let Expr::Lambda(x_value, expr_c) = a {
                        // When the first argument of the binary application operator $ evaluates to a lambda abstraction,
                        // the second argument of the application is assigned to that variable.
                        println!("Apply, f={}, x{}, v={}", short_str(&expr_c.borrow()), x_value, short_str(&expr_b.borrow()));
                        let res = apply(expr_c.clone(), *x_value, expr_b.clone());
                        println!("Got: {:?}", res);
                        return res;
                    } else {
                        // Otherwise nothing happens and we proceed to next
                    }
                }
                let a_ptr = eval(expr_a.clone());
                let b_ptr = eval(expr_b.clone());
                
                let a_ptr_copy = a_ptr.clone();
                let b_ptr_copy = b_ptr.clone();

                let ref a = *a_ptr.borrow();
                let ref b = *b_ptr.borrow();
                if is_basic(a) && is_basic(b) {
                match op {
                    '+' => Expr::Integer(unwrap_i64(&a) + unwrap_i64(&b)),
                    '-' => Expr::Integer(unwrap_i64(&a) - unwrap_i64(&b)),
                    '*' => Expr::Integer(unwrap_i64(&a) * unwrap_i64(&b)),
                    '/' => Expr::Integer(unwrap_i64(&a) / unwrap_i64(&b)),
                    '%' => Expr::Integer(unwrap_i64(&a) % unwrap_i64(&b)),

                    '<' => Expr::Boolean(unwrap_i64(&a) < unwrap_i64(&b)),
                    '>' => Expr::Boolean(unwrap_i64(&a) > unwrap_i64(&b)),
                    '=' => {
                        match (a, b) {
                            (Expr::Integer(va), Expr::Integer(vb)) => {
                                Expr::Boolean(va == vb)
                            },
                            (Expr::String(va), Expr::String(vb)) => {
                                Expr::Boolean(va == vb)
                            },
                            (Expr::Boolean(va), Expr::Boolean(vb)) => {
                                Expr::Boolean(va == vb)
                            },
                            _ => {panic!("Unsupported comparison a={:?} vs b={:?}", a, b)},
                        }
                    }

                    '|' => Expr::Boolean(unwrap_bool(&a) || unwrap_bool(&b)),
                    '&' => Expr::Boolean(unwrap_bool(&a) && unwrap_bool(&b)),

                    '.' => Expr::String(unwrap_string(&a) + &unwrap_string(&b)),
                    'T' => {
                        let x = unwrap_i64(&a);
                        let s = unwrap_string(&b);
                        let chars = to_chars(s);
                        Expr::String(to_string(&chars[..x as usize]))
                    }
                    'D' => {
                        let x = unwrap_i64(&a);
                        let s = unwrap_string(&b);
                        let chars = to_chars(s);
                        Expr::String(to_string(&chars[x as usize..]))
                    }
                    '$' => Expr::Binary('$', a_ptr.clone(), b_ptr.clone()),
                    _ => panic!("Unexpected op: {}", op),
                }
                } else {
                    Expr::Binary(*op, a_ptr_copy, b_ptr_copy)
                }
            };
            as_ptr(res)
        }
        Expr::If(expr_a, expr_b, expr_c) => {
            let a_ptr = eval(expr_a.clone());
            let a_ptr_copy = a_ptr.clone();
            let ref a = *a_ptr.borrow();
            
            if is_basic(a) {
                let condition = unwrap_bool(&a);
                if condition {
                    let b = eval(expr_b.clone());
                    b
                } else {
                    let c = eval(expr_c.clone());
                    c
                }    
            } else {
                let res = Expr::If(a_ptr_copy, expr_b.clone(), expr_c.clone());
                as_ptr(res)
            }
        }
        _ => {
            panic!("[eval] Unsupported expression: {:?}", e)
        }
    }
}

fn base94_string_to_int(chars: &[char]) -> i64 {
    let mult = 94;
    let mut res = 0;
    for c in chars.iter() {
        res *= mult;
        let d = (*c as u8 - '!' as u8) as i64;
        res += d;
        // println!("d = {} -> {}", *c, d);
    }
    res
}
fn int_to_base94_string(mut x: i64) -> String {
    let mut res = Vec::new();
    while x > 0 {
        let d = x % 94;
        let c = (d as u8 + '!' as u8) as char;
        res.push(c);
        x = x / 94;
    }
    res.iter().rev().collect()
}

static TRANSLATION_TABLE: Lazy<Vec<char>> = Lazy::new(|| {
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!\"#$%&'()*+,-./:;<=>?@[\\]^_`|~ \n".chars().collect()
});

static TRANSLATION_TABLE_REVERSE: Lazy<HashMap<char, usize>> = Lazy::new(|| {
    let mut res = HashMap::new();
    for (idx, c) in TRANSLATION_TABLE.iter().enumerate() {
        res.insert(*c, idx);
    }
    res
});

fn decode_string(chars: &[char]) -> String {
    let mut res = Vec::new();
    for c in chars.iter() {
        let code = *c as u8;
        if code < 33 || code > 126 {
            panic!("Invalid code: {} ('{}')", code, *c);
        }
        let index = (*c as u8 - 33) as usize;
        res.push(TRANSLATION_TABLE[index]);
    }
    res.iter().collect()
}

fn encode_string(s: String) -> Vec<char> {
    let mut res = Vec::new();
    for c in s.chars() {
        let idx = TRANSLATION_TABLE_REVERSE.get(&c).unwrap();
        let code = (*idx as u8 + 33) as char;
        res.push(code);
    }
    res
}

fn parse_token(s: String) -> Token {
    let chars: Vec<char> = s.chars().collect();
    let indicator = chars[0];
    match indicator {
        'T' => Token::Boolean(true),
        'F' => Token::Boolean(false),
        'I' => {
            let value = base94_string_to_int(&chars[1..]);
            Token::Integer(value)
        }
        'S' => {
            let decoded = decode_string(&chars[1..]);
            return Token::String(decoded);
        }
        'U' => {
            let op = chars[1];
            return Token::Unary(op);
        }
        'B' => {
            let op = chars[1];
            return Token::Binary(op);
        }
        '?' => {
            return Token::If;
        }
        'L' => {
            let value = base94_string_to_int(&chars[1..]);
            Token::Lambda(value)
        }
        'v' => {
            let value = base94_string_to_int(&chars[1..]);
            Token::Var(value)
        }
        _ => {
            panic!(
                "[parse_token] Unknown indicator: '{}', while parsing '{}'",
                indicator, s
            );
        }
    }
}

fn tokenize(s: String) -> Vec<Token> {
    let mut res = Vec::new();
    let parts = split_string(&s);
    // println!("splitted string: {:?}", &parts);
    for part in parts {
        res.push(parse_token(part));
    }
    res
}

pub fn split_string(s: &String) -> Vec<String> {
    let re = Regex::new(r"\s+").unwrap();
    let parts = re.split(s).map(|s| s.to_string()).collect();
    return parts;
}

fn create_ast(tokens: &[Token], idx: usize) -> (Expr, usize) {
    if idx >= tokens.len() {
        panic!(
            "Create AST, out of bounds error: idx={}, len={}",
            idx,
            tokens.len()
        );
    }
    let token = &tokens[idx];

    match token {
        Token::Boolean(b) => (Expr::Boolean(*b), idx + 1),
        Token::Integer(x) => (Expr::Integer(*x), idx + 1),
        Token::String(s) => (Expr::String(s.clone()), idx + 1),
        Token::Unary(op) => {
            let (expr, next_idx) = create_ast(&tokens, idx + 1);
            (Expr::Unary(*op, as_ptr(expr)), next_idx)
        }
        Token::Binary(op) => {
            let (expr_a, idx_a) = create_ast(&tokens, idx + 1);
            let (expr_b, idx_b) = create_ast(&tokens, idx_a);
            (Expr::Binary(*op, as_ptr(expr_a), as_ptr(expr_b)), idx_b)
        }
        Token::If => {
            let (expr_a, idx_a) = create_ast(&tokens, idx + 1);
            let (expr_b, idx_b) = create_ast(&tokens, idx_a);
            let (expr_c, idx_c) = create_ast(&tokens, idx_b);
            (
                Expr::If(as_ptr(expr_a), as_ptr(expr_b), as_ptr(expr_c)),
                idx_c,
            )
        }
        Token::Lambda(x) => {
            let (expr, next_idx) = create_ast(&tokens, idx + 1);
            (Expr::Lambda(*x, as_ptr(expr)), next_idx)
        }
        Token::Var(x) => (Expr::Var(*x), idx + 1),
        _ => {
            panic!("[create_ast] Unsupported token: {:?}", token);
        }
    }
}

fn parse_into_ast(s: String) -> ExprPtr {
    let tokens = tokenize(s);
    println!("{:?}", tokens);
    let (expr, _) = create_ast(&tokens, 0);
    as_ptr(expr)
}

fn print_ast(e_ptr: ExprPtr) {
    let mut p = Printer::new();
    p.print_ast_impl(e_ptr, 0, p.counter);
}

struct Printer {
    counter: i32,
}

impl Printer {
    fn new() -> Printer {
        Printer { counter: 0 }
    }

    fn print_ast_impl(&mut self, e_ptr: ExprPtr, indent: usize, counter: i32) {
        let delta = 4 as usize;
        let ref e = *e_ptr.borrow();
        match e {
            Expr::Unary(op, expr) => {
                println!("{} [{}] Unary {}", " ".repeat(indent), indent, op);
                self.print_ast_impl(expr.clone(), indent + delta, counter + 1);
            }
            Expr::Binary(op, expr_a, expr_b) => {
                println!("{} [{}] Binary {} ", " ".repeat(indent), indent, op);
                self.print_ast_impl(expr_a.clone(), indent + delta, counter + 1);
                self.print_ast_impl(expr_b.clone(), indent + delta, counter + 1);
            }
            Expr::Lambda(x, expr) => {
                println!("{} [{}] Lambda x{}", " ".repeat(indent), indent, x);
                self.print_ast_impl(expr.clone(), indent + delta, counter + 1);
            }
            Expr::Var(x) => {
                println!("{} [{}] x{}", " ".repeat(indent), indent, x);
            }
            Expr::If(expr_a, expr_b, expr_c) => {
                println!("{} [{}] If", " ".repeat(indent), indent);
                self.print_ast_impl(expr_a.clone(), indent + delta, counter + 1);
                self.print_ast_impl(expr_b.clone(), indent + delta, counter + 1);
                self.print_ast_impl(expr_c.clone(), indent + delta, counter + 1);
            }
            _ => {
                let str = format!("{:?}", e);
                println!("{} [{}] {}", " ".repeat(indent), indent, str);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer1() {
        assert_eq!(parse_token("I/6".to_string()), Token::Integer(1337));
    }

    #[test]
    fn test_string1() {
        assert_eq!(
            parse_token("SB%,,/}Q/2,$_".to_string()),
            Token::String("Hello World!".to_string())
        );
    }
}

pub async fn run_repl_loop() {
    loop {
        // Prompt the user for input
        print!("\nEnter a string (or type 'exit' to quit):\n");
        io::stdout().flush().expect("Failed to flush stdout");

        // Read input from the user
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        // Trim the input to remove trailing newline
        let input = input.trim();

        // Check for exit condition
        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        // Run the foo function over the input
        let res = run_repl(input).await;
        println!("\n[debug] Result = {:?}", res);
    }
}

pub async fn run_repl(input: &str) -> Result<(), Error> {
    // let send = "get language_test".to_string();
    let send = input.to_string();
    let encoded  = encode_string(send);
    let encoded_prefix: String = encoded.iter().collect();
    let encoded_string = "S".to_string() + &encoded_prefix;
    println!("Hello, world!");
    println!("{:?}", parse_token(encoded_string.clone()));

    let client = Client::new();

    let response = client
        .post("http://localhost:8000/communicate")
        .body(encoded_string)
        .header(
            "Authorization",
            "Bearer 00000000-0000-0000-0000-000000000000",
        )
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        // let chars: Vec<char> = body.chars().collect();
        println!("Response Text: '{}'", body);
        let code = parse_into_ast(body);
        let evaled = eval(code);
        let expr = &*evaled.borrow();
        println!("Decoded: \n\n{}", unwrap_string(expr));
    } else {
        println!("HTTP Request failed with status: {}", response.status());
    }

    Ok(())
}

fn test_unary_operators() {
    let examples = ["U- I$", "U! T", "U# S4%34", "U$ I4%34"];
    for example in examples {
        let expr = parse_into_ast(example.to_string());
        println!("\nStr: '{}'", example);
        println!("AST: {:?}", expr);
        let result = eval(expr);
        println!("Eval: {:?}", result);
    }
}

fn test_binary_operators() {
    // + Integer addition B+ I# I$ -> 5
    // - Integer subtraction B- I$ I# -> 1
    // * Integer multiplication B* I$ I# -> 6
    // / Integer division (truncated towards zero) B/ U- I( I# -> -3
    // % Integer modulo B% U- I( I# -> -1
    // < Integer comparison B< I$ I# -> false
    // > Integer comparison B> I$ I# -> true
    // = Equality comparison, works for int, bool and string B= I$ I# -> false
    // | Boolean or B| T F -> true
    // & Boolean and B& T F -> false
    // . String concatenation B. S4% S34 -> "test"
    // T Take first x chars of string y BT I$ S4%34 -> "tes"
    // D Drop first x chars of string y BD I$ S4%34 -> "t"
    let examples = [
        "B+ I# I$",
        "B- I$ I#",
        "B* I$ I#",
        "B/ U- I( I#",
        "B% U- I( I#",
        "B< I$ I#",
        "B> I$ I#",
        "B= I$ I#",
        "B| T F",
        "B& T F",
        "B. S4% S34",
        "BT I$ S4%34",
        "BD I$ S4%34",
        // r#"B$ L# B$ L" B+ v" v" B* I$ I# v8"#,
    ];
    for example in examples {
        let expr = parse_into_ast(example.to_string());
        println!("\nStr: '{}'", example);
        println!("AST: {:?}", expr);
        let result = eval(expr);
        println!("Eval: {:?}", result);
    }
}

fn test_if_operator() {
    let example = "? B> I# I$ S9%3 S./";
    let expr = parse_into_ast(example.to_string());
    println!("\nStr: '{}'", example);
    println!("AST: {:?}", expr);
    let result = eval(expr);
    println!("Eval: {:?}", result);
}

fn print_ast_from_str(s: &str) {
    let expr = parse_into_ast(s.to_string());
    println!("Str: '{}'", s);
    println!("AST:");
    print_ast(expr);
}

fn test_lambda_operator() {
    let steps = [
        r#"B$ L# B$ L" B+ v" v" B* I$ I# v8"#,
        r#"B$ L" B+ v" v" B* I$ I#"#,
        "B+ B* I$ I# B* I$ I#",
        "B+ I' B* I$ I#",
        "B+ I' I'",
        "I-",
    ];
    for (idx, step) in steps.iter().enumerate() {
        println!("\nstep = {}", idx);
        print_ast_from_str(step);
    }
}

fn test_lambda_operator2() {
    // let tokens = [
    //     Token::Unary('-'),
    //     Token::Var(1),
    // ];
    // let tokens = [
    //     Token::Binary('$'),
    //     Token::Lambda(2),
    //     Token::Binary('$'),
    //     Token::Lambda(1),
    //     Token::Binary('+'),
    //     Token::Var(1),
    //     Token::Var(1),
    //     Token::Binary('*'),
    //     Token::Integer(3),
    //     Token::Integer(2),
    //     Token::Var(23),
    // ];
    let tokens = [
        Token::Binary('$'),
        Token::Lambda(2),
        Token::Binary('$'),
        Token::Integer(1),
        Token::Binary('$'),
        Token::Var(2),
        Token::Var(2),
        Token::Lambda(2),
        Token::Binary('$'),
        Token::Integer(1),
        Token::Binary('$'),
        Token::Var(2),
        Token::Var(2),
    ];
    let (expr, _) = create_ast(&tokens, 0);
    let mut expr_ptr = as_ptr(expr);
    println!("AST:");
    print_ast(expr_ptr.clone());

    for step in 1..5 {
        let next_ptr = eval(expr_ptr.clone());
        println!();
        println!("eval{} :", step);
        print_ast(next_ptr.clone());
        expr_ptr = next_ptr;
    }
}

fn test_lambda_operator3() {
    let examples = [
        // r#"B$ L# B$ L" B+ v" v" B* I$ I# v8"#,
        r#"B$ B$ L" B$ L# B$ v" B$ v# v# L# B$ v" B$ v# v# L" L# ? B= v# I! I" B$ L$ B+ B$ v" v$ B$ v" v$ B- v# I" I%"#,
    ];
    for (idx, example) in examples.iter().enumerate() {
        println!("\nexample = {}", example);
        print_ast_from_str(example);
        let mut expr_ptr = parse_into_ast(example.to_string());
        println!("AST:");
        print_ast(expr_ptr.clone());

        for step in 1..100 {
            let next_ptr = eval(expr_ptr.clone());
            println!();
            println!("eval{} :", step);
            print_ast(next_ptr.clone());
            expr_ptr = next_ptr;
        }
    }
}

pub fn test_language_test() {
    let example = fs::read_to_string("language_test.txt").unwrap();
    println!("\nexample = {}", example);
    print_ast_from_str(&example);
    let mut expr_ptr = parse_into_ast(example.to_string());
    println!("AST:");
    print_ast(expr_ptr.clone());
    for step in 1..10 {
        let next_ptr = eval(expr_ptr.clone());
        println!();
        println!("eval{} :", step);
        print_ast(next_ptr.clone());
        expr_ptr = next_ptr;
    }
}

