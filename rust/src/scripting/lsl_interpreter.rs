//! LSL language interpreter and parser

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tracing::{debug, warn};

use super::{LSLRotation, LSLValue, LSLVector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceLocation {
    pub line: u32,
    pub col: u32,
}

/// LSL Token types
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i32),
    Float(f32),
    String(String),
    Identifier(String),

    // Keywords
    Default,
    State,
    If,
    Else,
    For,
    While,
    Do,
    Return,
    Jump,

    // Types
    IntegerType,
    FloatType,
    StringType,
    KeyType,
    VectorType,
    RotationType,
    ListType,

    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Assign,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Not,

    // Bitwise operators
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    ShiftLeft,
    ShiftRight,

    // Compound assignment
    PlusAssign,
    MinusAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,

    // Increment/Decrement
    Increment,
    Decrement,

    // Additional
    At,
    Dot,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,

    // Special
    Newline,
    Eof,
}

#[derive(Debug, Clone)]
pub enum ASTNode {
    Program(Vec<ASTNode>),
    State {
        name: String,
        body: Vec<ASTNode>,
    },
    Function {
        name: String,
        return_type: String,
        parameters: Vec<(String, String)>,
        body: Vec<ASTNode>,
    },
    Event {
        name: String,
        parameters: Vec<(String, String)>,
        body: Vec<ASTNode>,
    },
    Variable {
        var_type: String,
        name: String,
        value: Option<Box<ASTNode>>,
    },
    Assignment {
        target: Box<ASTNode>,
        value: Box<ASTNode>,
    },
    CompoundAssignment {
        target: Box<ASTNode>,
        operator: Token,
        value: Box<ASTNode>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<ASTNode>,
    },
    If {
        condition: Box<ASTNode>,
        then_body: Vec<ASTNode>,
        else_body: Option<Vec<ASTNode>>,
    },
    While {
        condition: Box<ASTNode>,
        body: Vec<ASTNode>,
    },
    DoWhile {
        body: Vec<ASTNode>,
        condition: Box<ASTNode>,
    },
    For {
        init: Option<Box<ASTNode>>,
        condition: Option<Box<ASTNode>>,
        increment: Option<Box<ASTNode>>,
        body: Vec<ASTNode>,
    },
    Return(Option<Box<ASTNode>>),
    Label(String),
    Jump(String),
    StateChange(String),
    BinaryOp {
        left: Box<ASTNode>,
        operator: Token,
        right: Box<ASTNode>,
    },
    UnaryOp {
        operator: Token,
        operand: Box<ASTNode>,
    },
    PreIncrement(Box<ASTNode>),
    PreDecrement(Box<ASTNode>),
    PostIncrement(Box<ASTNode>),
    PostDecrement(Box<ASTNode>),
    TypeCast {
        target_type: String,
        expr: Box<ASTNode>,
    },
    MemberAccess {
        object: Box<ASTNode>,
        member: String,
    },
    VectorLiteral {
        x: Box<ASTNode>,
        y: Box<ASTNode>,
        z: Box<ASTNode>,
    },
    RotationLiteral {
        x: Box<ASTNode>,
        y: Box<ASTNode>,
        z: Box<ASTNode>,
        s: Box<ASTNode>,
    },
    ListLiteral(Vec<ASTNode>),
    Literal(LSLValue),
    Identifier(String),
    Block(Vec<ASTNode>),
}

/// LSL Lexer for tokenizing source code
pub struct LSLLexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    line: u32,
    col: u32,
}

impl LSLLexer {
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();

