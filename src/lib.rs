#![feature(test)]
extern crate test;

use std::collections::HashMap;
use std::iter::Iterator;
use std::rc::Rc;

#[derive(Clone)]
struct CSP<K,V> where
    K: std::cmp::Eq,
    K: std::hash::Hash,
    V: std::clone::Clone
{
    vars: HashMap<K, DVar<V>>,
    constrs: Vec<(K, K, Rc<Fn(V,V) -> bool>)>
}

struct CSPSolution<K,V> where
    K: std::cmp::Eq,
    K: std::hash::Hash,
    V: std::clone::Clone
{
    problem_stack: Vec<CSP<K,V>>,
    variables: Vec<K>,
    nvars: usize,
    branches: Vec<usize>,
    done: bool,
}

#[derive(Clone)]
struct DVar<V> where V: std::clone::Clone {
    options: Vec<V>,
}

impl<K,V> CSP<K,V> where
    K: std::cmp::Eq,
    K: std::clone::Clone,
    K: std::hash::Hash,
    V: std::clone::Clone,
{
    fn new() -> CSP<K,V> {
        CSP { vars: HashMap::new(), constrs: Vec::new() }
    }

    fn add_var(&mut self, key: K, options: Vec<V>) {
        let var = DVar { options: options };
        self.vars.insert(key, var);
    }

    fn add_constr(&mut self, key1: K, key2: K, constr: Rc<Fn(V, V) -> bool>) {
        self.constrs.push((key1, key2, constr));
    }

    fn reduce(&mut self) -> Option<bool> {
        let mut reduced = false;
        let mut did_some = false;
        let mut first = true;
        while did_some || first {
            first = false;
            did_some = false;
            for &(ref x, ref y, ref cf) in self.constrs.iter() {
                let mut goodopts = Vec::new();
                {
                    let xvar = self.vars.get(&x).unwrap();
                    let nopts = xvar.options.len();
                    for xo in xvar.options.iter() {
                        if self.vars.get(&y).unwrap().options.iter()
                               .any(|y| cf(xo.clone(), y.clone())) {
                            goodopts.push(xo.clone());
                        }
                    }
                    if goodopts.len() == 0 { return None };
                    if goodopts.len() < nopts { reduced = true; did_some = true };
                }
                self.vars.get_mut(&x).unwrap().options = goodopts;
            }
        }
        Some(reduced)
    }

    fn solutions(&mut self) -> CSPSolution<K,V> {
        let _ = self.reduce();
        CSPSolution::new(self)
    }
}

impl<K,V> CSPSolution<K,V> where
    K: std::cmp::Eq,
    K: std::clone::Clone,
    K: std::hash::Hash,
    V: std::clone::Clone,
{
    fn new(csp: &CSP<K,V>) -> CSPSolution<K,V> {
        let vars: Vec<K> = csp.vars.keys().map(|x| (*x).clone()).collect();
        let nvars = vars.len();
        let mut stack = Vec::with_capacity(nvars + 1);
        stack.push(csp.clone());
        let mut ret = CSPSolution {
            problem_stack: stack,
            variables: vars,
            nvars: nvars,
            branches: vec![0; nvars],
            done: false,
        };
        if !ret.find_consistent(0) { ret.done = true }
        ret
    }

    fn find_consistent(&mut self, start: usize) -> bool {
        let mut cur = start;
        loop {
            if cur >= self.nvars { break }
            let _ = self.problem_stack.drain((cur + 1)..);
            let mut csp = self.problem_stack.last().unwrap().clone();
            csp.vars.get_mut(&self.variables[cur]).unwrap().restrict(self.branches[cur]);
            if csp.reduce().is_none() {
                cur = match self.incr_branches(cur) {
                    None => return false,
                    Some(s) => s,
                }
            } else {
                self.problem_stack.push(csp);
                cur += 1;
            }
        }
        true
    }

    fn incr_branches(&mut self, last: usize) -> Option<usize> {
        for cur in (last + 1)..self.nvars { self.branches[cur] = 0 }
        let mut cur = last;
        while self.branches[cur] + 1 ==
            self.problem_stack[cur].vars.get(&self.variables[cur]).unwrap().options.len()
        {
            self.branches[cur] = 0;
            if cur > 0 { cur -= 1 } else { self.done = true; return None }
        }
        self.branches[cur] += 1;
        Some(cur)
    }

    fn incr_consistent(&mut self) -> bool {
        let last = self.nvars - 1;
        match self.incr_branches(last) {
            None => false,
            Some(s) => self.find_consistent(s),
        }
    }

    fn result(&self) -> HashMap<K,V> {
        let mut map = HashMap::new();
        for (k, v) in self.problem_stack.last().unwrap().vars.iter() {
            let _ = map.insert((*k).clone(), v.options[0].clone());
        }
        map
    }
}

impl<K,V> Iterator for CSPSolution<K,V> where
    K: std::cmp::Eq,
    K: std::clone::Clone,
    K: std::hash::Hash,
    V: std::clone::Clone,
{
    type Item = HashMap<K,V>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None }
        let res = Some(self.result());
        self.done = !self.incr_consistent();
        res
    }
}

impl <V> DVar<V> where V: std::clone::Clone {
    fn restrict(&mut self, which: usize) {
        let opt = self.options[which].clone();
        self.options.clear();
        self.options.push(opt);
    }
}

#[cfg(test)]
mod tests {
    use super::CSP;
    use test::Bencher;
    use std::rc::Rc;

    #[test]
    fn simple_reduce_test() {
        let mut csp = CSP::new();
        csp.add_var(1, vec![1,2]);
        csp.add_var(2, vec![1,2]);
        let t1 = Rc::new(|x,_| x == 1);
        csp.add_constr(1, 2, t1);
        csp.reduce();
        let mut nsols = 0;
        for m in csp.solutions() {
            nsols += 1;
            if nsols > 10 { assert!(false) }
            assert!(m[&1] == 1);
            assert!(m[&2] == 1 || m[&2] == 2);
        }
        assert!(nsols == 2);
    }

    #[bench]
    fn eight_queens(b: &mut Bencher) {
        let mut csp: CSP<i32, i32> = CSP::new();
        for q in 1..9 { csp.add_var(q, (1..9).collect()) }
        for i in 1i32..9 {
            for j in 1i32..9 {
                if i != j {
                    csp.add_constr(i, j, Rc::new(|x,y| x != y));
                    csp.add_constr(i, j, Rc::new(move |x,y| (x-y).abs() != (i-j).abs()));
                }
            }
        }
        assert!(!csp.reduce().unwrap());
        assert!(csp.vars.values().all(|d| d.options.len() == 8));
        assert!(!csp.reduce().unwrap());
        assert!(csp.vars.values().all(|d| d.options.len() == 8));

        b.iter(|| {
            let mut nsols = 0;
            for _ in csp.solutions() {
                nsols += 1;
                if nsols > 1000 { break }
            }
            assert!(nsols == 92);
        });
    }
}
