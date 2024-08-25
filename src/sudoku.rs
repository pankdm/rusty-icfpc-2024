
use crate::*;


pub type Grid = Vec<Vec<u8>>;


fn print_grid(grid: &Grid) {
    for row in grid {
        for &val in row.iter() {
            print!("{} ", val);
        }
        println!();
    }
}

pub fn solve_empty_sudoku() -> Grid {
    // Solver from empty state
    let mut state = vec![vec![0; 9]; 9];
    let ok = solve_from_state(&mut state);
    assert!(ok);
    print_grid(&state);
    state
}


fn is_ok(state: &Grid, row: usize, col: usize, num: u8) -> bool {
    for x in 0..9 {
        if state[row][x] == num || state[x][col] == num {
            return false;
        }
    }

    let start_row = row / 3 * 3;
    let start_col = col / 3 * 3;
    for i in 0..3 {
        for j in 0..3 {
            if state[start_row + i][start_col + j] == num {
                return false;
            }
        }
    }

    true
}

pub fn solve_from_state(state: &mut Grid) -> bool {
    for row in 0..9 {
        for col in 0..9 {
            if state[row][col] == 0 {
                for num in 1..=9 {
                    if is_ok(state, row, col, num) {
                        state[row][col] = num;
                        if solve_from_state(state) {
                            return true
                        }
                        state[row][col] = 0;
                    }
                }
                return false;
            }
        }
    }
    true
}

fn visit_sudoku(expr_ptr: ExprPtr, res: &mut HashMap<usize, u8>) {
    let e = &*expr_ptr.borrow();
    match e {
        Expr::Unary(_, a) => {
            visit_sudoku(a.clone(), res);
        }
        Expr::Binary(op, a, b) => {
            if *op == '=' {
                println!("At Op(=) a={:?}, b={:?}", short_str(&a.borrow()), short_str(&b.borrow()));
                if let Expr::Var(x) = &*a.borrow() {
                    if let Expr::Integer(y) = &*b.borrow() {
                        res.insert(*x as usize, *y as u8);
                    }
                }
            } else {
                visit_sudoku(a.clone(), res);
                visit_sudoku(b.clone(), res);    
            }
        }
        Expr::Lambda(_, a) => {
            visit_sudoku(a.clone(), res);
        }
        Expr::If(a, b, c) => {
            visit_sudoku(a.clone(), res);
            visit_sudoku(b.clone(), res);
            visit_sudoku(c.clone(), res);
        }
        Expr::Var(x) => {
            // Ignore variables
        }
        Expr::String(_) | Expr::Integer(_)=> {
            // ignore temporary strings
        }
        _ => {
            panic!("Unexpected expression: {:?}", e);
        }
    }
}

pub fn extract_initial_state(expr_ptr: ExprPtr) -> Grid {
    let mut res = HashMap::new();
    visit_sudoku(expr_ptr, &mut res);
    let mut state = vec![vec![0; 9]; 9];
    println!("{:?}", res);
    for (k, v) in res.iter() {
        let row = k / 10 - 1;
        let col = k % 10 - 1;
        println!("k = {}, row = {}, col = {}, v = {}", k, row, col, v);
        state[row][col] = *v;
    }
    state
}