        Self {
            input: chars,
            position: 0,
            current_char,
            line: 1,
            col: 1,
        }
    }

    pub fn location(&self) -> SourceLocation {
        SourceLocation {
            line: self.line,
            col: self.col,
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if (ch.is_whitespace() && ch != '\n') || ch == '\u{FEFF}' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip line comments
    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Skip block comments
    fn skip_block_comment(&mut self) -> Result<()> {
        self.advance(); // Skip '/'
        self.advance(); // Skip '*'

        while let Some(ch) = self.current_char {
            if ch == '*' {
                self.advance();
                if let Some('/') = self.current_char {
                    self.advance();
                    return Ok(());
                }
            } else {
                self.advance();
            }
        }

        Err(anyhow!("Unterminated block comment"))
    }

    fn read_number(&mut self) -> Result<Token> {
        let mut number = String::new();
        let mut is_float = false;

        if self.current_char == Some('0') {
            if let Some(next) = self.peek() {
                if next == 'x' || next == 'X' {
                    self.advance();
                    self.advance();
                    while let Some(ch) = self.current_char {
                        if ch.is_ascii_hexdigit() {
                            number.push(ch);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    if number.is_empty() {
                        return Ok(Token::Integer(0));
                    }
                    let value = i32::from_str_radix(&number, 16).unwrap_or(0);
                    return Ok(Token::Integer(value));
                }
            }
        }

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                number.push(ch);
                self.advance();
            } else if (ch == 'e' || ch == 'E') && !number.is_empty() {
                is_float = true;
                number.push(ch);
                self.advance();
                if self.current_char == Some('+') || self.current_char == Some('-') {
                    number.push(self.current_char.unwrap());
                    self.advance();
                }
            } else {
                break;
            }
        }

        if self.current_char == Some('f') || self.current_char == Some('F') {
            is_float = true;
            self.advance();
        }

        if is_float {
            let value = number
                .parse::<f32>()
                .map_err(|_| anyhow!("Invalid float: {}", number))?;
            Ok(Token::Float(value))
        } else {
            let value = number.parse::<i32>().unwrap_or(0);
            Ok(Token::Integer(value))
        }
    }

    fn read_dot_float(&mut self) -> Result<Token> {
        self.advance(); // skip '.'
        let mut number = String::from("0.");
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if let Some(ch) = self.current_char {
            if ch == 'e' || ch == 'E' {
                number.push(ch);
                self.advance();
                if self.current_char == Some('+') || self.current_char == Some('-') {
                    number.push(self.current_char.unwrap());
                    self.advance();
                }
                while let Some(ch) = self.current_char {
                    if ch.is_ascii_digit() {
                        number.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
        }
        if self.current_char == Some('f') || self.current_char == Some('F') {
            self.advance();
        }
        let value = number.parse::<f32>().unwrap_or(0.0);
        Ok(Token::Float(value))
    }

    /// Read a string literal
    fn read_string(&mut self) -> Result<Token> {
        let mut string = String::new();
        self.advance(); // Skip opening quote

        while let Some(ch) = self.current_char {
            if ch == '"' {
                self.advance(); // Skip closing quote
                return Ok(Token::String(string));
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char {
                    match escaped {
                        'n' => string.push('\n'),
                        't' => string.push('\t'),
                        'r' => string.push('\r'),
                        '\\' => string.push('\\'),
                        '"' => string.push('"'),
                        _ => {
                            string.push('\\');
                            string.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else {
                string.push(ch);
                self.advance();
            }
        }

        Err(anyhow!("Unterminated string literal"))
    }

    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> Token {
        let mut identifier = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for keywords
        match identifier.as_str() {
            "default" => Token::Default,
            "state" => Token::State,
            "if" => Token::If,
            "else" => Token::Else,
            "for" => Token::For,
            "while" => Token::While,
            "do" => Token::Do,
            "return" => Token::Return,
            "jump" => Token::Jump,
            "integer" => Token::IntegerType,
            "float" => Token::FloatType,
            "string" => Token::StringType,
            "key" => Token::KeyType,
            "vector" => Token::VectorType,
            "rotation" => Token::RotationType,
            "list" => Token::ListType,
            _ => Token::Identifier(identifier),
        }
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token> {
        loop {
            match self.current_char {
                None => return Ok(Token::Eof),
                Some(' ') | Some('\t') | Some('\r') => {
                    self.skip_whitespace();
                    continue;
                }
                Some('\n') => {
                    self.advance();
                    return Ok(Token::Newline);
                }
                Some('/') => {
                    if let Some(next) = self.peek() {
                        if next == '/' {
                            self.skip_line_comment();
                            continue;
                        } else if next == '*' {
                            self.skip_block_comment()?;
                            continue;
                        }
                    }
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token::DivideAssign);
                    }
                    return Ok(Token::Divide);
                }
                Some(ch) if ch.is_ascii_digit() => {
                    return self.read_number();
                }
                Some('"') => {
                    return self.read_string();
                }
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    return Ok(self.read_identifier());
                }
                Some('+') => {
                    self.advance();
                    if self.current_char == Some('+') {
                        self.advance();
                        return Ok(Token::Increment);
                    } else if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token::PlusAssign);
                    }
                    return Ok(Token::Plus);
                }
                Some('-') => {
                    self.advance();
                    if self.current_char == Some('-') {
                        self.advance();
                        return Ok(Token::Decrement);
                    } else if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token::MinusAssign);
                    }
                    return Ok(Token::Minus);
                }
                Some('*') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token::MultiplyAssign);
                    }
                    return Ok(Token::Multiply);
                }
                Some('%') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token::ModuloAssign);
                    }
                    return Ok(Token::Modulo);
                }
                Some('=') => {
                    self.advance();
                    if let Some('=') = self.current_char {
                        self.advance();
                        return Ok(Token::Equal);
                    }
                    return Ok(Token::Assign);
                }
                Some('!') => {
                    self.advance();
                    if let Some('=') = self.current_char {
                        self.advance();
                        return Ok(Token::NotEqual);
                    }
                    return Ok(Token::Not);
                }
                Some('<') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token::LessEqual);
                    } else if self.current_char == Some('<') {
                        self.advance();
                        return Ok(Token::ShiftLeft);
                    }
                    return Ok(Token::Less);
                }
                Some('>') => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token::GreaterEqual);
                    } else if self.current_char == Some('>') {
                        self.advance();
                        return Ok(Token::ShiftRight);
                    }
                    return Ok(Token::Greater);
                }
                Some('&') => {
                    self.advance();
                    if self.current_char == Some('&') {
                        self.advance();
                        return Ok(Token::And);
                    }
                    return Ok(Token::BitAnd);
                }
                Some('|') => {
                    self.advance();
                    if self.current_char == Some('|') {
                        self.advance();
                        return Ok(Token::Or);
                    }
                    return Ok(Token::BitOr);
                }
                Some('^') => {
                    self.advance();
                    return Ok(Token::BitXor);
                }
                Some('~') => {
                    self.advance();
                    return Ok(Token::BitNot);
                }
                Some('@') => {
                    self.advance();
                    return Ok(Token::At);
                }
                Some('.') => {
                    if let Some(next) = self.peek() {
                        if next.is_ascii_digit() {
                            return self.read_dot_float();
                        }
                    }
                    self.advance();
                    return Ok(Token::Dot);
                }
                Some('(') => {
                    self.advance();
                    return Ok(Token::LeftParen);
                }
                Some(')') => {
                    self.advance();
                    return Ok(Token::RightParen);
                }
                Some('{') => {
                    self.advance();
                    return Ok(Token::LeftBrace);
                }
                Some('}') => {
                    self.advance();
                    return Ok(Token::RightBrace);
                }
                Some('[') => {
                    self.advance();
                    return Ok(Token::LeftBracket);
                }
                Some(']') => {
                    self.advance();
                    return Ok(Token::RightBracket);
                }
                Some(';') => {
                    self.advance();
                    return Ok(Token::Semicolon);
                }
                Some(',') => {
                    self.advance();
                    return Ok(Token::Comma);
                }
                Some(':') | Some('\'') | Some('\u{FEFF}') | Some('?') | Some('$') | Some('\\') => {
                    self.advance();
                    continue;
                }
                Some('#') => {
                    self.skip_line_comment();
                    continue;
                }
                Some(ch) => {
                    return Err(anyhow!("Unexpected character: {}", ch));
                }
            }
        }
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token, Token::Eof);
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }
}

/// LSL Parser for building AST from tokens
pub struct LSLParser {
    tokens: Vec<Token>,
    position: usize,
    current_token: Token,
    in_vector_depth: usize,
}

