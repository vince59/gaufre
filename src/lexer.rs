use crate::grammar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Import,
    Fn,
    Main,
    Log,
    Call,
    Ident(String),
    Number(String),
    Str(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Pos {
    pub byte: usize,  // index byte
    pub line: usize,  // line source code
    pub col: usize,   // column source code
    pub file: String, // source file name
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub pos: Pos,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.pos.file, self.pos.line, self.pos.col, self.message
        )
    }
}
impl std::error::Error for LexError {}
pub struct Lexer<'a> {
    input: &'a str, // source code
    i: usize,       // index byte
    line: usize,    // line source code
    col: usize,     // column source code
    file: String,   // source file name
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self::with_file("<stdin>", input)
    }

    pub fn with_file(file: impl Into<String>, input: &'a str) -> Self {
        Self {
            input,
            i: 0,
            line: 1,
            col: 1,
            file: file.into(),
        }
    }

    // check en of file
    fn eof(&self) -> bool {
        self.i >= self.input.len()
    }

    // see the next byte without increase the cursor
    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.i).copied()
    }

    // return the next byte and increase the cursor
    fn bump(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.i += 1;
        if b == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(b)
    }

    // skip spaces and other separators
    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            match b {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    self.bump();
                }
                _ => break,
            }
        }
    }

    // check if the input starts with the searched token
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.i..].starts_with(s)
    }

    // checks if the next token is the one being searched for (s)
    fn try_take(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.i += s.len();
            self.col += s.len();
            true
        } else {
            false
        }
    }

    // try to see if the next token is a symbol
    fn try_symbol(&mut self) -> Option<Token> {
        if self.try_take(grammar::LPAREN) {
            return Some(Token::LParen);
        }
        if self.try_take(grammar::RPAREN) {
            return Some(Token::RParen);
        }
        if self.try_take(grammar::LBRACE) {
            return Some(Token::LBrace);
        }
        if self.try_take(grammar::RBRACE) {
            return Some(Token::RBrace);
        }
        if self.try_take(grammar::COMMA) {
            return Some(Token::Comma);
        }
        None
    }

    fn get_pos(&self) -> Pos {
        Pos {
            file: self.file.clone(),
            byte: self.i,
            line: self.line,
            col: self.col,
        }
    }

    // read a valid string
    fn read_string(&mut self) -> Result<Token, LexError> {
        let start_byte = self.i;
        let start_line = self.line;
        let start_col = self.col;
        self.bump(); // "
        let s = self.i;
        while let Some(b) = self.peek() {
            if b == b'"' {
                let out = &self.input[s..self.i];
                self.bump();
                return Ok(Token::Str(out.to_string()));
            }
            self.bump();
        }
        Err(LexError {
            message: "incomplete string (\" missing)".into(),
            pos: self.get_pos(),
        })
    }

    // ident can start with a upper or lower case letter or underscore
    fn is_ident_start(b: u8) -> bool {
        (b'a'..=b'z').contains(&b) || (b'A'..=b'Z').contains(&b) || b == b'_'
    }

    // check the next characters of the ident same as ident_start plus digits
    fn is_ident_continue(b: u8) -> bool {
        Self::is_ident_start(b) || (b'0'..=b'9').contains(&b)
    }

    fn read_ident(&mut self) -> (&'a str, usize, usize) {
        let s = self.i;
        while let Some(b) = self.peek() {
            if Self::is_ident_continue(b) {
                self.bump();
            } else {
                break;
            }
        }
        (&self.input[s..self.i], s, self.i) // return the ident, start and end position
    }

    fn read_number(&mut self) -> (&'a str, usize, usize) {
        let s = self.i;
        while let Some(b) = self.peek() {
            if (b'0'..=b'9').contains(&b) {
                self.bump();
            } else {
                break;
            }
        }
        (&self.input[s..self.i], s, self.i) // return the number, start and end position
    }

    // get next valid token
    pub fn next_token(&mut self) -> Result<(Token, Pos), LexError> {
        self.skip_ws();
        let pos = Pos {
            file: self.file.clone(),
            byte: self.i,
            line: self.line,
            col: self.col,
        };
        if self.eof() {
            return Ok((Token::Eof, pos));
        }
        if let Some(t) = self.try_symbol() {
            return Ok((t,pos));
        }
        if self.peek() == Some(b'"') {
            return Ok((self.read_string()?, pos));
        }
        // check if the token is an ident or a keyword                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   
        if let Some(b) = self.peek() {
            if Self::is_ident_start(b) {
                let (id, _, _) = self.read_ident();
                return Ok((
                    match id {
                        // check if the id is a key word
                        grammar::KW_IMPORT => Token::Import,
                        grammar::KW_CALL => Token::Call,
                        grammar::KW_FN => Token::Fn,
                        grammar::KW_MAIN => Token::Main,
                        grammar::KW_LOG => Token::Log,
                        _ => Token::Ident(id.to_string()), // if not it is an ident
                    },
                    pos,
                ));
            }
            // check if the token is a number
            if (b'0'..=b'9').contains(&b) {
                let (n, _, _) = self.read_number();
                return Ok((Token::Number(n.to_string()), pos));
            }
        }

        Err(LexError {
            message: format!("caractère inattendu: 0x{:02X}", self.peek().unwrap()),
            pos: self.get_pos(),
        })
    }
}
