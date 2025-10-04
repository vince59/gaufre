use crate::grammar;
use crate::lexer::{LexError, Lexer, Pos, Token};

use std::fmt;

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Log(Vec<Expr>),
    Call { name: String },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Str(String),
    Var(String),
    Int(i32),
    Add(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum ParseError {
    Lex(LexError),
    Unexpected {
        found: Token,
        expected: &'static str,
        pos: Pos,
    },
    IntOverflow {
        literal: String,
        pos: Pos,
    },
}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        Self::Lex(e)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lex(e) => write!(f, "{}", e),
            Self::Unexpected {
                found,
                expected,
                pos,
            } => write!(
                f,
                "{}:{}:{}: Expected {}, found {:?}",
                pos.file, pos.line, pos.col, expected, found
            ),
            Self::IntOverflow { literal, pos } => write!(
                f,
                "{}:{}:{}: Entier hors plage i32: {}",
                pos.file, pos.line, pos.col, literal
            ),
        }
    }
}
impl std::error::Error for ParseError {}

pub struct Parser<'a> {
    lx: Lexer<'a>, // lexer
    cur: Token,    // current token
    cur_pos: Pos,  // curent position
}

impl<'a> Parser<'a> {
    pub fn new(mut lx: Lexer<'a>) -> Result<Self, ParseError> {
        let (cur, cur_pos) = lx.next_token()?;
        Ok(Self { lx, cur, cur_pos })
    }

    // Move one token forward
    fn bump(&mut self) -> Result<(), ParseError> {
        (self.cur, self.cur_pos) = self.lx.next_token()?;
        Ok(())
    }

    // Checks if the current token matches the expected value; otherwise, it returns an error
    fn expect(&mut self, want: Token, name: &'static str) -> Result<(), ParseError> {
        if std::mem::discriminant(&self.cur) == std::mem::discriminant(&want) {
            self.bump()?;
            Ok(())
        } else {
            Err(ParseError::Unexpected {
                found: self.cur.clone(),
                expected: name,
                pos: self.cur_pos.clone(),
            })
        }
    }

    // import "string"
    // fn main() {}
    pub fn parse_main_program(&mut self) -> Result<(Vec<String>, Program), ParseError> {
        let imports = self.parse_imports()?;
        // fn main() { ... }
        self.expect(Token::Fn, grammar::KW_FN)?;
        self.expect(Token::Main, grammar::KW_MAIN)?;
        self.expect(Token::LParen, grammar::LPAREN)?;
        self.expect(Token::RParen, grammar::RPAREN)?;
        self.expect(Token::LBrace, grammar::LBRACE)?;
        let mut stmts = Vec::new();
        while !matches!(self.cur, Token::RBrace) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(Token::RBrace, grammar::RBRACE)?;
        self.expect(Token::Eof, grammar::EOF)?;
        Ok((imports, Program { stmts }))
    }

    /// Read import and return the path to the import, zero import is allowed
    pub fn parse_imports(&mut self) -> Result<Vec<String>, ParseError> {
        let mut paths = Vec::new();
        loop {
            match self.cur {
                Token::Import => {
                    self.bump()?; // 'import'
                    if let Token::Str(s) = &self.cur {
                        paths.push(s.clone());
                        self.bump()?; // string
                    } else {
                        return Err(ParseError::Unexpected {
                            found: self.cur.clone(),
                            expected: "a path string after `import`",
                            pos: self.cur_pos.clone(),
                        });
                    }
                }
                _ => break,
            }
        }
        Ok(paths)
    }

    // parse the log primitive : log(" string ")
    fn parse_log(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::Log, grammar::KW_LOG)?;
        self.expect(Token::LParen, grammar::LPAREN)?;
        // Get the string
        let s = if let Token::Str(txt) = &self.cur {
            let out = txt.clone();
            self.bump()?; // eat the string
            out
        } else {
            return Err(ParseError::Unexpected {
                found: self.cur.clone(),
                expected: "a string \"...\" after log(",
                pos: self.cur_pos.clone(),
            });
        };
        self.expect(Token::RParen, grammar::RPAREN)?;
        Ok(Stmt::Log(vec![Expr::Str(s)]))
    }

    // parse imported files (sub programs)
    pub fn parse_sub_programs(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !matches!(self.cur, Token::Eof) {
            // interdit explicitement tout `import` dans un fichier inclus
            if matches!(self.cur, Token::Import) {
                return Err(ParseError::Unexpected {
                    found: self.cur.clone(),
                    expected: "no `import` in an included file (only in main program)",
                    pos: self.cur_pos.clone(),
                });
            }
            stmts.push(self.parse_stmt()?);
        }
        self.expect(Token::Eof, grammar::EOF)?;
        Ok(stmts)
    }

    // call <ident>()
    fn parse_call(&mut self) -> Result<Stmt, ParseError> {
        self.expect(Token::Call, crate::grammar::KW_CALL)?;
        // nom de fonction
        let name = if let Token::Ident(s) = &self.cur {
            let n = s.clone();
            self.bump()?;
            n
        } else {
            return Err(ParseError::Unexpected {
                found: self.cur.clone(),
                expected: "function name after `call`",
                pos: self.cur_pos.clone(),
            });
        };
        self.expect(Token::LParen, crate::grammar::LPAREN)?;
        self.expect(Token::RParen, crate::grammar::RPAREN)?;
        Ok(Stmt::Call { name })
    }

    // Parse `(){ ... }` and return the vector stadment
    fn parse_fn_body_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(Token::LParen, crate::grammar::LPAREN)?;
        self.expect(Token::RParen, crate::grammar::RPAREN)?;
        self.expect(Token::LBrace, crate::grammar::LBRACE)?;
        let mut body = Vec::new();
        while !matches!(self.cur, Token::RBrace) {
            body.push(self.parse_stmt()?);
        }
        self.expect(Token::RBrace, crate::grammar::RBRACE)?;
        Ok(body)
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        let name = if let Token::Ident(s) = &self.cur {
            let n = s.clone();
            self.bump()?;
            n
        } else if matches!(self.cur, Token::Main) {
            return Err(ParseError::Unexpected {
                found: self.cur.clone(),
                expected: "function (hors `main`)",
                pos: self.cur_pos.clone(),
            });
        } else {
            return Err(ParseError::Unexpected {
                found: self.cur.clone(),
                expected: "nom de fonction",
                pos: self.cur_pos.clone(),
            });
        };

        let body = self.parse_fn_body_block()?;
        Ok(Function { name, body })
    }

    // parse a stadment
    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match &self.cur {
            Token::Call => self.parse_call(),
            Token::Log => self.parse_log(),
            _ => Err(ParseError::Unexpected {
                found: self.cur.clone(),
                expected: "`log`",
                pos: self.cur_pos.clone(),
            }),
        }
    }
}
