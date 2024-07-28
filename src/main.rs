use icfpc_2024::*;

use std::fs;

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

fn test_if_operator() {
    let example = "? B> I# I$ S9%3 S./";
    let expr = parse_into_ast(example.to_string());
    println!("\nStr: '{}'", example);
    println!("AST: {:?}", expr);
    let result = eval(expr);
    println!("Eval: {:?}", result);
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

#[tokio::main]
async fn main() {
    // test_unary_operators();
    // test_binary_operators();
    // test_if_operator();
    // test_lambda_operator();
    // test_lambda_operator3();
    // let _ = run_repl().await;
    test_language_test()
}
