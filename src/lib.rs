use once_cell::sync::Lazy;
use regex::Regex;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::{fmt, rc::Rc};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Token {
    Boolean(bool),
    Integer(i64),
    String(String),
    Unary(char),
    Binary(char),
    If,
    Lambda(i64),
    Var(i64),
}

pub type ExprPtr = Rc<RefCell<Expr>>;

pub fn as_ptr(e: Expr) -> ExprPtr {
    return Rc::new(RefCell::new(e));
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr {
    Boolean(bool),
    Integer(i64),
    String(String),
    Unary(char, ExprPtr),
    Binary(char, ExprPtr, ExprPtr),
    If(ExprPtr, ExprPtr, ExprPtr),
    Lambda(i64, ExprPtr),
    Var(i64),
}

pub fn short_str(expr: &Expr) -> String {
    if is_basic(expr) {
        return format!("{:?}", expr);
    }
    match expr {
        Expr::Unary(op, _) => format!("Unary({})", op),
        Expr::If(_, _, _) => format!("If"),
        Expr::Var(x) => format!("Var(x{})", x),
        Expr::Lambda(x, _) => format!("Lambda(x{})", x),
        Expr::Binary(op, _, _) => format!("Binary({})", op),
        _ => "N/A expr".to_string(),
    }
}

pub fn unwrap_binary(e: ExprPtr) -> (char, ExprPtr, ExprPtr) {
    if let Expr::Binary(op, a, b) = &*e.clone().borrow() {
        return (*op, a.clone(), b.clone());
    }
    panic!("Expected Expr::Binary, got {:?}", e);
}

pub fn unwrap_bool(e: &Expr) -> bool {
    if let Expr::Boolean(x) = e {
        return *x;
    }
    panic!("Expected Expr::Boolean from expression, got {:?}", e);
}

pub fn unwrap_i64(e: &Expr) -> i64 {
    if let Expr::Integer(x) = e {
        return *x;
    }
    print_ast(as_ptr(e.clone()));
    panic!("Expected Expr::Integer from expression, got {:?}", e);
}

pub fn unwrap_string(e: &Expr) -> String {
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
    // println!("Apply Impl, f={}, x{}, v={}", short_str(&expr_ptr.borrow()), target_x, short_str(&value.borrow()));
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
            return as_ptr(Expr::Binary(op, new_a, new_b));
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

pub fn eval(expr_ptr: ExprPtr) -> ExprPtr {
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
                        // println!("Apply, f={}, x{}, v={}", short_str(&expr_c.borrow()), x_value, short_str(&expr_b.borrow()));
                        let res = apply(expr_c.clone(), *x_value, expr_b.clone());
                        // println!("Got: {:?}", res);
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
                        '=' => match (a, b) {
                            (Expr::Integer(va), Expr::Integer(vb)) => Expr::Boolean(va == vb),
                            (Expr::String(va), Expr::String(vb)) => Expr::Boolean(va == vb),
                            (Expr::Boolean(va), Expr::Boolean(vb)) => Expr::Boolean(va == vb),
                            _ => {
                                panic!("Unsupported comparison a={:?} vs b={:?}", a, b)
                            }
                        },

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

pub fn base94_string_to_int(chars: &[char]) -> i64 {
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
pub fn int_to_base94_string(mut x: i64) -> String {
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

pub fn decode_string(chars: &[char]) -> String {
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

pub fn encode_string(s: String) -> Vec<char> {
    let mut res = Vec::new();
    for c in s.chars() {
        let idx = TRANSLATION_TABLE_REVERSE.get(&c).unwrap();
        let code = (*idx as u8 + 33) as char;
        res.push(code);
    }
    res
}

pub fn parse_token(s: String) -> Token {
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

pub fn tokenize(s: String) -> Vec<Token> {
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

pub fn create_ast(tokens: &[Token], idx: usize) -> (Expr, usize) {
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

pub fn parse_into_ast(s: String) -> ExprPtr {
    let tokens = tokenize(s);
    // println!("{:?}", tokens);
    let (expr, _) = create_ast(&tokens, 0);
    as_ptr(expr)
}

pub fn print_ast(e_ptr: ExprPtr) {
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
        // let delta = 0 as usize;
        let ref e = *e_ptr.borrow();
        let shift = " ".repeat(0 * indent);
        match e {
            Expr::Unary(op, expr) => {
                println!("{} [{}] Unary {}", shift, indent, op);
                self.print_ast_impl(expr.clone(), indent + delta, counter + 1);
            }
            Expr::Binary(op, expr_a, expr_b) => {
                println!("{} [{}] Binary {} ", shift, indent, op);
                self.print_ast_impl(expr_a.clone(), indent + delta, counter + 1);
                self.print_ast_impl(expr_b.clone(), indent + delta, counter + 1);
            }
            Expr::Lambda(x, expr) => {
                println!("{} [{}] Lambda x{}", shift, indent, x);
                self.print_ast_impl(expr.clone(), indent + delta, counter + 1);
            }
            Expr::Var(x) => {
                println!("{} [{}] x{}", shift, indent, x);
            }
            Expr::If(expr_a, expr_b, expr_c) => {
                println!("{} [{}] If", shift, indent);
                self.print_ast_impl(expr_a.clone(), indent + delta, counter + 1);
                self.print_ast_impl(expr_b.clone(), indent + delta, counter + 1);
                self.print_ast_impl(expr_c.clone(), indent + delta, counter + 1);
            }
            _ => {
                let str = format!("{:?}", e);
                println!("{} [{}] {}", shift, indent, str);
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

    #[test]
    fn test_unary_operators() {
        let test = |a, b| -> () {
            assert_eq!(eval_example(a), b);
        };
        test("U- I$", Expr::Integer(-3));
        test("U! T", Expr::Boolean(false));
        test("U# S4%34", Expr::Integer(15818151));
        test("U$ I4%34", Expr::String("test".to_string()));
    }

    #[test]
    fn test_binary_operators() {
        let test = |a, b| -> () {
            assert_eq!(eval_example(a), b);
        };
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
        test("B+ I# I$", Expr::Integer(5));
        test("B- I$ I#", Expr::Integer(1));
        test("B* I$ I#", Expr::Integer(6));
        test("B/ U- I( I#", Expr::Integer(-3));
        test("B% U- I( I#", Expr::Integer(-1));
        test("B< I$ I#", Expr::Boolean(false));
        test("B> I$ I#", Expr::Boolean(true));
        test("B= I$ I#", Expr::Boolean(false));
        test("B| T F", Expr::Boolean(true));
        test("B& T F", Expr::Boolean(false));
        test("B. S4% S34", Expr::String("test".to_string()));
        test("BT I$ S4%34", Expr::String("tes".to_string()));
        test("BD I$ S4%34", Expr::String("t".to_string()));
    }

    #[test]
    fn test_if_operator() {
        assert_eq!(
            eval_example("? B> I# I$ S9%3 S./"),
            Expr::String("no".to_string())
        );
    }

    #[test]
    fn test_lambda_operator() {
        assert_eq!(
            eval_example(r#"B$ B$ L# L$ v# B. SB%,,/ S}Q/2,$_ IK"#),
            Expr::String("Hello World!".to_string())
        );
    }

    #[test]
    fn test_eval() {
        let (res, steps) = eval_example_impl(
            r#"B$ B$ L" B$ L# B$ v" B$ v# v# L# B$ v" B$ v# v# L" L# ? B= v# I! I" B$ L$ B+ B$ v" v$ B$ v" v$ B- v# I" I%"#,
        );
        assert_eq!(res, Expr::Integer(16));
        assert_eq!(steps, 21);
    }

    #[test]
    fn test_language_test() {
        let example = fs::read_to_string("language_test.txt").unwrap();
        let expected = "Self-check OK, send `solve language_test 4w3s0m3` to claim points for it";
        assert_eq!(eval_example(&example), Expr::String(expected.to_string()));
    }
}

const OP_LIMIT: usize = 10_000_000;
const DEBUG: bool = true;

pub fn eval_expr(mut expr_ptr: ExprPtr) -> (Expr, usize) {
    for step in 0..OP_LIMIT {
        if DEBUG {
            println!("step = {}, AST:", step);
            print_ast(expr_ptr.clone());

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Press enter to continue:");
        }

        if is_basic(&*expr_ptr.clone().borrow()) {
            let borrowed = expr_ptr.borrow();
            return (borrowed.clone(), step);
        }
        let next_ptr = eval(expr_ptr);
        expr_ptr = next_ptr;
    }
    let borrowed = expr_ptr.borrow();
    (borrowed.clone(), OP_LIMIT)
}

pub fn eval_example_impl(example: &str) -> (Expr, usize) {
    let expr_ptr = parse_into_ast(example.to_string());
    eval_expr(expr_ptr)
}

pub fn eval_example(example: &str) -> Expr {
    let (res, _) = eval_example_impl(example);
    res
}

pub fn print_ast_from_str(s: &str) {
    let expr = parse_into_ast(s.to_string());
    println!("Str: '{}'", s);
    println!("AST:");
    print_ast(expr);
}
