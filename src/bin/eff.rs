use icfpc_2024::*;

use std::fs;

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
            _ => return expr_ptr.clone()
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


// need to rewrite addition with multiplication
// Integer(17592186044416)
fn solve_eff1() {
    let example = fs::read_to_string("problems/1.txt").unwrap();
    let mut expr_ptr = parse_into_ast(example);
    expr_ptr = rewrite_expr_times(expr_ptr, 2);
    let (res, _) = eval_expr(expr_ptr);
    print_ast(as_ptr(res));
}


// multiply by 0, doesn't matter
// Integer(2134) 
fn solve_eff2() {
    let example = fs::read_to_string("problems/2.txt").unwrap();
    let mut expr_ptr = parse_into_ast(example);
    let (res, _) = eval_expr(expr_ptr);
    print_ast(as_ptr(res));
}

// The same as previous but multiply by 1, so no need to compute fairly
fn solve_eff3() {
    let example = fs::read_to_string("problems/3.txt").unwrap();
    let mut expr_ptr = parse_into_ast(example);
    let (res, _) = eval_expr(expr_ptr);
    print_ast(as_ptr(res));

}


fn main() {
    // solve_eff1();
    // solve_eff2();
    solve_eff3();
}
