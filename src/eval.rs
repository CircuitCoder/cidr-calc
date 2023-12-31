use std::{rc::Rc, collections::VecDeque, iter};

use anyhow::anyhow;

use crate::{parser::{Expr, Atomic, Stmt}, data::{V4, V6}};

#[derive(Clone, Debug)]
struct SetNode<const MAX_DEPTH: usize> {
    depth: usize,
    covered: bool,
    left: Option<Rc<SetNode<MAX_DEPTH>>>,
    right: Option<Rc<SetNode<MAX_DEPTH>>>,
}

fn union_option<const MAX_DEPTH: usize>(
    lhs: &Option<Rc<SetNode<MAX_DEPTH>>>,
    rhs: &Option<Rc<SetNode<MAX_DEPTH>>>,
) -> Option<Rc<SetNode<MAX_DEPTH>>> {
    match (lhs, rhs) {
        (None, r) => r.clone(),
        (l @ Some(_), None) => l.clone(),
        (Some(l), Some(r)) => Some(Rc::new(l.union(r.as_ref()))),
    }
}

fn subtract_option<const MAX_DEPTH: usize>(
    lhs: &Option<Rc<SetNode<MAX_DEPTH>>>,
    rhs: &Option<Rc<SetNode<MAX_DEPTH>>>,
) -> Option<Rc<SetNode<MAX_DEPTH>>> {
    match (lhs, rhs) {
        (None, _) => None,
        (l @ Some(_), None) => l.clone(),
        (Some(l), Some(r)) => {
            let raw = l.subtract(r.as_ref());
            // Subtraction may result in empty set
            if !raw.covered && raw.left.is_none() && raw.right.is_none() {
                None
            } else {
                Some(Rc::new(raw))
            }
        }
    }
}

impl<const MAX_DEPTH: usize> SetNode<MAX_DEPTH> {
    pub fn union(&self, ano: &SetNode<MAX_DEPTH>) -> SetNode<MAX_DEPTH> {
        assert_eq!(ano.depth, self.depth);
        if self.covered || ano.covered {
            return SetNode {
                depth: self.depth,
                covered: true,
                left: None,
                right: None,
            };
        }

        if self.is_empty() && ano.is_empty() {
            return self.clone();
        }

        assert_ne!(self.depth, MAX_DEPTH);
        let left = union_option(&self.left, &ano.left);
        let right = union_option(&self.right, &ano.right);
        let covered = left.as_ref().map_or(false, |i| i.covered)
            && right.as_ref().map_or(false, |i| i.covered);

        if covered {
            return SetNode {
                depth: self.depth,
                covered: true,
                left: None,
                right: None,
            };
        }

        SetNode {
            depth: self.depth,
            covered: false,
            left,
            right,
        }
    }

    pub fn subtract(&self, ano: &SetNode<MAX_DEPTH>) -> SetNode<MAX_DEPTH> {
        assert_eq!(ano.depth, self.depth);
        if self.is_empty() || ano.covered {
            return SetNode {
                depth: self.depth,
                covered: false,
                left: None,
                right: None,
            }
        }

        if ano.is_empty() {
            return self.clone();
        }

        let mut left_ref = &self.left;
        let mut right_ref = &self.right;
        let full;
        if self.covered {
            full = Some(Rc::new(SetNode {
                depth: self.depth + 1,
                covered: true,
                left: None,
                right: None,
            }));
            left_ref = &full;
            right_ref = &full;
        }

        assert_ne!(self.depth, MAX_DEPTH);
        let left = subtract_option(left_ref, &ano.left);
        let right = subtract_option(right_ref, &ano.right);
        let covered = left.as_ref().map_or(false, |i| i.covered)
            && right.as_ref().map_or(false, |i| i.covered);

        if covered {
            return SetNode {
                depth: self.depth,
                covered: true,
                left: None,
                right: None,
            };
        }

        SetNode {
            depth: self.depth,
            covered: false,
            left,
            right,
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.covered && self.left.is_none() && self.right.is_none()
    }

    pub fn is_canonical(&self) -> bool {
        todo!()
    }
}

#[derive(Clone)]
struct SetWalkerFrame<'a, const MAX_DEPTH: usize> {
    node: &'a SetNode<MAX_DEPTH>,
    inspected_branches: usize, // Inspected branches. Now always 0 / 1 / 2
}

