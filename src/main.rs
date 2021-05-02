#![allow(unused)]
mod parser {
    use std::convert::TryInto;

    type Parser<T> = Box<dyn Fn(&str) -> Option<(T, &str)>>;

    type Consumer  = Box<dyn Fn(&str) -> &str>;

    #[derive(Debug)]
    pub enum Expr {
        Symbol(String),
        List(Vec<Expr>),
    }

    use Expr::*;
    struct IterParse<'a, T> {
        parser: Parser<T>,
        s: &'a str,
    }

    impl<'a, T> Iterator for IterParse<'a, T> {
        type Item = (T, &'a str);

        fn next(&mut self) -> Option<(T, &'a str)> {
            let (expr, tail) = (self.parser)(self.s)?;
            self.s = tail;
            Some((expr, tail))
        }
    }

    fn parse_consume<T:'static>(parser: Parser<T>, consumer:Consumer) -> Parser<T> {
        Box::new(
            move |s: &str| {
                let (result, tail) = parser(s)?;
                Some((result, consumer(tail)))
            }
        )
    }

    fn parse_iter<T>(parser: Parser<T>, s: &str) -> IterParse<T> {
        IterParse { parser, s }
    }

    fn parse_multiple<T>(parser: Parser<T>, s: &str) -> Option<(Vec<T>, &str)> {
        let (expr, tail) =
            parse_iter(parser, s).fold((Vec::new(), s), |mut acc, (parsed, tail)| {
                acc.0.push(parsed);
                (acc.0, tail)
            });

        if tail == s {
            return None;
        }

        Some((expr, tail))
    }

    fn char_parser(fun: &'static dyn Fn(char) -> bool) -> Parser<char> {
        Box::new(move |s| {
            let c = s.chars().next()?;
            if fun(c) {
                None
            } else {
                Some((c, &s[c.len_utf8()..]))
            }
        })
    }

    fn symbol_character(s: &str) -> Option<(char, &str)> {
        char_parser(&|c| c.is_whitespace() || "(')".contains(c))(s)
    }

    fn whitespace(s: &str) -> Option<(char, &str)> {
        char_parser(&|c| !c.is_whitespace())(s)
    }

    pub fn symbol(s: &str) -> Option<(Expr, &str)> {
        let (parsed, tail) = parse_multiple(Box::new(symbol_character), s)?;
        Some((Symbol(parsed.into_iter().collect()), tail))
    }

    pub fn expr(s: &str) -> Option<(Expr, &str)> {
        let (parsed, tail) = symbol(s.trim()).or_else(|| sexpr(s.trim()))?;
        Some((parsed, tail))
    }

    pub fn expr_list(s: &str) -> Option<(Expr, &str)> {
        let (parsed, tail) = parse_multiple(
            Box::new(expr),
            s,
        )?;
        Some((List(parsed), tail))
    }

    pub fn sexpr(s: &str) -> Option<(Expr, &str)> {

        fn helper(s: &str, count: i32, index:usize) -> Option<usize>
        {
            let count  = count + match s.chars().next()? {
                '(' => 1,
                ')' => -1,
                _ => 0
            };

            if count == 0 {
                Some(index)
            } else {
            helper(&s[1..], count, index + 1)
            }
        }

        let offset = helper(s.trim(), 0, 0)?;

        Some((
            expr_list(&s[1..offset])?.0,
            &s[offset..],
        ))
    }
}
use std::io::{self, BufRead};
fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        print!("lispy>");
        println!("{:#?}", match parser::expr(&line.unwrap()[0..]){
            Some(t) => t,
            None => continue
        })
    }
}
