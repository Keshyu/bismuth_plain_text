use anyhow::{bail, Result};
use dbg_pls::DebugPls;

pub fn parse(source: &[Token]) -> Result<Expression> {
    Parser { source, index: 0 }.parse()
}

pub fn lex(source: &[char]) -> Result<Box<[Token]>> {
    Lexer { source, index: 0 }.lex()
}

#[derive(DebugPls)]
pub enum Expression {
    Group(Box<[Expression]>),
    Name(Box<str>),
}

#[derive(DebugPls)]
pub enum Token {
    GroupStart,
    GroupEnd,
    Joiner,
    Separator,
    Name(Box<str>),
    NameExtended(Box<str>),
}

struct Parser<'a> {
    source: &'a [Token],
    index: usize,
}

impl<'a> Parser<'a> {
    fn parse(mut self) -> Result<Expression> {
        let mut ancestry: Vec<Vec<Expression>> = vec![Vec::new()];

        // expr:  expr2 (S expr)
        // expr2: expr1 expr2
        // expr1: expr0 (J expr1)
        // expr0: NAME | NAME_EX | START expr END

        while let Some(token) = self.next() {
            let expression = match token {
                Token::Name(n) => Expression::Name(n.clone()),
                Token::NameExtended(n) => Expression::Name(n.clone()),
                Token::GroupStart => {
                    ancestry.push(Vec::new());
                    continue;
                }
                Token::GroupEnd => {
                    if ancestry.len() < 2 {
                        bail!("unmatched '}}'")
                    }
                    Expression::Group(ancestry.pop().unwrap().into())
                }
                Token::Separator => {
                    while matches!(self.next(), Some(Token::Separator | Token::Joiner)) {}
                    self.go_back();

                    let e = Expression::Group(ancestry.pop().unwrap().into());

                    if matches!(ancestry.last(), None) {
                        ancestry.push(Vec::new());
                    }

                    ancestry.last_mut().unwrap().push(e);
                    ancestry.push(Vec::new());

                    continue;
                }
                Token::Joiner => {
                    let Some(e) = ancestry.last_mut().unwrap().pop() else {
                        continue;
                    };
                    ancestry.push(vec![e]);
                    continue;
                }
            };
            ancestry.last_mut().unwrap().push(expression);
        }

        if ancestry.len() > 1 {
            bail!("unmatched '{{' (one or more)")
        }
        debug_assert!(ancestry.len() == 1);

        Ok(Expression::Group(ancestry.pop().unwrap().into()))
    }

    fn next(&mut self) -> Option<&Token> {
        let it = self.source.get(self.index);
        self.index += 1;
        it
    }

    fn go_back(&mut self) {
        self.index -= 1;
    }
}

struct Lexer<'a> {
    source: &'a [char],
    index: usize,
}

impl<'a> Lexer<'a> {
    fn lex(mut self) -> Result<Box<[Token]>> {
        let mut tokens = Vec::new();

        while let Some(c) = self.next() {
            let token = match c {
                '{' => Token::GroupStart,
                '}' => Token::GroupEnd,
                '.' => Token::Joiner,
                ';' => Token::Separator,
                '\n' => Token::Separator,
                '"' => {
                    let mut name = String::new();
                    let mut terminated = false;
                    while let Some(c) = self.next() {
                        if c == '"' {
                            terminated = true;
                            break;
                        }
                        name.push(c);
                    }
                    if !terminated {
                        bail!("unterminated extended name")
                    }
                    Token::NameExtended(name.into())
                }
                _ if c.is_whitespace() => continue,
                _ => {
                    let mut name = String::new();
                    name.push(c);
                    while let Some(c) = self.next() {
                        if ['{', '}', '.', ';'].contains(&c) || c.is_whitespace() {
                            self.go_back();
                            break;
                        }
                        name.push(c);
                    }
                    Token::Name(name.into())
                }
            };
            tokens.push(token);
        }

        Ok(tokens.into())
    }

    fn next(&mut self) -> Option<char> {
        let it = self.source.get(self.index).copied();
        self.index += 1;
        it
    }

    fn go_back(&mut self) {
        self.index -= 1;
    }
}
