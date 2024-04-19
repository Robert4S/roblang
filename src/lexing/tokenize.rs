use std::fs;
use std::io;

use crate::data::*;

pub fn parse_start(buildfile: &String) -> Result<Vec<Token>, io::Error> {
    let reader = fs::read_to_string(buildfile)?;
    let filereader = FileReader::new(&reader);
    Ok(parse_tokenize(filereader))
}

pub fn parse_tokenize(reader: FileReader) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = reader.chars().peekable();
    let mut wordbuf = String::new();
    let mut textlookup = false;
    let mut numlookup = false;
    let mut line_number: usize = 1;
    let mut comment = false;

    while let Some(current) = iter.next() {
        if comment {
            if current != '\n' {
                continue;
            }
        }
        match current {
            ',' => {
                if let Some(keyword) = test_keyword(&wordbuf, line_number) {
                    tokens.push(keyword);
                } else {
                    let ident = TokenTypes::IDENT { name: wordbuf };
                    tokens.push(Token::new(ident, line_number));
                }
                wordbuf = String::new();
                tokens.push(Token::new(TokenTypes::COMMA, line_number));
            }
            '"' => {
                'textlookup: while let Some(t) = iter.next() {
                    if t == '\\' {
                        if let Some(escaped) = iter.next() {
                            match escaped {
                                '"' => wordbuf.push('"'),
                                'n' => {
                                    wordbuf.push('\\');
                                    wordbuf.push('n');
                                }
                                _ => {
                                    wordbuf.push('\\');
                                    wordbuf.push(escaped);
                                }
                            }
                        }
                        continue 'textlookup;
                    }
                    if t == '"' {
                        break 'textlookup;
                    }
                    wordbuf.push(t);
                }
                tokens.push(Token::new(TokenTypes::TEXT { text: wordbuf }, line_number));
                wordbuf = String::new();
                // tokens.push(Token::new(TokenTypes::QUOTE));
                textlookup = !textlookup;
                continue;
            }
            ';' => {
                if wordbuf.len() >= 1_usize {
                    if let Some(number) = try_number(&wordbuf) {
                        tokens.push(Token::new(TokenTypes::NUMBER { val: number }, line_number));
                        wordbuf = String::new();
                    } else if let Some(keyword) = test_keyword(&wordbuf, line_number) {
                        tokens.push(keyword);
                        wordbuf = String::new();
                    } else if wordbuf.len() >= 1 {
                        let ident = Token::new(
                            TokenTypes::IDENT {
                                name: wordbuf.clone(),
                            },
                            line_number,
                        );
                        tokens.push(ident);
                        wordbuf = String::new();
                    }
                }
                tokens.push(Token::new(TokenTypes::SEMI, line_number));
                textlookup = false;
                numlookup = false;
                continue;
            }
            '+' => {
                if let Some(number) = try_number(&wordbuf) {
                    tokens.push(Token::new(TokenTypes::NUMBER { val: number }, line_number));
                    wordbuf = String::new();
                }
                tokens.push(Token::new(TokenTypes::PLUS, line_number));
            }
            '-' => {
                if let Some(next) = iter.peek() {
                    match next {
                        '>' => {
                            iter.next();
                            tokens.push(Token::new(TokenTypes::ARROW, line_number));
                        }
                        _ => {
                            tokens.push(Token::new(TokenTypes::MINUS, line_number));
                        }
                    }
                }
                continue;
            }
            '<' => {
                tokens.push(Token::new(TokenTypes::LESSER, line_number));
            }
            '>' => {
                tokens.push(Token::new(TokenTypes::LESSER, line_number));
            }
            '=' => {
                if let Some(eq) = iter.peek() {
                    if *eq == '=' {
                        iter.next();
                        tokens.push(Token::new(TokenTypes::BOOLEQ, line_number));
                        continue;
                    }
                }
                tokens.push(Token::new(TokenTypes::EQ, line_number));
            }
            '(' => {
                if let Some(keyword) = test_keyword(&wordbuf, line_number) {
                    tokens.push(keyword);
                    wordbuf = String::new();
                } else if wordbuf.len() > 0 {
                    let ident = Token::new(TokenTypes::IDENT { name: wordbuf }, line_number);
                    wordbuf = String::new();
                    tokens.push(ident);
                }
                tokens.push(Token::new(TokenTypes::LBRACKET, line_number));
            }
            ')' => {
                if let Some(keyword) = test_keyword(&wordbuf, line_number) {
                    tokens.push(keyword);
                } else if let Some(number) = try_number(&wordbuf) {
                    tokens.push(Token::new(TokenTypes::NUMBER { val: number }, line_number));
                    numlookup = false;
                } else if wordbuf.len() >= 1 {
                    let ident = TokenTypes::IDENT {
                        name: wordbuf.clone(),
                    };
                    wordbuf = String::new();
                    tokens.push(Token::new(ident, line_number));
                }
                wordbuf = String::new();
                tokens.push(Token::new(TokenTypes::RBRACKET, line_number));
                textlookup = false;
                numlookup = false;
                continue;
            }
            '{' => {
                tokens.push(Token::new(TokenTypes::LCURLY, line_number));
            }
            '}' => {
                tokens.push(Token::new(TokenTypes::RCURLY, line_number));
            }
            '|' => {
                tokens.push(Token::new(TokenTypes::OR, line_number));
            }
            '&' => {
                if let Some(and) = iter.next() {
                    if and == '&' {
                        tokens.push(Token::new(TokenTypes::AND, line_number));
                    } else {
                        tokens.push(Token::new(TokenTypes::AMPER, line_number));
                    }
                }
            }
            ':' => {
                if let Some(keyword) = test_keyword(&wordbuf, line_number) {
                    tokens.push(keyword);
                } else {
                    let ident = TokenTypes::IDENT {
                        name: wordbuf.clone(),
                    };
                    wordbuf = String::new();
                    tokens.push(Token::new(ident, line_number));
                }
                tokens.push(Token::new(TokenTypes::COLON, line_number));
            }
            '\\' => {
                if let Some(escaped) = iter.next() {
                    wordbuf.push(escaped);
                }
            }
            '#' => {
                comment = true;
                continue;
            }
            '\n' => {
                comment = false;
                line_number += 1;
                continue;
            }
            '\t' | '\r' => {}
            _ => {
                if textlookup {
                    wordbuf.push(current);
                } else if current == ' ' {
                    if let Some(keyword) = test_keyword(&wordbuf, line_number) {
                        tokens.push(keyword);
                        wordbuf = String::new();
                        continue;
                    } else if wordbuf.len() >= 1 {
                        let ident = Token::new(
                            TokenTypes::IDENT {
                                name: wordbuf.clone(),
                            },
                            line_number,
                        );
                        tokens.push(ident);
                        wordbuf = String::new();
                        continue;
                    }
                } else if current.is_numeric() {
                    numlookup = true;
                    wordbuf.push(current);
                } else if numlookup {
                    wordbuf.push(current);
                } else {
                    wordbuf.push(current);
                }
            }
        }
    }
    tokens
}

