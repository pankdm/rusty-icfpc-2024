use icfpc_2024::*;

use std::{collections::HashMap, fs};

use std::fs::File;
use std::io::Write;
use std::io::Result;

fn is_lambda(expr_ptr: ExprPtr) -> bool {
    let ref e = *expr_ptr.borrow();
    if let Expr::Binary('$', _, _) = e {
        return true;
    }
    return false;
}

fn decompose(expr_ptr: ExprPtr, level: usize, list: &mut Vec<ExprPtr>) {
    if level > 1 && is_lambda(expr_ptr.clone()) {
        let e_copy = as_ptr(expr_ptr.borrow().clone());
        let id: usize = {
            let mut res = list.len();
            for (id2, other) in list.iter().enumerate() {
                if *other.borrow() == *expr_ptr.borrow() {
                    res = id2;
                    break;
                }
            }
            res
        };
        {
            let ref mut e = *expr_ptr.borrow_mut();
            *e = Expr::String(format!("f{}", id));
        }

        if id == list.len() {
            list.push(e_copy.clone());
            decompose(e_copy.clone(), 0, list);
        }
        return;
    }

    let ref e = *expr_ptr.borrow();
    match e {
        Expr::Unary(_, a) => {
            decompose(a.clone(), level + 1, list);
        }
        Expr::Binary(_, a, b) => {
            decompose(a.clone(), level + 1, list);
            decompose(b.clone(), level + 1, list);
        }
        Expr::Lambda(_, a) => {
            decompose(a.clone(), level + 1, list);
        }
        Expr::If(a, b, c) => {
            decompose(a.clone(), level + 1, list);
            decompose(b.clone(), level + 1, list);
            decompose(c.clone(), level + 1, list);
        }
        _ => {}
    }
}

fn decompose_expr(expr_ptr: ExprPtr) -> Vec<ExprPtr> {
    let mut list = Vec::new();
    list.push(expr_ptr.clone());
    decompose(expr_ptr, 0, &mut list);
    list
}

fn rewrite_expr(expr_ptr: ExprPtr) -> ExprPtr {
    let ref e = *expr_ptr.borrow();
    if let Expr::Binary('+', a, b) = e {
        if a == b {
            return as_ptr(Expr::Binary('*', a.clone(), as_ptr(Expr::Integer(2))));
        } else {
            println!("a and b are not equal: {:?}, {:?}", a, b);
        }
        expr_ptr.clone()
    } else {
        match e {
            Expr::Lambda(x, a) => {
                let new_a = rewrite_expr(a.clone());
                return as_ptr(Expr::Lambda(*x, new_a));
            }
            Expr::Binary(op, a, b) => {
                let new_a = rewrite_expr(a.clone());
                let new_b = rewrite_expr(b.clone());
                return as_ptr(Expr::Binary(*op, new_a, new_b));
            }
            _ => return expr_ptr.clone(),
        }
    }
}

fn rewrite_expr_times(mut expr_ptr: ExprPtr, times: usize) -> ExprPtr {
    for i in 0..times {
        expr_ptr = rewrite_expr(expr_ptr.clone());
    }
    expr_ptr
}

mod tests {
    use super::*;

    #[test]
    fn test_rewrite() {
        let tokens = [
            Token::Lambda(0),
            Token::Binary('+'),
            Token::Binary('+'),
            Token::Var(0),
            Token::Var(0),
            Token::Binary('+'),
            Token::Var(0),
            Token::Var(0),
        ];
        let (expr, _) = create_ast(&tokens, 0);
        let new_expr = rewrite_expr_times(as_ptr(expr), 2);
        print_ast(new_expr);
    }
}

fn solve_eff_generic(name: String) {
    let example = fs::read_to_string(format!("problems/{}.txt", name)).unwrap();
    let mut expr_ptr = parse_into_ast(example.clone());
    // let (res, _) = eval_expr(expr_ptr.clone());
    // print_ast(expr_ptr.clone());

    let list = decompose_expr(expr_ptr);
    for (n, expr) in list.iter().enumerate() {
        println!("\n>>> f{}: ", n);
        print_ast(expr.clone());
    }

    let expr_copy = parse_into_ast(example);

    println!("\nFull example:");
    print_ast(expr_copy)
}

// == Eff 1 ==

// need to rewrite addition with multiplication
// Integer(17592186044416)
fn solve_eff1() {
    let example = fs::read_to_string("problems/1.txt").unwrap();
    let mut expr_ptr = parse_into_ast(example);
    expr_ptr = rewrite_expr_times(expr_ptr, 2);
    let (res, _) = eval_expr(expr_ptr);
    print_ast(as_ptr(res));
}

