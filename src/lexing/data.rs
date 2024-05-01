#[derive(PartialEq, Debug, Clone)]
pub enum TokenTypes {
    EXIT,
    TEXT {
        text: String,
    },
    NUMBER {
        val: i32,
    },
    IDENT {
        name: String,
        isptr: bool,
        isref: bool,
    },
    STAR,
    DOT,
    SEMI,
    LET,
    QUOTE,
    PLUS,
    MINUS,
    EQ,
    BOOLEQ,
    GREATER,
    LESSER,
    IF,
    ELSE,
    MOD,
    OR,
    AND,
    LBRACKET,
    RBRACKET,
    LCURLY,
    RCURLY,
    COLON,
    ARROW,
    NULL,
    NUMTYPE(bool),
    BOOLTYPE(bool),
    TEXTTYPE(bool),
    FUNCTYPE,
    NOTHINGTYPE,
    BOOL {
        val: bool,
    },
    EOF,
    COMMA,
    RETURN,
    AMPER,
    INLINE,
    FOR,
    IN,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub variant: TokenTypes,
    pub line_num: usize,
}

impl Token {
    pub fn variant_name(&self) -> &str {
        use TokenTypes;
        match self.variant {
            TokenTypes::IF => "IF",
            TokenTypes::AND => "AND",
            TokenTypes::OR => "OR",
            TokenTypes::MOD => "MOD",
            TokenTypes::EXIT => "EXIT",
            TokenTypes::TEXT { text: _ } => "TEXT",
            TokenTypes::NUMBER { val: _ } => "NUMBER",
            TokenTypes::SEMI => "SEMI",
            TokenTypes::QUOTE => "QUOTE",
            TokenTypes::PLUS => "PLUS",
            TokenTypes::MINUS => "MINUS",
            TokenTypes::EQ => "EQ",
            TokenTypes::BOOLEQ => "BOOLEQ",
            TokenTypes::GREATER => "GREATER",
            TokenTypes::LESSER => "LESSER",
            TokenTypes::LBRACKET => "LBRACKET",
            TokenTypes::RBRACKET => "RBRACKET",
            TokenTypes::LCURLY => "LCURLY",
            TokenTypes::RCURLY => "RCURLY",
            TokenTypes::NULL => "NULL",
            TokenTypes::LET => "LET",
            TokenTypes::IDENT {
                name: _,
                isptr: _,
                isref: _,
            } => "IDENT",
            TokenTypes::COLON => "COLON",
            TokenTypes::ARROW => "ARROW",
            TokenTypes::NUMTYPE(_) => "TEXTTYPE",
            TokenTypes::BOOLTYPE(_) => "BOOLTYPE",
            TokenTypes::TEXTTYPE(_) => "TEXTTYPE",
            TokenTypes::FUNCTYPE => "FUNCTYPE",
            TokenTypes::BOOL { val: _ } => "BOOL",
            TokenTypes::EOF => "EOF",
            TokenTypes::NOTHINGTYPE => "NOTHINGTYPE",
            TokenTypes::COMMA => "COMMA",
            TokenTypes::RETURN => "RETURN",
            TokenTypes::AMPER => "AMPER",
            TokenTypes::ELSE => "ELSE",
            TokenTypes::INLINE => "INLINE",
            TokenTypes::FOR => "FOR",
            TokenTypes::IN => "IN",
            TokenTypes::DOT => "DOT",
            TokenTypes::STAR => "STAR",
        }
    }
}

impl Token {
    pub fn new(tipe: TokenTypes, line: usize) -> Self {
        Self {
            variant: tipe,
            line_num: line,
        }
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
