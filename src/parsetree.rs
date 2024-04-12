use crate::data::*;
use crate::nodes::*;
use std::cmp::PartialEq;

use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug)]
pub struct ParseTree<'a> {
    pub index: usize,
    pub iter: Peekable<Iter<'a, Token>>,
    pub root: Program,
    pub symbols: SymbolStack,
}

impl<'a> ParseTree<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        ParseTree {
            index: 0_usize,
            iter: tokens.iter().peekable(),
            root: Program::new(),
            symbols: SymbolStack::new(),
        }
    }

    pub fn parse(&mut self) -> &Program {
        let basetbl = SymbolTable::new();
        self.symbols.push(basetbl);
        'mainloop: while let Some(current) = self.iter.next() {
            match current.variant {
                TokenTypes::LET => {
                    if let Some(node) = self.parse_declare_assign() {
                        let stmt = StatementNode::DeclareAssign(node);
                        self.root.children.push(stmt);
                    }
                }
                _ => {}
            }
        }
        &self.root
    }

    fn parse_declare_assign(&mut self) -> Option<DecAssignNode> {
        let mut name: String = String::new();
        let mut value: Value = Value::Nothing;
        let mut i_type: Types;
        if let Some(next) = self.iter.next() {
            match &next.variant {
                TokenTypes::IDENT { name: nombre } => {
                    name = nombre.clone();
                }
                _ => {
                    eprintln!(
                        "Expected identifier after let but found {}",
                        next.variant_name()
                    );
                    return None;
                }
            }
        }
        if let Some(col) = self.iter.next() {
            if col.variant_name() != "COLON" {
                eprintln!("Expected ':' after declaration");
                return None;
            }
        } else {
            eprintln!("EOF after LET");
            return None;
        }

        if let Some(typetkn) = self.iter.next() {
            match &typetkn.variant {
                TokenTypes::NUMTYPE => {
                    i_type = Types::Number;
                }
                TokenTypes::TEXTTYPE => {
                    i_type = Types::String;
                }
                TokenTypes::BOOLTYPE => {
                    i_type = Types::Bool;
                }
                _ => {
                    eprintln!("Expected TYPE during declaration");
                    return None;
                }
            }
        } else {
            eprintln!("Expected TYPE, found EOF");
            return None;
        }

        if let Some(eqtoken) = self.iter.next() {
            if eqtoken.variant_name() != "EQ" {
                eprintln!("Expected EQ, got {}", eqtoken.variant_name());
                return None;
            }
        } else {
            eprintln!("Expected '=' during assignment");
            return None;
        }

        if let Some(valtoken) = self.iter.next() {
            match &valtoken.variant {
                TokenTypes::NUMBER { val } => {
                    match i_type {
                        Types::Number => {}
                        _ => {
                            eprintln!("Expected {:?}, but got expression of type Number", i_type);
                            return None;
                        }
                    }
                    let val = val.clone();
                    let num = NumLiteral { val };
                    let lit = Literal::Num(num);
                    value = Value::Lit(lit);
                }
                TokenTypes::IDENT { name } => {
                    if let Some(ident) = self.symbols.current()?.get(&name) {
                        if ident.i_type == i_type {
                            value = Value::Ident(ident.clone());
                        }
                    }
                }
                TokenTypes::BOOL { val } => {
                    match i_type {
                        Types::Bool => {}
                        _ => {
                            eprintln!("Expected {:?}, but got expression of type Bool", i_type);
                            return None;
                        }
                    }
                    let boolval = {
                        if *val {
                            BoolLiteral::True
                        } else {
                            BoolLiteral::False
                        }
                    };
                    value = Value::Lit(Literal::Bool(boolval));
                }
                TokenTypes::TEXT { text } => {
                    match i_type {
                        Types::String => {}
                        _ => {
                            eprintln!("Expected {:?}, but got expression of type String", i_type);
                            return None;
                        }
                    }
                    value = Value::Lit(Literal::Text(TextLit {
                        value: text.clone(),
                    }));
                }
                _ => {
                    eprintln!("Expected a value after EQ");
                    return None;
                }
            }
        } else {
            eprintln!("Got EOF after EQ");
            return None;
        }
        if !value.is_nothing() {
            let ident = IdentifierNode::new(&name, &i_type, value);
            if let Some(table) = self.symbols.current_mut() {
                table.insert(name, ident.clone());
            }
            return Some(DecAssignNode { ident, i_type });
        } else {
            eprintln!("Problem parsing delcaration");
            return None;
        }
    }
}