// == Eff 2 ==
// multiply by 0, doesn't matter
// Integer(2134)
fn solve_eff2() {
    let example = fs::read_to_string("problems/2.txt").unwrap();
    let mut expr_ptr = parse_into_ast(example);
    let (res, _) = eval_expr(expr_ptr);
    print_ast(as_ptr(res));
}

// == Eff 3 ==

// The same as previous but multiply by 1, so no need to compute fairly
// [0] Binary +
// [4] Integer(2134)
// [4] Binary *
//     [8] Binary +
//         [12] Integer(1)
//         [12] If
//             [16] Binary =
//                 [20] Integer(9345873498)
//                 [20] Integer(0)
//             [16] Integer(1)
//             [16] Binary +
//                 [20] Integer(1)
//                 [20] Binary $
//                     [24] Binary $
// The pattern is to +1 until reaching if-condition

//  [16] Lambda x3
//      [20] Lambda x4
//          [24] If
//              [28] Binary =
//                  [32] x4
//                  [32] Integer(0)
//              [28] Integer(1)
//              [28] Binary +
//                  [32] Integer(1)
//                  [32] Binary $
//                      [36] x3
//                      [36] Binary -
//                          [40] x4
//                          [40] Integer(1)

// if x4 == 0 {
//     1
// } else {
//     1 + f(x4 - 1)
// }

// Solution: Integer(9345875634)
fn solve_eff3() {
    let example = fs::read_to_string("problems/3.txt").unwrap();
    let mut expr_ptr = parse_into_ast(example);
    let (res, _) = eval_expr(expr_ptr);
    print_ast(as_ptr(res));
}

// == Eff 4 ==
// Solution: Integer(165580141)

// [8] Lambda x1
//     [12] Binary $
//         [16] Lambda x2
//             [20] Binary $
//                 [24] x1
//                 [24] Binary $
//                     [28] x2
//                     [28] x2
//         [16] Lambda x2
//             [20] Binary $
//                 [24] x1
//                 [24] Binary $
//                     [28] x2
//                     [28] x2
// Lx -> (Ly -> (x (y y))) (Lz -> (x (z z)))

//  [8] Lambda x3
//      [12] Lambda x4
//          [16] If
//              [20] Binary <
//                  [24] x4
//                  [24] Integer(2)
//              [20] Integer(1)
//              [20] Binary +
//                  [24] Binary $
//                      [28] x3
//                      [28] Binary -
//                          [32] x4
//                          [32] Integer(1)
//                  [24] Binary $
//                      [28] x3
//                      [28] Binary -
//                          [32] x4
//                          [32] Integer(2)

// if (x4 < 2) {
//     1
// } else {
//     f (x4 - 1) + f (x4 - 2)
// }
fn eff4(n: i32, cache: &mut HashMap<i32, i32>) -> i32 {
    if cache.contains_key(&n) {
        return cache[&n];
    }
    if n < 2 {
        1
    } else {
        let res = eff4(n - 1, cache) + eff4(n - 2, cache);
        cache.insert(n, res);
        return res;
    }
}

fn solve_eff4() {
    let mut cache = HashMap::new();
    println!("eff4 = {}", eff4(40, &mut cache));

    let example = fs::read_to_string("problems/4.txt").unwrap();
    let mut expr_ptr = parse_into_ast(example);
    let (res, _) = eval_expr(expr_ptr);
    print_ast(as_ptr(res));
}

// == Eff 5 ==
// first prime number x after 1000000 such that x + 1 is power of 2
// Solution: Integer(2147483647)

fn is_prime(x: i64) -> bool {
    for p in 2..x {
        if p * p > x {
            return true;
        }
        if x % p == 0 {
            return false;
        }
    }
    return true;
}

fn main_eff5() -> i64 {
    let mut x = 2 as i64;
    loop {
        x *= 2;

        if is_prime(x - 1) {
            println!("is_prime: {}", x - 1);
            if x > 1000000 {
                break;
            }
        }
    }
    x - 1
}

fn solve_eff5() {
    let res = main_eff5();
    println!("found = {}", res);
}

// == Eff 6 ==
// Solution: Integer(42)

// x > 30 && is_prime(fibo(x))
fn solve_eff6() {
    let mut x = 31;
    let mut cache = HashMap::new();
    loop {
        let f = fibo(x, &mut cache);
        println!("x={} -> f={}", x, f);
        if is_prime(f) {
            break;
        }
        x += 1;
    }
}

