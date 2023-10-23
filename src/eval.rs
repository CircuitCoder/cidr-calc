use std::{rc::Rc, collections::VecDeque};

use anyhow::anyhow;

use crate::{parser::{Expr, Atomic}, data::{V4, V6}};

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

fn substract_option<const MAX_DEPTH: usize>(
    lhs: &Option<Rc<SetNode<MAX_DEPTH>>>,
    rhs: &Option<Rc<SetNode<MAX_DEPTH>>>,
) -> Option<Rc<SetNode<MAX_DEPTH>>> {
    match (lhs, rhs) {
        (None, _) => None,
        (l @ Some(_), None) => l.clone(),
        (Some(l), Some(r)) => {
            let raw = l.substract(r.as_ref());
            // Substraction may result in empty set
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

    pub fn substract(&self, ano: &SetNode<MAX_DEPTH>) -> SetNode<MAX_DEPTH> {
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
        let left = substract_option(left_ref, &ano.left);
        let right = substract_option(right_ref, &ano.right);
        println!("Depth: {}", self.depth);
        println!("Left: {:?}", left);
        println!("Right: {:?}", right);
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

    fn substract(&self, ano: &Value) -> anyhow::Result<Value> {
        if !self.is_same_len(&ano) {
            return Err(anyhow!("Cannot substract a v4 set to a v6 set")); // TODO: diagnostic
        }

        match (self, ano) {
            (Value::V4Set(l), Value::V4Set(r)) => Ok(Value::V4Set(l.substract(r))),
            (Value::V6Set(l), Value::V6Set(r)) => Ok(Value::V6Set(l.substract(r))),
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
struct Scope<'s> {
    bindings: im::HashMap<&'s str, Value>,
}

pub fn eval<'a>(expr: &Expr<'a>) -> anyhow::Result<Value> {
    eval_scope(expr, Scope {
        bindings: Default::default()
    })
}

pub fn format<'a>(v: &'a Value) -> Box<dyn Iterator<Item = String> + 'a> {
    match v {
        Value::V4Set(s) => Box::new(SetWalker::new(s).map(V4::from).map(|e| e.to_string())),
        Value::V6Set(s) => Box::new(SetWalker::new(s).map(V6::from).map(|e| e.to_string())),
    }
}

fn eval_scope<'a>(expr: &Expr<'a>, s: Scope<'a>) -> anyhow::Result<Value> {
    match expr {
        Expr::LetIn { ident, val, body } => {
            let val_evaled = eval_scope(val.as_ref(), s.clone())?;
            let mut new_scope = s.clone();
            new_scope.bindings.insert(*ident, val_evaled);
            eval_scope(&body, new_scope)
        }
        Expr::Addition(lhs, rhs) => {
            let lhs = eval_scope(lhs, s.clone())?;
            let rhs = eval_scope(rhs, s)?;
            lhs.union(&rhs)
        }
        Expr::Subtraction(lhs, rhs) => {
            let lhs = eval_scope(lhs, s.clone())?;
            let rhs = eval_scope(rhs, s)?;
            lhs.substract(&rhs)
        }
        Expr::Atomic(a) => match a {
            Atomic::Ident(i) => {
                let lookup = s.bindings.get(i);
                lookup.cloned().ok_or_else(|| anyhow!("Identifier not found in scope: {}", *i))
            }
            Atomic::V4(v) => Ok(Value::V4Set(v.into())),
            Atomic::V6(v) => Ok(Value::V6Set(v.into())),
        }
    }
}

#[test]
fn test() {
    use crate::parser::parse;
    // FIXME: write real test!
    println!("{:?}", eval(&parse("0.0.0.0/0").unwrap()));
    println!("{:?}", eval(&parse("::1/128").unwrap()));
    println!("{:?}", eval(&parse("::1/128 - ::/0").unwrap()));
    println!("{:?}", eval(&parse("0.0.0.0/1").unwrap()));
    println!("{:?}", eval(&parse("0.0.0.0/1 + 128.0.0.0/1").unwrap()));
    println!("{:?}", eval(&parse("0.0.0.0/0 - 101.6.6.6/32").unwrap()));
}