use std::rc::Rc;
use super::CSP;

pub fn n_queens(n: i32) -> CSP<i32,i32> {
    let mut csp = CSP::new();
    for q in 1..(n+1) { csp.add_var(q, (1..(n+1)).collect()) }
    for i in 1..(n+1) {
        for j in 1..(n+1) {
            if i != j {
                csp.add_constr(i, j, Rc::new(|x,y| x != y));
                csp.add_constr(i, j, Rc::new(move |x,y| (x-y).abs() != (i-j).abs()));
            }
        }
    }
    csp
}

pub fn sudoku() -> CSP<(usize, usize), usize> {
    let mut csp = CSP::<(usize, usize), usize>::new();
    for i in 1..10 {
        for j in 1..10 { csp.add_var((i,j), (1..10).collect()) }
    }
    
    // Row / column constraints
    for i in 1..10 {
        for j in 1..10 {
            for k in 1..10 {
                if j == k { continue }
                csp.add_constr((i,j), (i,k), Rc::new(|x,y| x != y));
                csp.add_constr((j,i), (k,i), Rc::new(|x,y| x != y)); }}}

    // Block constraints
    for bi in 0..3 {
        for bj in 0..3 {
            for i in (bi*3 + 1)..(bi*3 + 4) {
                for j in (bj*3 + 1)..(bj*3 + 4) {
                    for k in (bi*3 + 1)..(bi*3 + 4) {
                        for l in (bj*3 + 1)..(bj*3 + 4) {
                            if i != k || j != l {
                                csp.add_constr((i,j), (k,l), Rc::new(|x,y| x != y)); }}}}}}}

    // Row existential constraints, numbered (11...19 (which row), 1...9 (which value))
    for i in 11..20 {
        for j in 1..10 {
            csp.add_var((i,j), (1..10).collect()); }}
    // Constraints against column position
    for i in 11..20 {
        for j in 1..10 {
            for k in 1..10 {
                for l in 1..10 {
                    if k == i - 10 { // Placing value j in row k, compare to column l
                        csp.add_constr((i,j), (k,l), Rc::new(move |&x,&y| (x != l) ^ (y == j)));
                        csp.add_constr((k,l), (i,j), Rc::new(move |&y,&x| (x != l) ^ (y == j)));
                    } else { // Placing value j in row i-10, compare to row k column l
                        csp.add_constr((i,j), (k,l), Rc::new(move |&x,&y| (x != l) || (y != j)));
                        csp.add_constr((k,l), (i,j), Rc::new(move |&y,&x| (x != l) || (y != j)));
                    }}}}}
    // Column existential constraints, numbered (11...19 (which row), 1...9 (which value))
    for i in 11..20 {
        for j in 1..10 {
            csp.add_var((j,i), (1..10).collect()); }}
    // Constraints against row position
    for i in 11..20 {
        for j in 1..10 {
            for k in 1..10 {
                for l in 1..10 {
                    if k == i - 10 { // Placing value j in row k, compare to column l
                        csp.add_constr((j,i), (l,k), Rc::new(move |&x,&y| (x != l) ^ (y == j)));
                        csp.add_constr((l,k), (j,i), Rc::new(move |&y,&x| (x != l) ^ (y == j)));
                    } else { // Placing value j in row i-10, compare to row k column l
                        csp.add_constr((j,i), (l,k), Rc::new(move |&x,&y| (x != l) || (y != j)));
                        csp.add_constr((l,k), (j,i), Rc::new(move |&y,&x| (x != l) || (y != j)));
                    }}}}}
    
    csp
}