fn show_eff6() {
    let example = fs::read_to_string("problems/6.txt").unwrap();
    let mut expr_ptr = parse_into_ast(example);

    let list = decompose_expr(expr_ptr);
    for (n, expr) in list.iter().enumerate() {
        println!("\n>>> f{}: ", n);
        print_ast(expr.clone());
    }
}

fn fibo(n: i64, cache: &mut HashMap<i64, i64>) -> i64 {
    if cache.contains_key(&n) {
        return cache[&n];
    }
    if n < 2 {
        1
    } else {
        let res = fibo(n - 1, cache) + fibo(n - 2, cache);
        cache.insert(n, res);
        return res;
    }
}

// == Eff 7 ==

fn visit_single_sat3(expr_ptr: ExprPtr) -> Vec<i64> {
    let e = &*expr_ptr.borrow();
    match e {
        Expr::Binary('|', a, b) => {
            let mut res = Vec::new();
            let mut res_a = visit_single_sat3(a.clone());
            let mut res_b = visit_single_sat3(b.clone());
            res.append(&mut res_a);
            res.append(&mut res_b);
            return res;
        }
        Expr::Unary('!', a) => {
            let mut res = Vec::new();
            let res_a = visit_single_sat3(a.clone());
            assert_eq!(res_a.len(), 1);
            for x in res_a {
                res.push(-x);
            }
            return res;
        }
        Expr::Var(x) => {
            return vec![*x];
        }
        _ => { panic!("Unexpected expression: {:?}", e)}
    }
}

fn visit_sat3(expr_ptr: ExprPtr, res: &mut Vec<Vec<i64>>) {
    let e = &*expr_ptr.borrow();
    match e {
        Expr::Unary(_, a) => {
            let sat = visit_single_sat3(expr_ptr.clone());
            res.push(sat);
        }
        Expr::Binary(op, a, b) => {
            if *op == '|' {
                let sat = visit_single_sat3(expr_ptr.clone());
                res.push(sat);
            } else {
                visit_sat3(a.clone(), res);
                visit_sat3(b.clone(), res);    
            }
        }
        Expr::Lambda(_, a) => {
            visit_sat3(a.clone(), res);
        }
        Expr::If(a, b, c) => {
            visit_sat3(a.clone(), res);
            visit_sat3(b.clone(), res);
            visit_sat3(c.clone(), res);
        }
        Expr::Var(x) => {
            res.push(vec![*x]);
        }
        Expr::String(_) | Expr::Integer(_)=> {
            // ignore temporary strings
        }
        _ => {
            panic!("Unexpected expression: {:?}", e);
        }
    }

}

fn convert_to_sat3(expr_ptr: ExprPtr) -> Vec<Vec<i64>> {
    let mut res = Vec::new();
    let list = decompose_expr(expr_ptr);
    for (id, l) in list.iter().enumerate() {
        // println!("\nid = {}", id);
        // print_ast(l.clone());
    }
    let last_expr = list[list.len() - 2].clone();
    visit_sat3(last_expr, &mut res);
    res
}


fn get_sat_string(input: &Vec<Vec<i64>>) -> String {
    let total = input.len();
    let mut mx = 0;
    for vals in input {
        mx = mx.max(vals.iter().map(|x| {x.abs()}).max().unwrap());
    }
    format!("p cnf {} {}\n", mx, total)
}

