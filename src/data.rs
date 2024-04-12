#[derive(PartialEq, Debug, Clone)]
pub enum TokenTypes {
    PRINT,
    EXIT,
    TEXT { text: String },
    NUMBER { val: i32 },
    IDENT { name: String },
    SEMI,
    LET,
    QUOTE,
    PLUS,
    MINUS,
    EQ,
    LBRACKET,
    RBRACKET,
    LCURLY,
    RCURLY,
    COLON,
    ARROW,
    NULL,
    NUMTYPE,
    BOOLTYPE,
    TEXTTYPE,
    BOOL { val: bool },
}

impl Token {
    pub fn variant_name(&self) -> &str {
        match self.variant {
            TokenTypes::PRINT => "PRINT",
            TokenTypes::EXIT => "EXIT",
            TokenTypes::TEXT { text: _ } => "TEXT",
            TokenTypes::NUMBER { val: _ } => "NUMBER",
            TokenTypes::SEMI => "SEMI",
            TokenTypes::QUOTE => "QUOTE",
            TokenTypes::PLUS => "PLUS",
            TokenTypes::MINUS => "MINUS",
            TokenTypes::EQ => "EQ",
            TokenTypes::LBRACKET => "LBRACKET",
            TokenTypes::RBRACKET => "RBRACKET",
            TokenTypes::LCURLY => "LCURLY",
            TokenTypes::RCURLY => "RCURLY",
            TokenTypes::NULL => "NULL",
            TokenTypes::LET => "LET",
            TokenTypes::IDENT { name: _ } => "IDENT",
            TokenTypes::COLON => "COLON",
            TokenTypes::ARROW => "ARROW",
            TokenTypes::NUMTYPE => "TEXTTYPE",
            TokenTypes::BOOLTYPE => "BOOLTYPE",
            TokenTypes::TEXTTYPE => "TEXTTYPE",
            TokenTypes::BOOL {val: _} => "BOOL",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub variant: TokenTypes,
}

impl Token {
    pub fn new(tipe: TokenTypes) -> Self {
        Self { variant: tipe }
    }
}

pub struct FileReader {
    content: String,
    index: usize,
}

impl FileReader {
    pub fn new(content: &String) -> Self {
        let content = content.to_string();
        FileReader { content, index: 0 }
    }

    pub fn chars(&self) -> std::str::Chars {
        self.content.chars()
    }

    pub fn peek(&self, distance: usize) -> Option<char> {
        self.content.chars().nth(self.index + distance)
    }
}
