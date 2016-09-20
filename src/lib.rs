use std::collections::HashMap;
use std::collections::HashSet;
use std::iter::Iterator;
use std::rc::Rc;

#[derive(Clone)]
struct CSP<K,V> where
    K: std::cmp::Eq,
    K: std::clone::Clone,
    K: std::hash::Hash,
    V: std::clone::Clone
{
    vars: HashMap<K, DVar<V>>,
    constrs: Vec<(K, K, Rc<Fn(V,V) -> bool>)>
}

struct CSPSolution<K,V> where
    K: std::cmp::Eq,
    K: std::clone::Clone,
    K: std::hash::Hash,
    V: std::clone::Clone
{
    problem_stack: Vec<CSP<K,V>>,
    variables: Vec<K>,
    branches: Vec<usize>,
    done: bool,
    visited: HashSet<Vec<usize>>,
}

#[derive(Clone,Debug)]
struct DVar<V> where V: std::clone::Clone {
    options: Vec<V>,
}

impl<K,V> CSP<K,V> where
    K: std::cmp::Eq,
    K: std::clone::Clone,
    K: std::hash::Hash,
    K: std::fmt::Debug,
    V: std::clone::Clone,
    V: std::fmt::Debug,
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
    K: std::fmt::Debug,
    V: std::clone::Clone,
    V: std::fmt::Debug,
{
    fn new(csp: &CSP<K,V>) -> CSPSolution<K,V> {
        let vars: Vec<K> = csp.vars.keys().map(|x| (*x).clone()).collect();
        let nvars = vars.len();
        let cur = (*csp).clone();
        let mut stack = Vec::with_capacity(nvars + 1);
        stack.push(cur.clone());
        let mut ret = CSPSolution {
            problem_stack: stack,
            variables: vars,
            branches: vec![0; nvars],
            done: false,
            visited: HashSet::new(),
        };
        if !ret.find_consistent(0) { ret.done = true }
        ret
    }

    fn find_consistent(&mut self, start: usize) -> bool {
        println!("find_consistent() called");
        let mut csp = self.problem_stack[start].clone();
        let _ = self.problem_stack.drain((start + 1)..);
        if start + 1 == self.variables.len() {
            let _ = csp.vars.get_mut(self.variables.last().unwrap()).unwrap().options.drain(1..);
            self.problem_stack.push(csp);
            return true;
        }

        let opts = csp.vars.get(&self.variables[start]).unwrap().options.clone();
        for i in 0..opts.len() {
            let mut cur = csp.clone();
            cur.vars.get_mut(&self.variables[start]).unwrap().options = vec![opts[i].clone()];
            if cur.reduce().is_some() {
                self.problem_stack.push(cur);
                self.branches[start] = i;
                if !self.find_consistent(start + 1) { let _ = self.problem_stack.pop(); }
                else { return true }
            }
        }
        false
    }

    fn incr_branches(&mut self, last: usize) -> Option<usize> {
        let mut cur = last;
        while self.branches[cur] + 1 ==
            self.problem_stack[cur].vars.get( &self.variables[cur]).unwrap().options.len()
        {
            self.branches[cur] = 0;
            if cur > 0 { cur -= 1 } else { self.done = true; return None }
        }
        self.branches[cur] += 1;
        Some(cur)
    }

    fn incr_consistent(&mut self) -> bool {
        let mut start = self.variables.len() - 1;
        loop {
            println!("Incrementing...");
            start = match self.incr_branches(start) {
                None => return false,
                Some(s) => s,
            };

            let _ = self.problem_stack.drain((start + 1)..);
            let mut csp = self.problem_stack.last().unwrap().clone();
            let opt = csp.vars.get_mut(&self.variables[start]).unwrap()
                              .options[self.branches[start]].clone();
            csp.vars.get_mut(&self.variables[start]).unwrap().options = vec![opt];
            if csp.reduce().is_none() { continue }
            self.problem_stack.push(csp.clone());

            if start + 1 >= self.variables.len() || self.find_consistent(start + 1) { break }
        }
        if !self.visited.contains(&self.branches) {
            self.visited.insert(self.branches.clone());
        } else {
            panic!("Configuration {:?} already seen!", self.branches);
        }
        true
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
    K: std::fmt::Debug,
    V: std::clone::Clone,
    V: std::fmt::Debug,
{
    type Item = HashMap<K,V>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done { return None }
        let res = Some(self.result());
        self.done = !self.incr_consistent();
        res
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    #[test]
    fn simple_reduce_test() {
        let mut csp = super::CSP::new();
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
            println!("Solution: {:?}", m);
        }
        assert!(nsols == 2);
    }

    #[test]
    fn eight_queens() {
        let mut csp: super::CSP<i32, i32> = super::CSP::new();
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

        let mut nsols = 0;
        for s in csp.solutions() {
            nsols += 1;
            if nsols > 100 { println!("Solution: {:?}", s); break }
        }
        println!("Num solutions: {}", nsols);
        assert!(nsols == 92);
    }
}