impl LSLParser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let current_token = tokens.get(0).cloned().unwrap_or(Token::Eof);

        Self {
            tokens,
            position: 0,
            current_token,
            in_vector_depth: 0,
        }
    }

    /// Advance to the next token
    fn advance(&mut self) {
        self.position += 1;
        self.current_token = self
            .tokens
            .get(self.position)
            .cloned()
            .unwrap_or(Token::Eof);
    }

    /// Check if current token matches expected type
    fn skip_newlines(&mut self) {
        while matches!(self.current_token, Token::Newline) {
            self.advance();
        }
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        self.skip_newlines();
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(anyhow!(
                "Expected {:?}, found {:?}",
                expected,
                self.current_token
            ))
        }
    }

    /// Parse the entire program
    pub fn parse(&mut self) -> Result<ASTNode> {
        let mut statements = Vec::new();

        while !matches!(self.current_token, Token::Eof) {
            if matches!(self.current_token, Token::Newline) {
                self.advance();
                continue;
            }

            statements.push(self.parse_top_level()?);
        }

        Ok(ASTNode::Program(statements))
    }

    /// Parse top-level declarations (states, functions, global variables)
    fn parse_top_level(&mut self) -> Result<ASTNode> {
        match &self.current_token {
            Token::Default => self.parse_state(),
            Token::State => self.parse_state(),
            Token::IntegerType
            | Token::FloatType
            | Token::StringType
            | Token::KeyType
            | Token::VectorType
            | Token::RotationType
            | Token::ListType => self.parse_global_declaration(),
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                self.skip_newlines();
                if matches!(self.current_token, Token::LeftParen) {
                    self.parse_function("void".to_string(), name)
                } else {
                    Err(anyhow!("Unexpected identifier at top level: {}", name))
                }
            }
            _ => Err(anyhow!(
                "Unexpected token at top level: {:?}",
                self.current_token
            )),
        }
    }

    /// Parse a state declaration
    fn parse_state(&mut self) -> Result<ASTNode> {
        let state_name = if matches!(self.current_token, Token::Default) {
            self.advance();
            "default".to_string()
        } else {
            self.expect(Token::State)?;
            if let Token::Identifier(name) = &self.current_token {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(anyhow!("Expected state name"));
            }
        };

        self.expect(Token::LeftBrace)?;

        let mut body = Vec::new();
        while !matches!(self.current_token, Token::RightBrace)
            && !matches!(self.current_token, Token::Eof)
        {
            if matches!(self.current_token, Token::Newline) {
                self.advance();
                continue;
            }

            body.push(self.parse_event_or_function()?);
        }

        self.expect(Token::RightBrace)?;

        Ok(ASTNode::State {
            name: state_name,
            body,
        })
    }

    /// Parse global variable or function declaration
    fn parse_global_declaration(&mut self) -> Result<ASTNode> {
        let type_name = match &self.current_token {
            Token::IntegerType => {
                self.advance();
                "integer".to_string()
            }
            Token::FloatType => {
                self.advance();
                "float".to_string()
            }
            Token::StringType => {
                self.advance();
                "string".to_string()
            }
            Token::KeyType => {
                self.advance();
                "key".to_string()
            }
            Token::VectorType => {
                self.advance();
                "vector".to_string()
            }
            Token::RotationType => {
                self.advance();
                "rotation".to_string()
            }
            Token::ListType => {
                self.advance();
                "list".to_string()
            }
            _ => return Err(anyhow!("Expected type name")),
        };
        self.skip_newlines();

        if let Token::Identifier(name) = &self.current_token {
            let name = name.clone();
            self.advance();
            self.skip_newlines();

            match &self.current_token {
                Token::LeftParen => {
                    // Function declaration
                    self.parse_function(type_name, name)
                }
                Token::Assign | Token::Semicolon => {
                    // Variable declaration
                    let value = if matches!(self.current_token, Token::Assign) {
                        self.advance();
                        Some(Box::new(self.parse_expression()?))
                    } else {
                        None
                    };

                    self.expect(Token::Semicolon)?;

                    Ok(ASTNode::Variable {
                        var_type: type_name,
                        name,
                        value,
                    })
                }
                _ => Err(anyhow!("Expected '=' or ';' after variable name")),
            }
        } else {
            Err(anyhow!("Expected identifier after type"))
        }
    }

    /// Parse function declaration
    fn parse_function(&mut self, return_type: String, name: String) -> Result<ASTNode> {
        self.expect(Token::LeftParen)?;
        self.skip_newlines();

        let mut parameters = Vec::new();

        while !matches!(self.current_token, Token::RightParen) {
            self.skip_newlines();
            let param_type = match &self.current_token {
                Token::IntegerType => {
                    self.advance();
                    "integer".to_string()
                }
                Token::FloatType => {
                    self.advance();
                    "float".to_string()
                }
                Token::StringType => {
                    self.advance();
                    "string".to_string()
                }
                Token::KeyType => {
                    self.advance();
                    "key".to_string()
                }
                Token::VectorType => {
                    self.advance();
                    "vector".to_string()
                }
                Token::RotationType => {
                    self.advance();
                    "rotation".to_string()
                }
                Token::ListType => {
                    self.advance();
                    "list".to_string()
                }
                _ => return Err(anyhow!("Expected parameter type")),
            };
            self.skip_newlines();

            if let Token::Identifier(param_name) = &self.current_token {
                let param_name = param_name.clone();
                self.advance();
                self.skip_newlines();
                parameters.push((param_name, param_type));

                if matches!(self.current_token, Token::Comma) {
                    self.advance();
                    self.skip_newlines();
                } else if !matches!(self.current_token, Token::RightParen) {
                    return Err(anyhow!("Expected ',' or ')' in parameter list"));
                }
            } else {
                return Err(anyhow!("Expected parameter name"));
            }
        }

        self.expect(Token::RightParen)?;
        self.expect(Token::LeftBrace)?;

        let body = self.parse_block()?;

        Ok(ASTNode::Function {
            name,
            return_type,
            parameters,
            body,
        })
    }

    /// Parse event or function inside a state
    fn parse_event_or_function(&mut self) -> Result<ASTNode> {
        if let Token::Identifier(name) = &self.current_token {
            let name = name.clone();
            self.advance();

            self.expect(Token::LeftParen)?;
            self.skip_newlines();

            let mut parameters = Vec::new();

            while !matches!(self.current_token, Token::RightParen) {
                self.skip_newlines();
                let param_type = match &self.current_token {
                    Token::IntegerType => {
                        self.advance();
                        "integer".to_string()
                    }
                    Token::FloatType => {
                        self.advance();
                        "float".to_string()
                    }
                    Token::StringType => {
                        self.advance();
                        "string".to_string()
                    }
                    Token::KeyType => {
                        self.advance();
                        "key".to_string()
                    }
                    Token::VectorType => {
                        self.advance();
                        "vector".to_string()
                    }
                    Token::RotationType => {
                        self.advance();
                        "rotation".to_string()
                    }
                    Token::ListType => {
                        self.advance();
                        "list".to_string()
                    }
                    _ => return Err(anyhow!("Expected parameter type")),
                };
                self.skip_newlines();

                if let Token::Identifier(param_name) = &self.current_token {
                    let param_name = param_name.clone();
                    self.advance();
                    self.skip_newlines();
                    parameters.push((param_name, param_type));

                    if matches!(self.current_token, Token::Comma) {
                        self.advance();
                        self.skip_newlines();
                    } else if !matches!(self.current_token, Token::RightParen) {
                        return Err(anyhow!("Expected ',' or ')' in parameter list"));
                    }
                } else {
                    return Err(anyhow!("Expected parameter name"));
                }
            }

            self.expect(Token::RightParen)?;
            self.expect(Token::LeftBrace)?;

            let body = self.parse_block()?;

            Ok(ASTNode::Event {
                name,
                parameters,
                body,
            })
        } else {
            Err(anyhow!("Expected event or function name"))
        }
    }

    /// Parse a block of statements
    fn parse_block(&mut self) -> Result<Vec<ASTNode>> {
        let mut statements = Vec::new();

        while !matches!(self.current_token, Token::RightBrace)
            && !matches!(self.current_token, Token::Eof)
        {
            if matches!(self.current_token, Token::Newline) {
                self.advance();
                continue;
            }

            statements.push(self.parse_statement()?);
        }

        self.expect(Token::RightBrace)?;
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<ASTNode> {
        self.skip_newlines();
        match &self.current_token {
            Token::Semicolon => {
                self.advance();
                Ok(ASTNode::Literal(LSLValue::Integer(0)))
            }
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Do => self.parse_do_while(),
            Token::For => self.parse_for(),
            Token::Return => self.parse_return(),
            Token::Jump => {
                self.advance();
                if let Token::Identifier(label) = &self.current_token {
                    let label = label.clone();
                    self.advance();
                    self.expect(Token::Semicolon)?;
                    Ok(ASTNode::Jump(label))
                } else {
                    Err(anyhow!("Expected label name after 'jump'"))
                }
            }
            Token::At => {
                self.advance();
                if let Token::Identifier(label) = &self.current_token {
                    let label = label.clone();
                    self.advance();
                    self.expect(Token::Semicolon)?;
                    Ok(ASTNode::Label(label))
                } else {
                    Err(anyhow!("Expected label name after '@'"))
                }
            }
            Token::State => {
                self.advance();
                if let Token::Identifier(name) = &self.current_token {
                    let name = name.clone();
                    self.advance();
                    self.expect(Token::Semicolon)?;
                    Ok(ASTNode::StateChange(name))
                } else if matches!(self.current_token, Token::Default) {
                    self.advance();
                    self.expect(Token::Semicolon)?;
                    Ok(ASTNode::StateChange("default".to_string()))
                } else {
                    Err(anyhow!("Expected state name after 'state'"))
                }
            }
            Token::LeftBrace => {
                self.advance();
                let statements = self.parse_block()?;
                Ok(ASTNode::Block(statements))
            }
            Token::IntegerType
            | Token::FloatType
            | Token::StringType
            | Token::KeyType
            | Token::VectorType
            | Token::RotationType
            | Token::ListType => self.parse_local_variable(),
            _ => {
                let expr = self.parse_expression()?;
                self.expect(Token::Semicolon)?;
                Ok(expr)
            }
        }
    }

    fn parse_do_while(&mut self) -> Result<ASTNode> {
        self.expect(Token::Do)?;

        let body = if matches!(self.current_token, Token::LeftBrace) {
            self.advance();
            self.parse_block()?
        } else {
            vec![self.parse_statement()?]
        };

        self.expect(Token::While)?;
        self.expect(Token::LeftParen)?;
        let condition = Box::new(self.parse_expression()?);
        self.expect(Token::RightParen)?;
        self.expect(Token::Semicolon)?;

        Ok(ASTNode::DoWhile { body, condition })
    }

    fn parse_local_variable(&mut self) -> Result<ASTNode> {
        let type_name = self.parse_type_name()?;
        self.skip_newlines();

        if let Token::Identifier(name) = &self.current_token {
            let name = name.clone();
            self.advance();
            self.skip_newlines();

            let value = if matches!(self.current_token, Token::Assign) {
                self.advance();
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            self.expect(Token::Semicolon)?;

            Ok(ASTNode::Variable {
                var_type: type_name,
                name,
                value,
            })
        } else {
            Err(anyhow!("Expected variable name after type"))
        }
    }

    fn parse_type_name(&mut self) -> Result<String> {
        let name = match &self.current_token {
            Token::IntegerType => "integer",
            Token::FloatType => "float",
            Token::StringType => "string",
            Token::KeyType => "key",
            Token::VectorType => "vector",
            Token::RotationType => "rotation",
            Token::ListType => "list",
            _ => return Err(anyhow!("Expected type name")),
        };
        let s = name.to_string();
        self.advance();
        Ok(s)
    }

    fn is_type_token(token: &Token) -> bool {
        matches!(
            token,
            Token::IntegerType
                | Token::FloatType
                | Token::StringType
                | Token::KeyType
                | Token::VectorType
                | Token::RotationType
                | Token::ListType
        )
    }

    /// Parse an if statement
    fn parse_if(&mut self) -> Result<ASTNode> {
        self.expect(Token::If)?;
        self.expect(Token::LeftParen)?;
        let condition = Box::new(self.parse_expression()?);
        self.expect(Token::RightParen)?;

        let then_body = if matches!(self.current_token, Token::LeftBrace) {
            self.advance();
            self.parse_block()?
        } else {
            vec![self.parse_statement()?]
        };

        self.skip_newlines();
        let else_body = if matches!(self.current_token, Token::Else) {
            self.advance();
            self.skip_newlines();
            if matches!(self.current_token, Token::LeftBrace) {
                self.advance();
                Some(self.parse_block()?)
            } else {
                Some(vec![self.parse_statement()?])
            }
        } else {
            None
        };

        Ok(ASTNode::If {
            condition,
            then_body,
            else_body,
        })
    }

    /// Parse a while statement
    fn parse_while(&mut self) -> Result<ASTNode> {
        self.expect(Token::While)?;
        self.expect(Token::LeftParen)?;
        let condition = Box::new(self.parse_expression()?);
        self.expect(Token::RightParen)?;

        let body = if matches!(self.current_token, Token::LeftBrace) {
            self.advance();
            self.parse_block()?
        } else {
            vec![self.parse_statement()?]
        };

        Ok(ASTNode::While { condition, body })
    }

    /// Parse a for statement
    fn parse_for(&mut self) -> Result<ASTNode> {
        self.expect(Token::For)?;
        self.expect(Token::LeftParen)?;

        let init = if matches!(self.current_token, Token::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_comma_expression()?))
        };
        self.expect(Token::Semicolon)?;

        let condition = if matches!(self.current_token, Token::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        self.expect(Token::Semicolon)?;

        let increment = if matches!(self.current_token, Token::RightParen) {
            None
        } else {
            Some(Box::new(self.parse_comma_expression()?))
        };
        self.expect(Token::RightParen)?;

        let body = if matches!(self.current_token, Token::LeftBrace) {
            self.advance();
            self.parse_block()?
        } else {
            vec![self.parse_statement()?]
        };

        Ok(ASTNode::For {
            init,
            condition,
            increment,
            body,
        })
    }

    /// Parse a return statement
    fn parse_return(&mut self) -> Result<ASTNode> {
        self.expect(Token::Return)?;

        let value = if matches!(self.current_token, Token::Semicolon) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };

        self.expect(Token::Semicolon)?;
        Ok(ASTNode::Return(value))
    }

    fn parse_expression(&mut self) -> Result<ASTNode> {
        self.skip_newlines();
        self.parse_assignment()
    }

    fn parse_comma_expression(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_expression()?;
        while matches!(self.current_token, Token::Comma) {
            self.advance();
            self.skip_newlines();
            let right = self.parse_expression()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator: Token::Comma,
                right: Box::new(right),
            };
        }
        Ok(node)
    }

    fn parse_assignment(&mut self) -> Result<ASTNode> {
        let node = self.parse_logical_or()?;
        self.skip_newlines();

        match &self.current_token {
            Token::Assign => {
                self.advance();
                self.skip_newlines();
                let value = self.parse_assignment()?;
                Ok(ASTNode::Assignment {
                    target: Box::new(node),
                    value: Box::new(value),
                })
            }
            Token::PlusAssign
            | Token::MinusAssign
            | Token::MultiplyAssign
            | Token::DivideAssign
            | Token::ModuloAssign => {
                let operator = self.current_token.clone();
                self.advance();
                self.skip_newlines();
                let value = self.parse_assignment()?;
                Ok(ASTNode::CompoundAssignment {
                    target: Box::new(node),
                    operator,
                    value: Box::new(value),
                })
            }
            _ => Ok(node),
        }
    }

    fn parse_logical_or(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_logical_and()?;

        loop {
            self.skip_newlines();
            if !matches!(self.current_token, Token::Or) {
                break;
            }
            let operator = self.current_token.clone();
            self.advance();
            self.skip_newlines();
            let right = self.parse_logical_and()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn parse_logical_and(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_bitwise_or()?;

        loop {
            self.skip_newlines();
            if !matches!(self.current_token, Token::And) {
                break;
            }
            let operator = self.current_token.clone();
            self.advance();
            self.skip_newlines();
            let right = self.parse_bitwise_or()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn parse_bitwise_or(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_bitwise_xor()?;

        loop {
            self.skip_newlines();
            if !matches!(self.current_token, Token::BitOr) {
                break;
            }
            let operator = self.current_token.clone();
            self.advance();
            self.skip_newlines();
            let right = self.parse_bitwise_xor()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn parse_bitwise_xor(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_bitwise_and()?;

        loop {
            self.skip_newlines();
            if !matches!(self.current_token, Token::BitXor) {
                break;
            }
            let operator = self.current_token.clone();
            self.advance();
            self.skip_newlines();
            let right = self.parse_bitwise_and()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn parse_bitwise_and(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_equality()?;

        loop {
            self.skip_newlines();
            if !matches!(self.current_token, Token::BitAnd) {
                break;
            }
            let operator = self.current_token.clone();
            self.advance();
            self.skip_newlines();
            let right = self.parse_equality()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn parse_equality(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_comparison()?;

        loop {
            self.skip_newlines();
            if !matches!(self.current_token, Token::Equal | Token::NotEqual) {
                break;
            }
            let operator = self.current_token.clone();
            self.advance();
            self.skip_newlines();
            let right = self.parse_comparison()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn parse_comparison(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_shift()?;

        loop {
            self.skip_newlines();
            match &self.current_token {
                Token::Greater | Token::GreaterEqual if self.in_vector_depth > 0 => break,
                Token::Less | Token::Greater | Token::LessEqual | Token::GreaterEqual => {}
                _ => break,
            }
            let operator = self.current_token.clone();
            self.advance();
            self.skip_newlines();
            let right = self.parse_shift()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn parse_shift(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_term()?;

        loop {
            self.skip_newlines();
            if !matches!(self.current_token, Token::ShiftLeft | Token::ShiftRight) {
                break;
            }
            let operator = self.current_token.clone();
            self.advance();
            self.skip_newlines();
            let right = self.parse_term()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn parse_term(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_factor()?;

        loop {
            self.skip_newlines();
            if !matches!(self.current_token, Token::Plus | Token::Minus) {
                break;
            }
            let operator = self.current_token.clone();
            self.advance();
            self.skip_newlines();
            let right = self.parse_factor()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    fn parse_factor(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_unary()?;

        loop {
            self.skip_newlines();
            if !matches!(
                self.current_token,
                Token::Multiply | Token::Divide | Token::Modulo
            ) {
                break;
            }
            let operator = self.current_token.clone();
            self.advance();
            self.skip_newlines();
            let right = self.parse_unary()?;
            node = ASTNode::BinaryOp {
                left: Box::new(node),
                operator,
                right: Box::new(right),
            };
        }

        Ok(node)
    }

    /// Parse unary expression
    fn parse_unary(&mut self) -> Result<ASTNode> {
        match &self.current_token {
            Token::Not | Token::Minus | Token::BitNot => {
                let operator = self.current_token.clone();
                self.advance();
                self.skip_newlines();
                let operand = self.parse_unary()?;
                Ok(ASTNode::UnaryOp {
                    operator,
                    operand: Box::new(operand),
                })
            }
            Token::Increment => {
                self.advance();
                self.skip_newlines();
                let operand = self.parse_unary()?;
                Ok(ASTNode::PreIncrement(Box::new(operand)))
            }
            Token::Decrement => {
                self.advance();
                self.skip_newlines();
                let operand = self.parse_unary()?;
                Ok(ASTNode::PreDecrement(Box::new(operand)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<ASTNode> {
        let mut node = self.parse_primary()?;

        loop {
            match &self.current_token {
                Token::Increment => {
                    self.advance();
                    node = ASTNode::PostIncrement(Box::new(node));
                }
                Token::Decrement => {
                    self.advance();
                    node = ASTNode::PostDecrement(Box::new(node));
                }
                Token::Dot => {
                    self.advance();
                    if let Token::Identifier(member) = &self.current_token {
                        let member = member.clone();
                        self.advance();
                        node = ASTNode::MemberAccess {
                            object: Box::new(node),
                            member,
                        };
                    } else {
                        return Err(anyhow!("Expected member name after '.'"));
                    }
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_primary(&mut self) -> Result<ASTNode> {
        match &self.current_token {
            Token::Integer(value) => {
                let value = *value;
                self.advance();
                Ok(ASTNode::Literal(LSLValue::Integer(value)))
            }
            Token::Float(value) => {
                let value = *value;
                self.advance();
                Ok(ASTNode::Literal(LSLValue::Float(value)))
            }
            Token::String(value) => {
                let value = value.clone();
                self.advance();
                Ok(ASTNode::Literal(LSLValue::String(value)))
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();

                if matches!(self.current_token, Token::LeftParen) {
                    self.advance();
                    self.skip_newlines();
                    let mut arguments = Vec::new();

                    while !matches!(self.current_token, Token::RightParen) {
                        arguments.push(self.parse_expression()?);
                        self.skip_newlines();

                        if matches!(self.current_token, Token::Comma) {
                            self.advance();
                            self.skip_newlines();
                        } else if !matches!(self.current_token, Token::RightParen) {
                            return Err(anyhow!("Expected ',' or ')' in function call"));
                        }
                    }

                    self.expect(Token::RightParen)?;

                    Ok(ASTNode::FunctionCall { name, arguments })
                } else {
                    Ok(ASTNode::Identifier(name))
                }
            }
            Token::LeftParen => {
                self.advance();
                self.skip_newlines();
                if Self::is_type_token(&self.current_token) {
                    let target_type = self.parse_type_name()?;
                    self.expect(Token::RightParen)?;
                    let expr = self.parse_unary()?;
                    Ok(ASTNode::TypeCast {
                        target_type,
                        expr: Box::new(expr),
                    })
                } else {
                    let expr = self.parse_expression()?;
                    self.expect(Token::RightParen)?;
                    Ok(expr)
                }
            }
            Token::Less => self.parse_vector_or_rotation_literal(),
            Token::LeftBracket => {
                self.advance();
                self.skip_newlines();
                let mut elements = Vec::new();

                while !matches!(self.current_token, Token::RightBracket) {
                    elements.push(self.parse_expression()?);
                    self.skip_newlines();
                    if matches!(self.current_token, Token::Comma) {
                        self.advance();
                        self.skip_newlines();
                    } else if !matches!(self.current_token, Token::RightBracket) {
                        return Err(anyhow!("Expected ',' or ']' in list literal"));
                    }
                }

                self.expect(Token::RightBracket)?;
                Ok(ASTNode::ListLiteral(elements))
            }
            _ => Err(anyhow!(
                "Unexpected token in expression: {:?} at token #{}",
                self.current_token,
                self.position
            )),
        }
    }

    fn parse_vector_or_rotation_literal(&mut self) -> Result<ASTNode> {
        self.expect(Token::Less)?;
        self.skip_newlines();
        self.in_vector_depth += 1;
        let x = self.parse_expression()?;
        self.expect(Token::Comma)?;
        self.skip_newlines();
        let y = self.parse_expression()?;
        self.expect(Token::Comma)?;
        self.skip_newlines();
        let z = self.parse_expression()?;

        if matches!(self.current_token, Token::Comma) {
            self.advance();
            self.skip_newlines();
            let s = self.parse_expression()?;
            self.in_vector_depth -= 1;
            self.expect(Token::Greater)?;
            Ok(ASTNode::RotationLiteral {
                x: Box::new(x),
                y: Box::new(y),
                z: Box::new(z),
                s: Box::new(s),
            })
        } else {
            self.in_vector_depth -= 1;
            self.expect(Token::Greater)?;
            Ok(ASTNode::VectorLiteral {
                x: Box::new(x),
                y: Box::new(y),
                z: Box::new(z),
            })
        }
    }
}

/// LSL Interpreter for executing parsed AST
pub struct LSLInterpreter {
    variables: HashMap<String, LSLValue>,
    constants: HashMap<String, LSLValue>,
}

impl LSLInterpreter {
    pub fn new() -> Self {
        let constant_map = super::lsl_constants::build_constant_map();
        let constants: HashMap<String, LSLValue> = constant_map
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        Self {
            variables: HashMap::new(),
            constants,
        }
    }

    /// Parse LSL source code
    pub fn parse(&mut self, source: &str) -> Result<ASTNode> {
        let trimmed = source.trim();
        let effective_source = if trimmed.starts_with('{') {
            format!("default {}", trimmed)
        } else {
            source.to_string()
        };
        let mut lexer = LSLLexer::new(effective_source);
        let tokens = lexer.tokenize()?;

        debug!("Tokenized {} tokens", tokens.len());

        let mut parser = LSLParser::new(tokens);
        let ast = parser.parse()?;

        debug!("Parsed AST successfully");
        Ok(ast)
    }

    /// Evaluate an AST node
    pub fn evaluate(&mut self, node: &ASTNode) -> Result<LSLValue> {
        match node {
            ASTNode::Literal(value) => Ok(value.clone()),
            ASTNode::Identifier(name) => self
                .variables
                .get(name)
                .or_else(|| self.constants.get(name))
                .cloned()
                .ok_or_else(|| anyhow!("Undefined variable: {}", name)),
            ASTNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                self.evaluate_binary_op(&left_val, operator, &right_val)
            }
            ASTNode::UnaryOp { operator, operand } => {
                let operand_val = self.evaluate(operand)?;
                self.evaluate_unary_op(operator, &operand_val)
            }
            ASTNode::Assignment { target, value } => {
                let val = self.evaluate(value)?;
                if let ASTNode::Identifier(name) = target.as_ref() {
                    self.variables.insert(name.clone(), val.clone());
                    Ok(val)
                } else {
                    Err(anyhow!("Invalid assignment target"))
                }
            }
            _ => {
                warn!("Evaluation not implemented for node type: {:?}", node);
                Ok(LSLValue::Integer(0))
            }
        }
    }

    /// Evaluate binary operations
    fn evaluate_binary_op(
        &self,
        left: &LSLValue,
        operator: &Token,
        right: &LSLValue,
    ) -> Result<LSLValue> {
        match operator {
            Token::Plus => Ok(self.add_values(left, right)?),
            Token::Minus => Ok(self.subtract_values(left, right)?),
            Token::Multiply => Ok(self.multiply_values(left, right)?),
            Token::Divide => Ok(self.divide_values(left, right)?),
            Token::Equal => Ok(LSLValue::Integer(if self.values_equal(left, right) {
                1
            } else {
                0
            })),
            Token::NotEqual => Ok(LSLValue::Integer(if !self.values_equal(left, right) {
                1
            } else {
                0
            })),
            Token::Less => Ok(LSLValue::Integer(
                if self.compare_values(left, right)? < 0 {
                    1
                } else {
                    0
                },
            )),
            Token::Greater => Ok(LSLValue::Integer(
                if self.compare_values(left, right)? > 0 {
                    1
                } else {
                    0
                },
            )),
            Token::LessEqual => Ok(LSLValue::Integer(
                if self.compare_values(left, right)? <= 0 {
                    1
                } else {
                    0
                },
            )),
            Token::GreaterEqual => Ok(LSLValue::Integer(
                if self.compare_values(left, right)? >= 0 {
                    1
                } else {
                    0
                },
            )),
            Token::And => Ok(LSLValue::Integer(if left.is_true() && right.is_true() {
                1
            } else {
                0
            })),
            Token::Or => Ok(LSLValue::Integer(if left.is_true() || right.is_true() {
                1
            } else {
                0
            })),
            Token::Modulo => Ok(self.modulo_values(left, right)?),
            Token::BitAnd => Ok(LSLValue::Integer(left.to_integer() & right.to_integer())),
            Token::BitOr => Ok(LSLValue::Integer(left.to_integer() | right.to_integer())),
            Token::BitXor => Ok(LSLValue::Integer(left.to_integer() ^ right.to_integer())),
            Token::ShiftLeft => Ok(LSLValue::Integer(
                left.to_integer() << (right.to_integer() & 31),
            )),
            Token::ShiftRight => Ok(LSLValue::Integer(
                left.to_integer() >> (right.to_integer() & 31),
            )),
            Token::Comma => Ok(right.clone()),
            _ => Err(anyhow!("Unsupported binary operator: {:?}", operator)),
        }
    }

    /// Evaluate unary operations
    fn evaluate_unary_op(&self, operator: &Token, operand: &LSLValue) -> Result<LSLValue> {
        match operator {
            Token::Minus => match operand {
                LSLValue::Integer(i) => Ok(LSLValue::Integer(-i)),
                LSLValue::Float(f) => Ok(LSLValue::Float(-f)),
                _ => Err(anyhow!("Cannot negate non-numeric value")),
            },
            Token::Not => Ok(LSLValue::Integer(if operand.is_true() { 0 } else { 1 })),
            Token::BitNot => Ok(LSLValue::Integer(!operand.to_integer())),
            _ => Err(anyhow!("Unsupported unary operator: {:?}", operator)),
        }
    }

    /// Add two LSL values
    fn add_values(&self, left: &LSLValue, right: &LSLValue) -> Result<LSLValue> {
        match (left, right) {
            (LSLValue::Integer(a), LSLValue::Integer(b)) => Ok(LSLValue::Integer(a + b)),
            (LSLValue::Float(a), LSLValue::Float(b)) => Ok(LSLValue::Float(a + b)),
            (LSLValue::Integer(a), LSLValue::Float(b)) => Ok(LSLValue::Float(*a as f32 + b)),
            (LSLValue::Float(a), LSLValue::Integer(b)) => Ok(LSLValue::Float(a + *b as f32)),
            (LSLValue::String(a), LSLValue::String(b)) => {
                Ok(LSLValue::String(format!("{}{}", a, b)))
            }
            (LSLValue::Vector(a), LSLValue::Vector(b)) => Ok(LSLValue::Vector(*a + *b)),
            (LSLValue::List(a), LSLValue::List(b)) => {
                let mut result = a.clone();
                result.extend(b.iter().cloned());
                Ok(LSLValue::List(result))
            }
            (LSLValue::List(a), other) => {
                let mut result = a.clone();
                result.push(other.clone());
                Ok(LSLValue::List(result))
            }
            (other, LSLValue::List(b)) => {
                let mut result = vec![other.clone()];
                result.extend(b.iter().cloned());
                Ok(LSLValue::List(result))
            }
            _ => Err(anyhow!("Cannot add these types")),
        }
    }

    /// Subtract two LSL values
    fn subtract_values(&self, left: &LSLValue, right: &LSLValue) -> Result<LSLValue> {
        match (left, right) {
            (LSLValue::Integer(a), LSLValue::Integer(b)) => Ok(LSLValue::Integer(a - b)),
            (LSLValue::Float(a), LSLValue::Float(b)) => Ok(LSLValue::Float(a - b)),
            (LSLValue::Integer(a), LSLValue::Float(b)) => Ok(LSLValue::Float(*a as f32 - b)),
            (LSLValue::Float(a), LSLValue::Integer(b)) => Ok(LSLValue::Float(a - *b as f32)),
            (LSLValue::Vector(a), LSLValue::Vector(b)) => Ok(LSLValue::Vector(*a - *b)),
            _ => Err(anyhow!("Cannot subtract these types")),
        }
    }

    /// Multiply two LSL values
    fn multiply_values(&self, left: &LSLValue, right: &LSLValue) -> Result<LSLValue> {
        match (left, right) {
            (LSLValue::Integer(a), LSLValue::Integer(b)) => Ok(LSLValue::Integer(a * b)),
            (LSLValue::Float(a), LSLValue::Float(b)) => Ok(LSLValue::Float(a * b)),
            (LSLValue::Integer(a), LSLValue::Float(b)) => Ok(LSLValue::Float(*a as f32 * b)),
            (LSLValue::Float(a), LSLValue::Integer(b)) => Ok(LSLValue::Float(a * *b as f32)),
            (LSLValue::Vector(a), LSLValue::Float(b)) => Ok(LSLValue::Vector(*a * *b)),
            (LSLValue::Float(a), LSLValue::Vector(b)) => Ok(LSLValue::Vector(*b * *a)),
            (LSLValue::Rotation(a), LSLValue::Rotation(b)) => Ok(LSLValue::Rotation(*a * *b)),
            _ => Err(anyhow!("Cannot multiply these types")),
        }
    }

    /// Divide two LSL values
    fn divide_values(&self, left: &LSLValue, right: &LSLValue) -> Result<LSLValue> {
        match (left, right) {
            (LSLValue::Integer(a), LSLValue::Integer(b)) => {
                if *b == 0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(LSLValue::Integer(a / b))
                }
            }
            (LSLValue::Float(a), LSLValue::Float(b)) => {
                if *b == 0.0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(LSLValue::Float(a / b))
                }
            }
            (LSLValue::Integer(a), LSLValue::Float(b)) => {
                if *b == 0.0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(LSLValue::Float(*a as f32 / b))
                }
            }
            (LSLValue::Float(a), LSLValue::Integer(b)) => {
                if *b == 0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(LSLValue::Float(a / *b as f32))
                }
            }
            (LSLValue::Vector(a), LSLValue::Float(b)) => {
                if *b == 0.0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(LSLValue::Vector(*a / *b))
                }
            }
            _ => Err(anyhow!("Cannot divide these types")),
        }
    }

    fn modulo_values(&self, left: &LSLValue, right: &LSLValue) -> Result<LSLValue> {
        match (left, right) {
            (LSLValue::Integer(a), LSLValue::Integer(b)) => {
                if *b == 0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(LSLValue::Integer(a % b))
                }
            }
            (LSLValue::Float(a), LSLValue::Float(b)) => {
                if *b == 0.0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(LSLValue::Float(a % b))
                }
            }
            (LSLValue::Integer(a), LSLValue::Float(b)) => {
                if *b == 0.0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(LSLValue::Float(*a as f32 % b))
                }
            }
            (LSLValue::Float(a), LSLValue::Integer(b)) => {
                if *b == 0 {
                    Err(anyhow!("Division by zero"))
                } else {
                    Ok(LSLValue::Float(a % *b as f32))
                }
            }
            _ => Err(anyhow!("Cannot modulo these types")),
        }
    }

    /// Check if two values are equal
    fn values_equal(&self, left: &LSLValue, right: &LSLValue) -> bool {
        match (left, right) {
            (LSLValue::Integer(a), LSLValue::Integer(b)) => a == b,
            (LSLValue::Float(a), LSLValue::Float(b)) => (a - b).abs() < f32::EPSILON,
            (LSLValue::String(a), LSLValue::String(b)) => a == b,
            (LSLValue::Key(a), LSLValue::Key(b)) => a == b,
            (LSLValue::Vector(a), LSLValue::Vector(b)) => a == b,
            (LSLValue::Rotation(a), LSLValue::Rotation(b)) => a == b,
            _ => false,
        }
    }

    /// Compare two values (-1, 0, 1)
    fn compare_values(&self, left: &LSLValue, right: &LSLValue) -> Result<i32> {
        match (left, right) {
            (LSLValue::Integer(a), LSLValue::Integer(b)) => Ok(a.cmp(b) as i32),
            (LSLValue::Float(a), LSLValue::Float(b)) => {
                Ok(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) as i32)
            }
            (LSLValue::String(a), LSLValue::String(b)) => Ok(a.cmp(b) as i32),
            _ => Err(anyhow!("Cannot compare these types")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic_tokens() -> Result<()> {
        let mut lexer = LSLLexer::new("42 3.14 \"hello\" identifier".to_string());
        let tokens = lexer.tokenize()?;

        assert!(matches!(tokens[0], Token::Integer(42)));
        assert!(matches!(tokens[1], Token::Float(f) if (f - 3.14).abs() < f32::EPSILON));
        assert!(matches!(tokens[2], Token::String(ref s) if s == "hello"));
        assert!(matches!(tokens[3], Token::Identifier(ref s) if s == "identifier"));

        Ok(())
    }

    #[test]
    fn test_lexer_operators() -> Result<()> {
        let mut lexer = LSLLexer::new("+ - * / == != < > <= >=".to_string());
        let tokens = lexer.tokenize()?;

        let expected = vec![
            Token::Plus,
            Token::Minus,
            Token::Multiply,
            Token::Divide,
            Token::Equal,
            Token::NotEqual,
            Token::Less,
            Token::Greater,
            Token::LessEqual,
            Token::GreaterEqual,
        ];

        for (i, expected_token) in expected.iter().enumerate() {
            assert_eq!(
                std::mem::discriminant(&tokens[i]),
                std::mem::discriminant(expected_token)
            );
        }

        Ok(())
    }

    #[test]
    fn test_lexer_bitwise_operators() -> Result<()> {
        let mut lexer = LSLLexer::new("& | ^ ~ << >>".to_string());
        let tokens = lexer.tokenize()?;

        assert!(matches!(tokens[0], Token::BitAnd));
        assert!(matches!(tokens[1], Token::BitOr));
        assert!(matches!(tokens[2], Token::BitXor));
        assert!(matches!(tokens[3], Token::BitNot));
        assert!(matches!(tokens[4], Token::ShiftLeft));
        assert!(matches!(tokens[5], Token::ShiftRight));

        Ok(())
    }

    #[test]
    fn test_lexer_compound_assignment() -> Result<()> {
        let mut lexer = LSLLexer::new("+= -= *= /= %=".to_string());
        let tokens = lexer.tokenize()?;

        assert!(matches!(tokens[0], Token::PlusAssign));
        assert!(matches!(tokens[1], Token::MinusAssign));
        assert!(matches!(tokens[2], Token::MultiplyAssign));
        assert!(matches!(tokens[3], Token::DivideAssign));
        assert!(matches!(tokens[4], Token::ModuloAssign));

        Ok(())
    }

    #[test]
    fn test_lexer_increment_decrement() -> Result<()> {
        let mut lexer = LSLLexer::new("++ -- @ .".to_string());
        let tokens = lexer.tokenize()?;

        assert!(matches!(tokens[0], Token::Increment));
        assert!(matches!(tokens[1], Token::Decrement));
        assert!(matches!(tokens[2], Token::At));
        assert!(matches!(tokens[3], Token::Dot));

        Ok(())
    }

    #[test]
    fn test_lexer_hex_literals() -> Result<()> {
        let mut lexer = LSLLexer::new("0xFF 0x1A 0x0".to_string());
        let tokens = lexer.tokenize()?;

        assert!(matches!(tokens[0], Token::Integer(255)));
        assert!(matches!(tokens[1], Token::Integer(26)));
        assert!(matches!(tokens[2], Token::Integer(0)));

        Ok(())
    }

    #[test]
    fn test_lexer_float_suffix() -> Result<()> {
        let mut lexer = LSLLexer::new("1.0f 2.5F".to_string());
        let tokens = lexer.tokenize()?;

        assert!(matches!(tokens[0], Token::Float(f) if (f - 1.0).abs() < f32::EPSILON));
        assert!(matches!(tokens[1], Token::Float(f) if (f - 2.5).abs() < f32::EPSILON));

        Ok(())
    }

    #[test]
    fn test_lexer_scientific_notation() -> Result<()> {
        let mut lexer = LSLLexer::new("1e3 2.5e-2".to_string());
        let tokens = lexer.tokenize()?;

        assert!(matches!(tokens[0], Token::Float(f) if (f - 1000.0).abs() < 1.0));
        assert!(matches!(tokens[1], Token::Float(f) if (f - 0.025).abs() < 0.001));

        Ok(())
    }

    #[test]
    fn test_lexer_logical_vs_bitwise() -> Result<()> {
        let mut lexer = LSLLexer::new("&& || & |".to_string());
        let tokens = lexer.tokenize()?;

        assert!(matches!(tokens[0], Token::And));
        assert!(matches!(tokens[1], Token::Or));
        assert!(matches!(tokens[2], Token::BitAnd));
        assert!(matches!(tokens[3], Token::BitOr));

        Ok(())
    }

    #[test]
    fn test_lexer_source_location() -> Result<()> {
        let mut lexer = LSLLexer::new("x\ny".to_string());

        let loc1 = lexer.location();
        assert_eq!(loc1.line, 1);
        assert_eq!(loc1.col, 1);

        lexer.next_token()?; // 'x'
        lexer.next_token()?; // '\n'
        let loc2 = lexer.location();
        assert_eq!(loc2.line, 2);
        assert_eq!(loc2.col, 1);

        Ok(())
    }

    #[test]
    fn test_parser_simple_expression() -> Result<()> {
        let mut lexer = LSLLexer::new("1 + 2".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = LSLParser::new(tokens);

        let ast = parser.parse_expression()?;

        if let ASTNode::BinaryOp {
            left,
            operator,
            right,
        } = ast
        {
            assert!(matches!(
                left.as_ref(),
                ASTNode::Literal(LSLValue::Integer(1))
            ));
            assert!(matches!(operator, Token::Plus));
            assert!(matches!(
                right.as_ref(),
                ASTNode::Literal(LSLValue::Integer(2))
            ));
        } else {
            panic!("Expected binary operation, got: {:?}", &ast);
        }

        Ok(())
    }

    #[test]
    fn test_parser_vector_literal() -> Result<()> {
        let mut lexer = LSLLexer::new("<1.0, 2.0, 3.0>".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = LSLParser::new(tokens);

        let ast = parser.parse_expression()?;
        assert!(matches!(ast, ASTNode::VectorLiteral { .. }));

        Ok(())
    }

    #[test]
    fn test_parser_rotation_literal() -> Result<()> {
        let mut lexer = LSLLexer::new("<0.0, 0.0, 0.0, 1.0>".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = LSLParser::new(tokens);

        let ast = parser.parse_expression()?;
        assert!(matches!(ast, ASTNode::RotationLiteral { .. }));

        Ok(())
    }

    #[test]
    fn test_parser_list_literal() -> Result<()> {
        let mut lexer = LSLLexer::new("[1, 2, 3]".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = LSLParser::new(tokens);

        let ast = parser.parse_expression()?;
        if let ASTNode::ListLiteral(elements) = ast {
            assert_eq!(elements.len(), 3);
        } else {
            panic!("Expected list literal");
        }

        Ok(())
    }

    #[test]
    fn test_parser_type_cast() -> Result<()> {
        let mut lexer = LSLLexer::new("(string)42".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = LSLParser::new(tokens);

        let ast = parser.parse_expression()?;
        if let ASTNode::TypeCast { target_type, .. } = ast {
            assert_eq!(target_type, "string");
        } else {
            panic!("Expected type cast, got: {:?}", ast);
        }

        Ok(())
    }

    #[test]
    fn test_parser_compound_assignment() -> Result<()> {
        let mut lexer = LSLLexer::new("x += 5".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = LSLParser::new(tokens);

        let ast = parser.parse_expression()?;
        assert!(matches!(ast, ASTNode::CompoundAssignment { .. }));

        Ok(())
    }

    #[test]
    fn test_parser_member_access() -> Result<()> {
        let mut lexer = LSLLexer::new("v.x".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = LSLParser::new(tokens);

        let ast = parser.parse_expression()?;
        if let ASTNode::MemberAccess { member, .. } = ast {
            assert_eq!(member, "x");
        } else {
            panic!("Expected member access, got: {:?}", ast);
        }

        Ok(())
    }

    #[test]
    fn test_parser_pre_increment() -> Result<()> {
        let mut lexer = LSLLexer::new("++x".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = LSLParser::new(tokens);

        let ast = parser.parse_expression()?;
        assert!(matches!(ast, ASTNode::PreIncrement(_)));

        Ok(())
    }

    #[test]
    fn test_parser_post_increment() -> Result<()> {
        let mut lexer = LSLLexer::new("x++".to_string());
        let tokens = lexer.tokenize()?;
        let mut parser = LSLParser::new(tokens);

        let ast = parser.parse_expression()?;
        assert!(matches!(ast, ASTNode::PostIncrement(_)));

        Ok(())
    }

    #[test]
    fn test_interpreter_arithmetic() -> Result<()> {
        let mut interpreter = LSLInterpreter::new();

        let ast = interpreter.parse("1 + 2")?;
        if let ASTNode::Program(statements) = ast {
            if let Some(expr) = statements.first() {
                let result = interpreter.evaluate(expr)?;
                assert_eq!(result, LSLValue::Integer(3));
            }
        }

        Ok(())
    }
}