fn solve_eff7() {
    let example = fs::read_to_string("problems/7.txt").unwrap();
    let expr_ptr = parse_into_ast(example);

    let sat3 = convert_to_sat3(expr_ptr);
    println!("{:?}", sat3);


    let mut file = File::create("solutions/7-minisat.txt").unwrap();
    file.write_all(get_sat_string(&sat3).as_bytes());
    for vals in sat3 {
        let mut new_vals = vals.clone();
        new_vals.push(0);
        let output: String = new_vals.iter().map(|x| {x.to_string()}).collect::<Vec<String>>().join(" ");
        file.write_all((output + "\n").as_bytes());
    }

    // let output = "-1 -2 -3 -4 -5 6 -7 -8 -9 10 -11 12 -13 -14 -15 -16 -17 -18 -19 -20 -21 22 -23 -24 25 26 -27 28 -29 -30 -31 -32 -33 -34 -35 36 -37 -38 -39 -40 0";
    let output = "1 -2 -3 -4 -5 6 -7 -8 -9 10 -11 12 -13 -14 15 -16 -17 -18 -19 -20 -21 22 -23 -24 25 26 -27 28 -29 -30 -31 -32 -33 -34 -35 36 -37 -38 -39 40 0";
    let parts = output.split(" ");
    let mut res = 0 as i64;
    for part in parts {
        let x: i64 = part.parse().unwrap();
        if x > 0 {
            let add = 1 << (x.abs() - 1);
            println!("x = {}, add = {}", x, add);
            res += add;
        }
    }
    println!("res = {}", res);
    return;
    // {
    //     // verify the solution
    //     println!("\nVerifying the solution");
    //     let example = fs::read_to_string("problems/7.txt").unwrap();
    //     let expr_ptr = parse_into_ast(example);
    //     let expr = &*expr_ptr.borrow();
    //     if let Expr::Binary(_, a, b) = expr {
    //         *b.borrow_mut() = Expr::Integer(34546387488);
    //         print_ast(b.clone());
    //     } else {
    //         panic!("Unreachable part");
    //     }
    //     let (res, _) = eval_expr(expr_ptr.clone());
    //     print_ast(as_ptr(res));
    // } 
    // {
    //     // verify the solution
    //     println!("\nVerifying the solution");
    //     let example = fs::read_to_string("problems/7.txt").unwrap();
    //     let expr_ptr = parse_into_ast(example);
    //     let (_, expr1, _) = unwrap_binary(expr_ptr);
    //     let (_, _, expr2) = unwrap_binary(expr1);
    //     let (_, expr3) = unwrap_lambda(expr2);
    //     let (_, expr4) = unwrap_lambda(expr3);

    //     // print_ast(expr3.clone());
    //     let res = apply(expr4, 41, as_ptr(Expr::Integer(34546387488)));
    //     // print_ast(res);
    //     eval_expr(res);

    // }

}


fn invert_solution(f: i32, s: &str) -> i64 {
    let parts = s.split(" ");

    let mut inverted = parts.clone().map(|x| {
        return (x.parse::<i64>().unwrap() * -1).to_string();
    }).collect::<Vec<String>>().join(" ");
    println!("inverted = \n{}", inverted);

    if f == 1 {
        let parts = inverted.split(" ");
        let mut res = 0 as i64;
        for part in parts {
            let x: i64 = part.parse().unwrap();
            if x > 0 {
                let add = 1 << (x.abs() - 1);
                println!("x = {}, add = {}", x, add);
                res += add;
            }
        }
        println!("res = {}", res);    
        return res;
    }
    -1
}

fn read_all_solution() {
    let mut res = Vec::new();
    let lines = fs::read_to_string("solutions/8-minisat-iter2.txt").unwrap();
    for line in lines.split('\n') {
        if line.len() < 30 {
            continue
        }
        let s = invert_solution(1, line);
        println!("{}", s);
        res.push(s);
    }
    println!("min = {}", res.iter().min().unwrap());
}


