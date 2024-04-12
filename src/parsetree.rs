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
        let mut children = Vec::new();
        if let Some(masterblock) = self.parse_until(TokenTypes::EOF, None) {
            children = masterblock.children.clone();
        }
        self.root.children = children;
        &self.root
    }

    pub fn parse_until(
        &mut self,
        end_token: TokenTypes,
        params: Option<Vec<IdentifierNode>>,
    ) -> Option<BlockNode> {
        let mut basetbl = SymbolTable::new();
        if let Some(params) = params {
            for param in params {
                basetbl.insert(param.name.clone(), param.clone());
            }
        }
        let mut newblock = BlockNode {
            children: Vec::new(),
        };
        let mocktkn = Token::new(end_token.clone());
        let eof = (end_token == TokenTypes::EOF);
        self.symbols.push(basetbl);
        'mainloop: while let Some(current) = self.iter.next() {
            if !eof && (current.variant_name() == mocktkn.variant_name()) {
                break 'mainloop;
            }
            match current.variant {
                TokenTypes::LET => {
                    if let Some(node) = self.parse_declare_assign() {
                        let stmt = StatementNode::DeclareAssign(node);
                        newblock.children.push(stmt);
                    }
                }
                _ => {}
            }
        }
        self.symbols.pop();
        Some(newblock)
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
                TokenTypes::FUNCTYPE => {
                    i_type = Types::Function;
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

        if i_type == Types::Function {
            if let Some(val) = self.parse_function() {
                value = Value::Func(val);
                let ident = IdentifierNode::new(&name, &i_type, value);
                if let Some(table) = self.symbols.current_mut() {
                    table.insert(name, ident.clone());
                }
                return Some(DecAssignNode { ident, i_type });
            } else {
                println!("Problem parsing function");
            }
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
                    } else {
                        eprintln!("No identifier {} in the current scope.", name.to_string());
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

    fn parse_function(&mut self) -> Option<Function> {
        let mut fnparams: Option<Vec<IdentifierNode>> = None;
        let mut hierdiefunksie: Option<Function> = None;
        let mut rettype = Types::Nothing;
        if let Some(lbrac) = self.iter.next() {
            if !(Self::val_token(lbrac, "LBRACKET")) {
                return None;
            }
        } else {
            return None;
        }

        if let Some(params) = self.parse_params() {
            fnparams = Some(params);
        } else {
            eprintln!("Could not parse params");
            return None;
        }

        if let Some(arrow) = self.iter.next() {
            if !(Self::val_token(&arrow, "ARROW")) {
                return None;
            }
        } else {
            return None;
        }

        if let Some(rettipe) = self.iter.next() {
            match rettipe.variant {
                TokenTypes::NOTHINGTYPE => {}
                TokenTypes::NUMTYPE => {
                    rettype = Types::Number;
                }
                TokenTypes::TEXTTYPE => {
                    rettype = Types::String;
                }
                TokenTypes::NUMTYPE => {
                    rettype = Types::Number;
                }
                TokenTypes::BOOLTYPE => {
                    rettype = Types::Bool;
                }
                _ => {
                    eprintln!("Expected RETURN TYPE of function. \nIf your function does not return, use 'Nothing'");
                    return None;
                }
            }
        }

        if let Some(lcurl) = self.iter.next() {
            if !(Self::val_token(&lcurl, "LCURLY")) {
                return None;
            }
        }

        if let Some(block) = self.parse_until(TokenTypes::RCURLY, Some(fnparams.clone())?) {
            let func = Function {
                params: Some(fnparams)??,
                ret: rettype,
                body: block,
            };
            hierdiefunksie = Some(func.clone());
        }
        hierdiefunksie
    }

    fn val_token(base: &Token, target: &str) -> bool {
        if !(base.variant_name() == target) {
            eprintln!("Expected {}, but found {}", target, base.variant_name());
            return false;
        }
        true
    }

    fn parse_params(&mut self) -> Option<Vec<IdentifierNode>> {
        let mut params: Vec<IdentifierNode> = Vec::new();
        let mut paramname = String::new();
        let mut paramtype = Types::Nothing;
        'mainloop: while let Some(param) = self.iter.next() {
            if param.variant == TokenTypes::RBRACKET {
                break 'mainloop;
            }
            match &param.variant {
                TokenTypes::IDENT { name } => {
                    paramname = name.to_string();
                    if self.iter.peek().is_some() {
                        let colon = self.iter.next().unwrap();
                        if colon.variant == TokenTypes::COLON {
                            let mut mytype = self.get_type(false);
                            if mytype.is_none() {
                                return None;
                            }
                            let mytype = mytype.unwrap();
                            paramtype = mytype.clone();
                        } else {
                            eprintln!("Expected COLON, found {:?}", colon.variant_name());
                            return None;
                        }
                    } else {
                        eprintln!("Unexpected EOF");
                        return None;
                    }
                }
                _ => {
                    eprintln!("Expected parameter, found {}", param.variant_name());
                    return None;
                }
            }
            let newident = IdentifierNode {
                name: paramname,
                i_type: paramtype,
                value: None,
            };
            params.push(newident);
            paramname = String::new();
            paramtype = Types::Nothing;
        }
        Some(params)
    }

    fn get_type(&mut self, allow_nothing: bool) -> Option<Types> {
        if let Some(typeid) = self.iter.next() {
            match &typeid.variant {
                TokenTypes::FUNCTYPE => Some(Types::Function),
                TokenTypes::NUMTYPE => Some(Types::Number),
                TokenTypes::TEXTTYPE => Some(Types::String),
                TokenTypes::BOOLTYPE => Some(Types::Bool),
                TokenTypes::NOTHINGTYPE => {
                    if allow_nothing {
                        Some(Types::Nothing)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            eprintln!("Expected type annotation, got EOF");
            return None;
        }
    }
}