pub fn test_keyword(word: &String, line: usize) -> Option<Token> {
    match word.as_str() {
        "showme" => Some(Token {
            variant: TokenTypes::PRINT,
            line_num: line,
        }),
        "exit" => Some(Token {
            variant: TokenTypes::EXIT,
            line_num: line,
        }),
        "let" => Some(Token {
            variant: TokenTypes::LET,
            line_num: line,
        }),
        "Number" => Some(Token {
            variant: TokenTypes::NUMTYPE,
            line_num: line,
        }),
        "Text" => Some(Token {
            variant: TokenTypes::TEXTTYPE,
            line_num: line,
        }),
        "Bool" => Some(Token {
            variant: TokenTypes::BOOLTYPE,
            line_num: line,
        }),
        "Function" => Some(Token {
            variant: TokenTypes::FUNCTYPE,
            line_num: line,
        }),
        "True" => Some(Token {
            variant: TokenTypes::BOOL { val: true },
            line_num: line,
        }),
        "False" => Some(Token {
            variant: TokenTypes::BOOL { val: false },
            line_num: line,
        }),
        "return" => Some(Token {
            variant: TokenTypes::RETURN,
            line_num: line,
        }),
        "if" => Some(Token::new(TokenTypes::IF, line)),
        "inline" => Some(Token::new(TokenTypes::INLINE, line)),
        _ => None,
    }
}

pub fn try_number(word: &String) -> Option<i32> {
    for character in word.chars() {
        if !character.is_numeric() {
            return None;
        }
    }
    word.parse::<i32>().ok()
}
