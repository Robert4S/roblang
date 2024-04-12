use crate::data::*;
use std::fs;
use std::io;

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

    while let Some(current) = iter.next() {
        match current {
            ',' => {
                if let Some(keyword) = test_keyword(&wordbuf) {
                    tokens.push(keyword);
                } else {
                    let ident = TokenTypes::IDENT { name: wordbuf };
                    tokens.push(Token::new(ident));
                }
                wordbuf = String::new();
                tokens.push(Token::new(TokenTypes::COMMA));
            }
            '"' => {
                if textlookup {
                    tokens.push(Token::new(TokenTypes::TEXT { text: wordbuf }));
                    wordbuf = String::new();
                }
                // tokens.push(Token::new(TokenTypes::QUOTE));
                textlookup = !textlookup;
                continue;
            }
            ';' => {
                if wordbuf.len() >= 1_usize {
                    if let Some(number) = try_number(&wordbuf) {
                        tokens.push(Token::new(TokenTypes::NUMBER { val: number }));
                        wordbuf = String::new();
                    } else if let Some(keyword) = test_keyword(&wordbuf) {
                        tokens.push(keyword);
                        wordbuf = String::new();
                        continue;
                    } else if wordbuf.len() >= 1 {
                        let ident = Token::new(TokenTypes::IDENT {
                            name: wordbuf.clone(),
                        });
                        tokens.push(ident);
                        wordbuf = String::new();
                        continue;
                    }
                }
                tokens.push(Token::new(TokenTypes::SEMI));
                textlookup = false;
                numlookup = false;
                continue;
            }
            '+' => {
                if let Some(number) = try_number(&wordbuf) {
                    tokens.push(Token::new(TokenTypes::NUMBER { val: number }));
                    wordbuf = String::new();
                }
                tokens.push(Token::new(TokenTypes::PLUS));
            }
            '-' => {
                if let Some(next) = iter.peek() {
                    match next {
                        '>' => {
                            iter.next();
                            tokens.push(Token::new(TokenTypes::ARROW));
                        }
                        _ => {
                            tokens.push(Token::new(TokenTypes::MINUS));
                        }
                    }
                }
                continue;
            }
            '=' => {
                tokens.push(Token::new(TokenTypes::EQ));
            }
            '(' => {
                if let Some(keyword) = test_keyword(&wordbuf) {
                    tokens.push(keyword);
                    wordbuf = String::new();
                    tokens.push(Token::new(TokenTypes::LBRACKET));
                    continue;
                }
                tokens.push(Token::new(TokenTypes::LBRACKET));
            }
            ')' => {
                if let Some(keyword) = test_keyword(&wordbuf) {
                    tokens.push(keyword);
                } else if let Some(number) = try_number(&wordbuf) {
                    tokens.push(Token::new(TokenTypes::NUMBER { val: number }));
                    numlookup = false;
                } else if wordbuf.len() >= 1 {
                    let ident = TokenTypes::IDENT {
                        name: wordbuf.clone(),
                    };
                    wordbuf = String::new();
                    tokens.push(Token::new(ident));
                }
                wordbuf = String::new();
                tokens.push(Token::new(TokenTypes::RBRACKET));
                textlookup = false;
                numlookup = false;
                continue;
            }
            '{' => {
                tokens.push(Token::new(TokenTypes::LCURLY));
            }
            '}' => {
                tokens.push(Token::new(TokenTypes::RCURLY));
            }
            ':' => {
                if let Some(keyword) = test_keyword(&wordbuf) {
                    tokens.push(keyword);
                } else {
                    let ident = TokenTypes::IDENT {
                        name: wordbuf.clone(),
                    };
                    wordbuf = String::new();
                    tokens.push(Token::new(ident));
                }
                tokens.push(Token::new(TokenTypes::COLON));
            }
            '\n' | '\t' => {
                continue;
            }
            _ => {
                if textlookup {
                    wordbuf.push(current);
                } else if current == ' ' {
                    if let Some(keyword) = test_keyword(&wordbuf) {
                        tokens.push(keyword);
                        wordbuf = String::new();
                        continue;
                    } else if wordbuf.len() >= 1 {
                        let ident = Token::new(TokenTypes::IDENT {
                            name: wordbuf.clone(),
                        });
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

pub fn test_keyword(word: &String) -> Option<Token> {
    match word.as_str() {
        "showme" => Some(Token {
            variant: TokenTypes::PRINT,
        }),
        "exit" => Some(Token {
            variant: TokenTypes::EXIT,
        }),
        "let" => Some(Token {
            variant: TokenTypes::LET,
        }),
        "Number" => Some(Token {
            variant: TokenTypes::NUMTYPE,
        }),
        "Text" => Some(Token {
            variant: TokenTypes::TEXTTYPE,
        }),
        "Bool" => Some(Token {
            variant: TokenTypes::BOOLTYPE,
        }),
        "Function" => Some(Token {
            variant: TokenTypes::FUNCTYPE,
        }),
        "True" => Some(Token {
            variant: TokenTypes::BOOL { val: true },
        }),
        "False" => Some(Token {
            variant: TokenTypes::BOOL { val: false },
        }),
        "return" => Some(Token {
            variant: TokenTypes::RETURN,
        }),
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