// Iterator, stack top always pointing at a covered node, except in the terminal state, where the stack is empty
struct SetWalker<'a, const MAX_DEPTH: usize> {
    stack: VecDeque<SetWalkerFrame<'a, MAX_DEPTH>>
}

impl<'a, const MAX_DEPTH: usize> SetWalker<'a, MAX_DEPTH> {
    pub fn new(n: &'a SetNode<MAX_DEPTH>) -> Self {
        let mut s = Self {
            stack: VecDeque::with_capacity(MAX_DEPTH + 1),
        };
        s.stack.push_back(SetWalkerFrame { node: n, inspected_branches: 0 });
        s
    }

    fn step(&mut self) {
        if self.stack.is_empty() {
            assert!(self.stack.is_empty());
            return;
        }

        let top_node = self.stack.back_mut().unwrap().node;
        let top_inspected = &mut self.stack.back_mut().unwrap().inspected_branches;
        match *top_inspected {
            0 => {
                *top_inspected += 1;
                if let Some(ref cur) = top_node.left {
                    self.stack.push_back(SetWalkerFrame { node: cur.as_ref(), inspected_branches: 0 })
                }
            },
            1 => {
                *top_inspected += 1;
                if let Some(ref cur) = top_node.right {
                    self.stack.push_back(SetWalkerFrame { node: cur.as_ref(), inspected_branches: 0 })
                }
            },
            2 => {
                self.stack.pop_back();
            },
            _ => unreachable!()
        }
    }
}

impl<'a, const MAX_DEPTH: usize> Iterator for SetWalker<'a, MAX_DEPTH> {
    type Item = (u128, usize); // TODO: high percision?

    fn next(&mut self) -> Option<Self::Item> {
        while match self.stack.back() {
            None => false,
            Some(inner) => !inner.node.covered || inner.inspected_branches > 0
        } {
            self.step();
        }

        if self.stack.is_empty() {
            return None;
        }

        // Arrived at a covered node
        assert_eq!(self.stack.back().map(|e| e.node.covered), Some(true));

        // Serialize stack
        let mut addr = 0u128;
        let mut len = 0;
        for elem in &self.stack {
            if elem.inspected_branches == 0 {
                // Last one
                break;
            }
            addr <<= 1;
            addr |= (elem.inspected_branches - 1) as u128;
            len += 1;
        }

        if len != 0 { // Avoid UB
            addr <<= MAX_DEPTH - len;
        }

        self.step();

        return Some((addr, len));
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Unit,
    V4Set(SetNode<32>),
    V6Set(SetNode<128>),
}

impl Value {
    fn is_same_len(&self, ano: &Value) -> bool {
        match (self, ano) {
            (Value::V4Set(_), Value::V6Set(_)) | (Value::V6Set(_), Value::V4Set(_)) => false,
            _ => true,
        }
    }

    fn union(&self, ano: &Value) -> anyhow::Result<Value> {
        if !self.is_same_len(&ano) {
            return Err(anyhow!("Cannot add a v4 set to a v6 set")); // TODO: diagnostic
        }

        match (self, ano) {
            (Value::V4Set(l), Value::V4Set(r)) => Ok(Value::V4Set(l.union(r))),
            (Value::V6Set(l), Value::V6Set(r)) => Ok(Value::V6Set(l.union(r))),
            _ => unreachable!(),
        }
    }

