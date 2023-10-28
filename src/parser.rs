use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
use anyhow::anyhow;

use crate::data::*;

#[derive(Parser)]
#[grammar="./syntax.pest"]
struct SrcParser;

#[derive(Debug, PartialEq, Eq)]
pub enum Stmt<'a> {
    LetIn {
        ident: &'a str,
        val: Box<Expr<'a>>,
    },
    Expr(Expr<'a>)
}

impl<'a> From<Expr<'a>> for Stmt<'a> {
    fn from(value: Expr<'a>) -> Self {
        Self::Expr(value)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Expr<'a> {
    Addition(Box<Expr<'a>>, Box<Expr<'a>>),
    Subtraction(Box<Expr<'a>>, Box<Expr<'a>>),
    Atomic(Atomic<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Atomic<'a> {
    Ident(&'a str),
    V4(V4),
    V6(V6),
}

fn process_v6_half(half: &str) -> (u128, u8) {
    if half.len() == 0 {
        // Is empty string
        return (0, 0);
    }
    let mut cnt = 0;
    let mut result = 0;
    for seg in half.split(":") {
        let s = u32::from_str_radix(seg, 16).unwrap();
        result = (result << 16) | (s as u128);
        cnt += 1;
    }
    (result, cnt)
}

fn map_expr<'a>(p: Pair<'a, Rule>) -> anyhow::Result<Expr<'a>> {
    // println!("Processing: {:?}", p.as_rule());
    // TODO: a million assertions
    match p.as_rule() {
        Rule::ident => Ok(Expr::Atomic(Atomic::Ident(p.as_str()))),
        Rule::v4cidr => {
            let mut collected: u32 = 0;
            let mut segs = p.as_str().split("/");
            let addr = segs.next().unwrap();
            let len = segs.next().unwrap();
            for seg in addr.split(".") {
                if seg.len() > 3 {
                    return Err(anyhow!("Number too big for v4 segment: {}", seg))
                }
                let parsed: u32 = seg.parse().unwrap();
                if parsed > 256 {
                    return Err(anyhow!("Number too big for v4 segment: {}", seg))
                }
                collected = collected << 8 | parsed;
            }
            if len.len() > 2 {
                return Err(anyhow!("Number too big for v4 CIDR length: {}", len))
            }
            let len_parsed: u32 = len.parse().unwrap();
            if len_parsed > 32 {
                return Err(anyhow!("Number too big for v4 CIDR length: {}", len))
            }
            Ok(Expr::Atomic(Atomic::V4(V4(collected, len_parsed as u8))))
        },
        Rule::v6cidr => {
            let mut split = p.as_str().split("/");
            let addr_str = split.next().unwrap();
            let len_str = split.next().unwrap();

            let mut halves = addr_str.split("::");
            let first_half = process_v6_half(halves.next().unwrap());
            let second_half = halves.next().map(process_v6_half);
            if halves.next().is_some() {
                return Err(anyhow!("IPv6 address containing more than one `::`: {}", p.as_str()))
            }
            let addr = if let Some(second_half) = second_half {
                if second_half.1 + first_half.1 > 8 {
                    return Err(anyhow!("IPv6 address containing too much specified segments: {}", p.as_str()))
                }
                let first_shifter = 8 - first_half.1;
                let collected = if first_shifter == 8 {
                    // First segment 0. Don't shift, because <<128 is UB
                    0
                } else {
                    first_half.0 << (first_shifter as i32 * 16)
                };
                collected | second_half.0
            } else {
                if first_half.1 != 8 {
                    return Err(anyhow!("IPv6 address containing too little specified segments: {}", p.as_str()))
                }
                first_half.0
            };
            if len_str.len() > 3 {
                return Err(anyhow!("Number too big for v6 CIDR length: {}", len_str))
            }
            let len_parsed: u32 = len_str.parse().unwrap();
            if len_parsed > 128 {
                return Err(anyhow!("Number too big for v6 CIDR length: {}", len_str))
            }

            Ok(Expr::Atomic(Atomic::V6(V6(addr, len_parsed as u8))))
        }
        Rule::expr => {
            let mut p = p.into_inner();
            let mut collected = map_expr(p.next().unwrap())?;
            while let Some(op) = p.next() {
                let rhs = map_expr(p.next().unwrap())?;
                if op.as_rule() == Rule::add_op {
                    collected = Expr::Addition(Box::new(collected), Box::new(rhs));
                } else {
                    collected = Expr::Subtraction(Box::new(collected), Box::new(rhs));
                }
            }
            Ok(collected)
        }
        Rule::atomic | Rule::expr => map_expr(p.into_inner().next().unwrap()),
        Rule::paren_expr => map_expr(p.into_inner().skip(1).next().unwrap()),
        e => unreachable!("Excuse me pest? Why am I reading {:?}?", e)
    }
}

fn map_stmt<'a>(p: Pair<'a, Rule>) -> anyhow::Result<Stmt<'a>> {
    assert_eq!(p.as_rule(), Rule::stmt);
    let p = p.into_inner().next().unwrap();
    match p.as_rule() {
        Rule::let_in => {
            let mut p = p.into_inner();
            let ident = p.next().unwrap().as_str();
            let val = Box::new(map_expr(p.next().unwrap())?);
            Ok(Stmt::LetIn { ident, val })
        },
        Rule::expr => {
            map_expr(p).map(Into::into)
        },
        e => unreachable!("Excuse me pest? Why am I reading {:?}?", e)
    }
}

pub fn parse_single<'a>(input: &'a str) -> anyhow::Result<Stmt<'a>> {
    let raw = SrcParser::parse(Rule::single_stmt, input)?.next().unwrap().into_inner().next().unwrap();
    map_stmt(raw)
}

pub fn parse<'a>(input: &'a str) -> anyhow::Result<Vec<Stmt<'a>>> {
    let raw = SrcParser::parse(Rule::multiple_stmt, input)?.next().unwrap();
    raw.into_inner().filter(|e| e.as_rule() != Rule::EOI).map(|p| map_stmt(p)).collect()
}

#[test]
fn test_parser() {
    let parsed = parse_single("0.0.0.0/0");
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), Stmt::Expr(Expr::Atomic(Atomic::V4(V4(0, 0)))));

    let parsed = parse_single("101.6.6.6/32");
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), Stmt::Expr(Expr::Atomic(Atomic::V4(V4(1694893574u32, 32)))));

    let parsed = parse_single("::/0");
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), Stmt::Expr(Expr::Atomic(Atomic::V6(V6(0, 0)))));

    let parsed = parse_single("::1/128");
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), Stmt::Expr(Expr::Atomic(Atomic::V6(V6(1, 128)))));

    let parsed = parse_single("2001:da8::666/24");
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), Stmt::Expr(Expr::Atomic(Atomic::V6(V6(42540765143631992628674583454950622822u128, 24)))));

    let example = r#"
    let meow = ::/0
    let meow_meow = 2001:da8::666/128
    meow + meow_meow - meow
    "#;
    let parsed = parse(example);
    // assert!(parsed.is_ok());
    println!("{:?}", parsed)
}