fn solve_eff8() {
    let example = fs::read_to_string("problems/8.txt").unwrap();
    let expr_ptr = parse_into_ast(example);

    let sat3 = convert_to_sat3(expr_ptr);
    println!("{:?}", sat3);


    let mut file = File::create("solutions/8-minisat.txt").unwrap();
    file.write_all(get_sat_string(&sat3).as_bytes());
    for vals in sat3 {
        let mut new_vals = vals.clone();
        new_vals.push(0);
        let output: String = new_vals.iter().map(|x| {x.to_string()}).collect::<Vec<String>>().join(" ");
        file.write_all((output + "\n").as_bytes());
    }

    read_all_solution();

    // let output = "-1 -2 -3 -4 -5 6 -7 -8 -9 10 -11 12 -13 -14 -15 -16 -17 -18 -19 -20 -21 22 -23 -24 25 26 -27 28 -29 -30 -31 -32 -33 -34 -35 36 -37 -38 -39 -40 0";


    // invert_solution(0, "-1 2 -3 4 5 6 7 8 9 -10 -11 12 -13 14 15 -16 -17 -18 19 20 -21 -22 23 -24 -25 -26 27 -28 -29 -30 -31 32 -33 -34 35 36 37 38 39 -40 -41 -42 -43 -44 -45 -46 47 48 49 -50 0");
    // invert_solution(0, "-1 2 -3 4 5 6 7 8 9 -10 -11 12 -13 14 15 -16 -17 -18 19 20 -21 -22 23 -24 -25 -26 27 -28 -29 -30 -31 32 -33 -34 35 36 37 38 39 -40 -41 42 -43 -44 -45 -46 47 48 49 -50 0");
    // invert_solution(0, "-1 2 -3 4 5 6 7 8 9 10 -11 12 -13 14 15 -16 -17 -18 19 20 -21 -22 23 -24 -25 -26 27 -28 -29 -30 -31 32 -33 -34 35 36 37 38 39 -40 -41 42 -43 -44 -45 -46 47 48 49 -50 0");
    // invert_solution(0, "-1 2 -3 4 5 6 7 8 9 10 -11 12 -13 14 15 -16 -17 -18 19 20 -21 -22 23 -24 -25 -26 27 -28 -29 -30 -31 32 -33 -34 35 36 37 38 39 -40 -41 -42 -43 -44 -45 -46 47 48 49 -50 0");
    // invert_solution(1, "-1 2 -3 4 5 6 7 8 9 -10 -11 12 -13 14 15 -16 -17 -18 19 20 -21 -22 23 -24 -25 -26 27 -28 -29 -30 -31 -32 -33 -34 35 36 37 38 39 -40 -41 42 -43 -44 -45 -46 47 48 49 -50 0");
    // {
    //     // verify the solution
    //     println!("\nVerifying the solution");
    //     let example = fs::read_to_string("problems/8.txt").unwrap();
    //     let expr_ptr = parse_into_ast(example);
    //     let expr = &*expr_ptr.borrow();
    //     if let Expr::Binary(_, a, b) = expr {
    //         *b.borrow_mut() = Expr::Integer(493116004788730);
    //         print_ast(b.clone());
    //     } else {
    //         panic!("Unreachable part");
    //     }
    //     let (res, _) = eval_expr(expr_ptr.clone());
    //     print_ast(as_ptr(res));
    // } 

    // {
    //     // verify the solution
    //     println!("\nVerifying the solution");
    //     let example = fs::read_to_string("problems/8.txt").unwrap();
    //     let expr_ptr = parse_into_ast(example);
    //     let (_, expr1, _) = unwrap_binary(expr_ptr);
    //     let (_, _, expr2) = unwrap_binary(expr1);
    //     let (_, expr3) = unwrap_lambda(expr2);
    //     let (_, expr4) = unwrap_lambda(expr3);

    //     // print_ast(expr3.clone());
    //     let res = apply(expr4, 51, as_ptr(Expr::Integer(493116004788730)));
    //     // print_ast(res);
    //     eval_expr(res);

    // }

}
// == Eff 13 ==
// Solution: Integer(536870919)

// Solution: Integer(536870919)
// 2**28 * 2 + 7
//  [8] Lambda x3
//      [12] Lambda x4
//          [16] If
//              [20] Binary =
//                  [24] x4
//                  [24] String("")
//              [20] Integer(0)
//              [20] Binary +
//                  [24] Integer(1)
//                  [24] Binary $
//                      [28] x3
//                      [28] Binary D
//                          [32] Integer(1)
//                          [32] x4
// strlen:
// def f(x) {
//     if x == 0 {
//         0
//     } else {
//         1 + f (x[1:])
//     }
// }

fn solve_eff13() {
    let example = fs::read_to_string("problems/13.txt").unwrap();
    let mut expr_ptr = parse_into_ast(example);
    let (_, _, right_ptr) = unwrap_binary(expr_ptr);
    print_ast(right_ptr.clone());
    let expr2 = adhoc_replace(right_ptr.clone());
    let expr3 = rewrite_expr(expr2);
    // print_ast(expr3.clone());
    let (expr4, _) = eval_expr(expr3);
    print_ast(as_ptr(expr4));
}

// very specific function for problem 13
fn adhoc_replace(expr_ptr: ExprPtr) -> ExprPtr {
    println!("adhoc_replace at {:?}", expr_ptr);
    let ref e = *expr_ptr.borrow();
    match e {
        Expr::Lambda(x, a) => {
            let new_a = adhoc_replace(a.clone());
            return as_ptr(Expr::Lambda(*x, new_a));
        }
        Expr::Binary(op, a, b) => {
            let new_a = adhoc_replace(a.clone());
            let new_b = adhoc_replace(b.clone());
            let new_op = {
                if *op == '.' {
                    '+'
                } else {
                    *op
                }
            };
            return as_ptr(Expr::Binary(new_op, new_a, new_b));
        }
        Expr::String(s) => {
            return as_ptr(Expr::Integer(s.len() as i64));
        }
        _ => return expr_ptr.clone(),
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() > 1 {
        solve_eff_generic(args[1].clone());
        return;
    }

    // solve_eff1();
    // solve_eff2();
    // solve_eff3();
    // solve_eff4();
    // solve_eff5();
    // solve_eff6();
    // solve_eff7();
    solve_eff8();


    // solve_eff13();
}