    fn subtract(&self, ano: &Value) -> anyhow::Result<Value> {
        if !self.is_same_len(&ano) {
            return Err(anyhow!("Cannot subtract a v4 set to a v6 set")); // TODO: diagnostic
        }

        match (self, ano) {
            (Value::V4Set(l), Value::V4Set(r)) => Ok(Value::V4Set(l.subtract(r))),
            (Value::V6Set(l), Value::V6Set(r)) => Ok(Value::V6Set(l.subtract(r))),
            _ => unreachable!(),
        }
    }
}

fn construct_set_node<const MAX_DEPTH: usize>(addr: u128, len: usize, depth: usize) -> SetNode<MAX_DEPTH> {
    if depth == len {
        assert!(depth <= MAX_DEPTH);
        return SetNode {
            depth,
            covered: true,
            left: None,
            right: None,
        };
    }

    assert!(depth < MAX_DEPTH);

    let child = Some(Rc::new(construct_set_node(addr, len, depth + 1)));
    let mut cur = SetNode {
        depth,
        covered: false,
        left: None,
        right: None,
    };
    if (addr >> (MAX_DEPTH - depth - 1)) & 1 == 0 {
        cur.left = child;
    } else {
        cur.right = child;
    }

    cur
}

impl From<&V4> for SetNode<32> {
    fn from(value: &V4) -> Self {
        construct_set_node(value.0 as u128, value.1 as usize, 0)
    }
}

impl From<(u128, usize)> for V4 {
    fn from(value: (u128, usize)) -> Self {
        Self(value.0 as u32, value.1 as u8)
    }
}

impl From<&V6> for SetNode<128> {
    fn from(value: &V6) -> Self {
        construct_set_node(value.0, value.1 as usize, 0)
    }
}

impl From<(u128, usize)> for V6 {
    fn from(value: (u128, usize)) -> Self {
        Self(value.0, value.1 as u8)
    }
}

#[derive(Clone)]
pub struct Scope {
    bindings: im::HashMap<String, Value>,
}

impl Default for Scope {
    fn default() -> Self {
        Self { bindings: Default::default() }
    }
}

impl Scope {
    pub fn keys<'s>(&'s self) -> impl Iterator<Item = &'s str> + 's {
        self.bindings.keys().map(String::as_str)
    }
}

pub fn eval<'a>(stmts: &Vec<Stmt<'a>>) -> anyhow::Result<Vec<Value>> {
    let mut scope = Scope {
        bindings: Default::default()
    };

    let mut output = Vec::with_capacity(stmts.len());

    for stmt in stmts {
        let (v, s) = eval_stmt(stmt, scope)?;
        scope = s;
        output.push(v);
    }

    Ok(output)
}

pub fn format<'a>(v: &'a Value) -> Box<dyn Iterator<Item = String> + 'a> {
    match v {
        Value::Unit => Box::new(iter::empty()),
        Value::V4Set(s) => Box::new(SetWalker::new(s).map(V4::from).map(|e| e.to_string())),
        Value::V6Set(s) => Box::new(SetWalker::new(s).map(V6::from).map(|e| e.to_string())),
    }
}

pub fn eval_stmt<'a>(stmt : &Stmt<'a>, mut s: Scope) -> anyhow::Result<(Value, Scope)> {
    match stmt {
        Stmt::LetIn { ident, val } => {
            let val_evaled = eval_expr(val.as_ref(), s.clone())?;
            s.bindings.insert(ident.to_string(), val_evaled);
            Ok((Value::Unit, s))
        },
        Stmt::Expr(e) => eval_expr(e, s.clone()).map(|r| (r, s))
    }
}

fn eval_expr<'a>(expr: &Expr<'a>, s: Scope) -> anyhow::Result<Value> {
    match expr {
        Expr::Addition(lhs, rhs) => {
            let lhs = eval_expr(lhs, s.clone())?;
            let rhs = eval_expr(rhs, s)?;
            lhs.union(&rhs)
        }
        Expr::Subtraction(lhs, rhs) => {
            let lhs = eval_expr(lhs, s.clone())?;
            let rhs = eval_expr(rhs, s)?;
            lhs.subtract(&rhs)
        }
        Expr::Atomic(a) => match a {
            Atomic::Ident(i) => {
                let lookup = s.bindings.get(*i);
                lookup.cloned().ok_or_else(|| anyhow!("Identifier not found in scope: {}", *i))
            }
            Atomic::V4(v) => Ok(Value::V4Set(v.into())),
            Atomic::V6(v) => Ok(Value::V6Set(v.into())),
        }
    }
}

#[test]
fn test() {
    fn eval_single<'a>(stmt: &Stmt<'a>) -> anyhow::Result<Value> {
        eval_stmt(stmt, Scope {
            bindings: Default::default()
        }).map(|e| e.0)
    }
    use crate::parser::parse_single;
    // FIXME: write real test!
    println!("{:?}", eval_single(&parse_single("0.0.0.0/0").unwrap()));
    println!("{:?}", eval_single(&parse_single("::1/128").unwrap()));
    println!("{:?}", eval_single(&parse_single("::1/128 - ::/0").unwrap()));
    println!("{:?}", eval_single(&parse_single("0.0.0.0/1").unwrap()));
    println!("{:?}", eval_single(&parse_single("0.0.0.0/1 + 128.0.0.0/1").unwrap()));
    println!("{:?}", eval_single(&parse_single("0.0.0.0/0 - 101.6.6.6/32").unwrap()));
}