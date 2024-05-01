use std::fs;
use std::io;

use crate::data::*;

pub fn parse_start(buildfile: &String) -> Result<Option<Vec<Token>>, io::Error> {
    let reader = fs::read_to_string(buildfile)?;
    let filereader = FileReader::new(&reader);
    Ok(parse_tokenize(filereader))
}

pub fn parse_tokenize(reader: FileReader) -> Option<Vec<Token>> {
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
            '*' => {
                if wordbuf.len() != 0 {
                    if let Some(keyword) = test_keyword(&wordbuf, line_number) {
                        tokens.push(keyword);
                    } else {
                        tokens.push(Token::new(
                            TokenTypes::TEXT {
                                text: wordbuf.clone(),
                            },
                            line_number,
                        ));
                    }
                }
                wordbuf.clear();
                tokens.push(Token::new(TokenTypes::STAR, line_number));
            }
            '.' => {
                if wordbuf.len() == 0 {
                    tokens.push(Token::new(TokenTypes::DOT, line_number));
                    continue;
                }
                if let Some(num) = try_number(&wordbuf) {
                    tokens.push(Token::new(TokenTypes::NUMBER { val: num }, line_number))
                } else if let Some(kw) = test_keyword(&wordbuf, line_number) {
                    tokens.push(kw);
                } else {
                    let ident = TokenTypes::IDENT {
                        name: wordbuf.clone(),
                        isptr: false,
                        isref: false,
                    };
                    tokens.push(Token::new(ident, line_number));
                }
                wordbuf.clear();
                tokens.push(Token::new(TokenTypes::DOT, line_number));
            }
            ',' => {
                if wordbuf.len() == 0 {
                    tokens.push(Token::new(TokenTypes::COMMA, line_number));
                    continue;
                }
                if let Some(keyword) = test_keyword(&wordbuf, line_number) {
                    tokens.push(keyword);
                } else {
                    let ident = TokenTypes::IDENT {
                        name: wordbuf,
                        isptr: false,
                        isref: false,
                    };
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
                                isptr: false,
                                isref: false,
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
            '%' => {
                if let Some(number) = try_number(&wordbuf) {
                    tokens.push(Token::new(TokenTypes::NUMBER { val: number }, line_number));
                    wordbuf = String::new();
                }
                tokens.push(Token::new(TokenTypes::MOD, line_number));
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
                if let Some(num) = try_number(&wordbuf) {
                    tokens.push(Token::new(TokenTypes::NUMBER { val: num }, line_number));
                    tokens.push(Token::new(TokenTypes::LBRACKET, line_number));
                    continue;
                }
                if let Some(keyword) = test_keyword(&wordbuf, line_number) {
                    tokens.push(keyword);
                    wordbuf = String::new();
                } else if wordbuf.len() > 0 {
                    let ident = Token::new(
                        TokenTypes::IDENT {
                            name: wordbuf,
                            isptr: false,
                            isref: false,
                        },
                        line_number,
                    );
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
                        isptr: false,
                        isref: false,
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
                if let Some(num) = try_number(&wordbuf) {
                    tokens.push(Token::new(TokenTypes::NUMBER { val: num }, line_number));
                    tokens.push(Token::new(TokenTypes::LCURLY, line_number));
                    continue;
                }
                tokens.push(Token::new(TokenTypes::LCURLY, line_number));
            }
            '}' => {
                tokens.push(Token::new(TokenTypes::RCURLY, line_number));
            }
            '|' => {
                tokens.push(Token::new(TokenTypes::OR, line_number));
            }
            '&' => {
                if wordbuf.len() != 0 {
                    if let Some(keyword) = test_keyword(&wordbuf, line_number) {
                        tokens.push(keyword)
                    } else {
                        tokens.push(Token::new(
                            TokenTypes::TEXT {
                                text: wordbuf.clone(),
                            },
                            line_number,
                        ));
                    }
                }
                wordbuf.clear();
                if let Some(and) = iter.peek() {
                    if *and == '&' {
                        tokens.push(Token::new(TokenTypes::AND, line_number));
                        iter.next();
                    } else {
                        tokens.push(Token::new(TokenTypes::AMPER, line_number));
                    }
                }
            }
            ':' => {
                if wordbuf.len() == 0 {
                    tokens.push(Token::new(TokenTypes::COLON, line_number));
                    continue;
                }
                if let Some(keyword) = test_keyword(&wordbuf, line_number) {
                    tokens.push(keyword);
                } else {
                    let ident = TokenTypes::IDENT {
                        name: wordbuf.clone(),
                        isptr: false,
                        isref: false,
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
                    } else if let Some(num) = try_number(&wordbuf) {
                        tokens.push(Token::new(TokenTypes::NUMBER { val: num }, line_number));
                        wordbuf.clear();
                        continue;
                    } else if wordbuf.len() >= 1 {
                        let ident = Token::new(
                            TokenTypes::IDENT {
                                name: wordbuf.clone(),
                                isptr: false,
                                isref: false,
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
    let fixed_tokens: Vec<Token> = tokens
        .clone()
        .into_iter()
        .map(|x| match x.variant {
            TokenTypes::IDENT {
                name: inner,
                isptr: _,
                isref: _,
            } => Token::new(
                TokenTypes::IDENT {
                    name: String::from(inner.trim()),
                    isptr: false,
                    isref: false,
                },
                x.line_num,
            ),
            _ => x,
        })
        .collect();
    let mut tokensiter = fixed_tokens.iter().peekable();
    let mut fixedtokens2 = Vec::new();
    while let Some(next) = tokensiter.next() {
        match next.variant {
            TokenTypes::MINUS => match tokensiter.peek()?.variant {
                TokenTypes::NUMBER { val: previous } => {
                    tokensiter.next();
                    fixedtokens2.push(Token::new(
                        TokenTypes::NUMBER { val: previous * -1 },
                        next.line_num,
                    ));
                    continue;
                }
                _ => {
                    fixedtokens2.push(next.clone());
                }
            },
            _ => {
                fixedtokens2.push(next.clone());
            }
        }
    }

    let mut i = 0;
    while i < fixedtokens2.len() {
        match fixedtokens2[i].variant {
            TokenTypes::STAR => {
                if i + 1 < fixedtokens2.len() {
                    match fixedtokens2[i + 1].variant {
                        TokenTypes::IDENT { name: _, isref: _, ref mut isptr } => {
                            *isptr = true;
                        }
                        TokenTypes::TEXTTYPE(ref mut isptr) => {
                            *isptr = true;
                        }
                        TokenTypes::BOOLTYPE(ref mut isptr) => {
                            *isptr = true;
                        }
                        TokenTypes::NUMTYPE(ref mut isptr) => {
                            *isptr = true;
                        }
                        _ => {continue;}
                    }
                    fixedtokens2.remove(i);
                    continue;
                }
            }
            TokenTypes::AMPER => {
                if i + 1 < fixedtokens2.len() {
                    match fixedtokens2[i + 1].variant {
                        TokenTypes::IDENT { name: _, isptr: _, ref mut isref } => {
                            *isref = true;
                        }
                        _ => {continue;}
                    }
                    fixedtokens2.remove(i);
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }

    Some(fixedtokens2)
}

pub fn test_keyword(word: &String, line: usize) -> Option<Token> {
    match word.as_str() {
        "exit" => Some(Token {
            variant: TokenTypes::EXIT,
            line_num: line,
        }),
        "let" => Some(Token {
            variant: TokenTypes::LET,
            line_num: line,
        }),
        "Num" => Some(Token {
            variant: TokenTypes::NUMTYPE(false),
            line_num: line,
        }),
        "Text" => Some(Token {
            variant: TokenTypes::TEXTTYPE(false),
            line_num: line,
        }),
        "Bool" => Some(Token {
            variant: TokenTypes::BOOLTYPE(false),
            line_num: line,
        }),
        "Func" => Some(Token {
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
        "else" => Some(Token::new(TokenTypes::ELSE, line)),
        "inline" => Some(Token::new(TokenTypes::INLINE, line)),
        "for" => Some(Token::new(TokenTypes::FOR, line)),
        "in" => Some(Token::new(TokenTypes::IN, line)),
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
