use std::iter::Peekable;
use std::slice::Iter;

use crate::lexing::data::*;
use crate::parsing::nodes::*;

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
        if let Some(masterblock) = self.parse_until(TokenTypes::EOF, None, None) {
            children = masterblock.children.clone();
        }
        self.root.children = children;
        &self.root
    }

    pub fn parse_until(
        &mut self,
        end_token: TokenTypes,
        params: Option<Vec<IdentifierNode>>,
        rettype: Option<Types>,
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
        let mocktkn = Token::new(end_token.clone(), 0);
        let eof = end_token == TokenTypes::EOF;

        // MANUALLY INSTERTING STD FUNCTIONS, WILL FIX LATER

        if eof {
            let showme = Function {
                name: String::from("showme"),
                body: BlockNode {
                    children: Vec::new(),
                },
                ret: Types::Nothing,
                params: Vec::new(),
            };
            let showmeident = IdentifierNode {
                name: String::from("showme"),
                i_type: Types::Function,
                value: Some(Box::new(Value::Func(showme))),
            };
            basetbl.insert(String::from("showme"), showmeident);

            let efailure = IdentifierNode {
                name: String::from("EXIT_FAILURE"),
                value: Some(Box::from(Value::Lit(Literal::Num(NumLiteral { val: 1 })))),
                i_type: Types::Number,
            };
            basetbl.insert(String::from("EXIT_FAILURE"), efailure);

            let esucc = IdentifierNode {
                name: String::from("EXIT_SUCCESS"),
                value: Some(Box::from(Value::Lit(Literal::Num(NumLiteral { val: 0 })))),
                i_type: Types::Number,
            };
            basetbl.insert(String::from("EXIT_SUCCESS"), esucc);
        }

        self.symbols.push(basetbl);
        'mainloop: while let Some(current) = self.iter.next() {
            if !eof && (current.variant_name() == mocktkn.variant_name()) {
                break 'mainloop;
            }
            match &current.variant {
                TokenTypes::LET => {
                    if let Some(node) = self.parse_declare_assign() {
                        let stmt = StatementNode::DeclareAssign(node);
                        newblock.children.push(stmt);
                    }
                }
                TokenTypes::RETURN => {
                    if rettype.is_none() {
                        eprintln!("line {}: Cannot return from global scope", current.line_num);
                        return None;
                    }
                    let targtype = rettype.clone().unwrap();
                    if let Some(value) = self.parse_return() {
                        match value.value {
                            Value::Ident(ref inner) => {
                                if inner.i_type != targtype {
                                    eprintln!("Line {}: Mismatched return type.", current.line_num);
                                    return None;
                                }
                            }
                            Value::Nothing => {
                                if targtype != Types::Nothing {
                                    eprintln!(
                                        "Line {}: Mismatched return type, found Nothing",
                                        current.line_num
                                    );
                                    return None;
                                }
                            }
                            _ => {}
                        }
                        let stmt = StatementNode::Return(value.clone());
                        newblock.children.push(stmt);
                    }
                }
                TokenTypes::IF => {
                    let Some(cond) = self.parse_conditional(rettype.clone()?) else {
                        eprintln!("Line {}: Failed to parse conditional", current.line_num);
                        return None;
                    };
                    let stmt = StatementNode::Conditional(cond);
                    newblock.children.push(stmt);
                }
                TokenTypes::INLINE => {
                    let Some(inlinenode) = self.parse_inline() else {
                        eprintln!("Line {}: Failed to parse inline", current.line_num);
                        return None;
                    };
                    let stmt = StatementNode::Inline(inlinenode);
                    newblock.children.push(stmt);
                }
                TokenTypes::IDENT {
                    name,
                    isref: _,
                    isptr: _,
                } => {
                    let Some(lbrac) = self.iter.next() else {
                        eprintln!("Line {}: Expected '(' during call", current.line_num);
                        return None;
                    };
                    if lbrac.variant != TokenTypes::LBRACKET {
                        eprintln!(
                            "Line {}: Expected '(' during function call",
                            current.line_num
                        );
                        let mockfunc = Function {
                            params: Vec::new(),
                            name: String::from(""),
                            body: BlockNode {
                                children: Vec::new(),
                            },
                            ret: Types::Nothing,
                        };
                        let _ = self.parse_call(mockfunc);
                        return None;
                    }
                    let Some(func) = self.symbols.search_down(name) else {
                        eprintln!(
                            "Line {}: No identifier {} found in current scope",
                            current.line_num, name
                        );
                        return None;
                    };
                    match func.value {
                        Some(mybox) => match *mybox {
                            Value::Func(myfunc) => {
                                let callnode = self.parse_call(myfunc)?;
                                newblock.children.push(StatementNode::Call(callnode));
                            }
                            _ => {
                                eprintln!(
                                    "Line {}: Identifier {} is not a function",
                                    current.line_num, func.name
                                );
                                return None;
                            }
                        },
                        None => panic!(),
                    }
                }
                TokenTypes::FOR => {
                    let forstmt = self.parse_for(rettype.clone()?)?;
                    newblock.children.push(StatementNode::ForLoop(forstmt));
                }
                _ => {}
            }
        }
        self.symbols.pop();
        Some(newblock)
    }

    fn parse_inline(&mut self) -> Option<InlineC> {
        let next = self.iter.next()?;
        match &next.variant {
            TokenTypes::TEXT { text } => Some(InlineC(text.clone())),
            _ => None,
        }
    }

    fn parse_return(&mut self) -> Option<ReturnNode> {
        if let Some(ident) = self.iter.next() {
            match &ident.variant {
                TokenTypes::IDENT {
                    name,
                    isptr: _,
                    isref: _,
                } => {
                    if let Some(identnode) = self.symbols.search_down(name) {
                        let retnode = ReturnNode {
                            value: Value::Ident(identnode.clone()),
                        };
                        if let Some(semi) = self.iter.peek() {
                            if semi.variant_name() == "SEMI" {
                                self.iter.next();
                                return Some(retnode);
                            } else {
                                eprintln!("Line {}: Expected SEMI", semi.line_num);
                                return None;
                            }
                        } else {
                            eprintln!("Line {}: Expected SEMI, found EOF", ident.line_num);
                            return None;
                        }
                    } else {
                        eprintln!(
                            "Line {}: No identifier {name} found in current scope",
                            ident.line_num
                        );
                        return None;
                    }
                }
                TokenTypes::SEMI => {
                    return Some(ReturnNode {
                        value: Value::Nothing,
                    });
                }
                _ => {
                    eprintln!("Line {}: A function must either return an identifier or nothing. assign the value you want to return to a variable, then return that variable.", ident.line_num);
                    return None;
                }
            }
        } else {
            eprintln!("Expected IDENT, found EOF");
            return None;
        }
    }

    fn parse_declare_assign(&mut self) -> Option<DecAssignNode> {
        let mut name: String = String::new();
        let mut value: Value = Value::Nothing;
        let mut i_type: Types;
        if let Some(next) = self.iter.next() {
            match &next.variant {
                TokenTypes::IDENT {
                    name: nombre,
                    isref: _,
                    isptr: _,
                } => {
                    name = nombre.clone();
                }
                _ => {
                    eprintln!(
                        "Line {}: Expected identifier after let but found {}",
                        next.line_num,
                        next.variant_name()
                    );
                    return None;
                }
            }
        }
        if let Some(col) = self.iter.next() {
            if col.variant_name() != "COLON" {
                eprintln!("Line {}: Expected ':' after declaration", col.line_num);
                return None;
            }
        } else {
            eprintln!("EOF after LET");
            return None;
        }

        if let Some(typetkn) = self.iter.next() {
            match &typetkn.variant {
                TokenTypes::NUMTYPE(isptr) => {
                    i_type = {
                        if *isptr {
                            Types::Pointer(Box::new(Types::Number))
                        } else {
                            Types::Number
                        }
                    };
                }
                TokenTypes::TEXTTYPE(isptr) => {
                    i_type = {
                        if *isptr {
                            Types::Pointer(Box::new(Types::String))
                        } else {
                            Types::String
                        }
                    };
                }
                TokenTypes::BOOLTYPE(isptr) => {
                    i_type = {
                        if *isptr {
                            Types::Pointer(Box::new(Types::Bool))
                        } else {
                            Types::Bool
                        }
                    };
                }
                TokenTypes::FUNCTYPE => {
                    i_type = Types::Function;
                }
                _ => {
                    eprintln!(
                        "Line {}: Expected TYPE during declaration",
                        typetkn.line_num
                    );
                    return None;
                }
            }
        } else {
            eprintln!("Expected TYPE, found EOF");
            return None;
        }

        if let Some(eqtoken) = self.iter.next() {
            if eqtoken.variant_name() != "EQ" {
                eprintln!(
                    "Line {}: Expected EQ, got {}",
                    eqtoken.line_num,
                    eqtoken.variant_name()
                );
                return None;
            }
        } else {
            eprintln!("Expected '=' during assignment");
            return None;
        }

        if i_type == Types::Function {
            if let Some(val) = self.parse_function(&name) {
                value = Value::Func(val);
                let ident = IdentifierNode::new(&name, &i_type, value);
                if let Some(table) = self.symbols.current_mut() {
                    table.insert(name, ident.clone());
                }
                return Some(DecAssignNode { ident, i_type });
            } else {
                println!("Problem parsing function");
            }
        } else if i_type == Types::Bool {
            let Some(val) = self.parse_bool() else {
                return None;
            };
            let value: Value = {
                match val {
                    Bool::Lit(lit) => Value::Lit(Literal::Bool(lit)),
                    Bool::Expr(expr) => Value::Expr(Expression::Bool(expr)),
                    Bool::Ident(ident) => Value::Ident(ident),
                    Bool::Call(call) => Value::Call(call),
                }
            };
            let ident = IdentifierNode::new(&name, &i_type, value);
            if let Some(table) = self.symbols.current_mut() {
                table.insert(name, ident.clone());
            }
            return Some(DecAssignNode { ident, i_type });
        } else if i_type == Types::Number {
            let val = self.parse_number()?;
            let value: Value = {
                match val {
                    Number::Lit(somelit) => Value::Lit(Literal::Num(somelit)),
                    Number::Ident(someident) => Value::Ident(someident),
                    Number::Exp(someexpr) => Value::Expr(Expression::Num(*someexpr)),
                    Number::Call(somecall) => Value::Call(somecall),
                }
            };
            let ident = IdentifierNode::new(&name, &i_type, value);
            if let Some(table) = self.symbols.current_mut() {
                table.insert(name, ident.clone());
            }
            return Some(DecAssignNode { ident, i_type });
        }

        if let Some(valtoken) = self.iter.next() {
            match &valtoken.variant {
                TokenTypes::NUMBER { val } => {
                    match i_type {
                        Types::Number => {}
                        _ => {
                            eprintln!(
                                "Line {}: Expected {:?}, but got expression of type Number",
                                valtoken.line_num, i_type
                            );
                            return None;
                        }
                    }
                    let val = val.clone();
                    let num = NumLiteral { val };
                    let lit = Literal::Num(num);
                    value = Value::Lit(lit);
                }
                TokenTypes::IDENT { name, isptr, isref } => {
                    if let Some(ident) = self.symbols.search_down(&name) {
                        if ident.i_type == Types::Function {
                            let Some(lbrac) = self.iter.next() else {
                                return None;
                            };
                            if lbrac.variant != TokenTypes::LBRACKET {
                                eprintln!("Line {}: Expected '('", valtoken.line_num);
                                return None;
                            }
                            let Some(mybox) = ident.value else {
                                panic!();
                            };
                            let Value::Func(myfunc) = *mybox else {
                                panic!();
                            };
                            if myfunc.ret != i_type {
                                eprintln!(
                                    "Line {}: Mismatched return type for call assignement",
                                    valtoken.line_num
                                );
                                return None;
                            }
                            let Some(val) = self.parse_call(myfunc) else {
                                return None;
                            };
                            value = Value::Call(val.clone());
                        } else {
                            if ident.i_type == i_type {
                                value = Value::Ident(ident.clone());
                            } else if *isptr {
                                //TODO: it should be isderef, not isptr, as a * before a variable
                                //name means you are extracting the value from the pointer, and
                                //isref should be isptr, as putting a & in front turns it into a
                                //pointer
                                let Types::Pointer(ptrtype) = ident.i_type else {
                                    return None;
                                };
                                if *ptrtype == i_type {
                                    value = *(ident.value?).clone();
                                }
                            } else if *isref {
                                let Types::Pointer(ref ptrtype) = i_type else {
                                    return None;
                                };
                                if **ptrtype == ident.i_type {
                                    value = Value::Pointer(Box::new(Value::Ident(ident)));
                                }
                            }
                        }
                    } else {
                        eprintln!(
                            "Line {}: No identifier {} in the current scope.",
                            valtoken.line_num,
                            name.to_string()
                        );
                    }
                }
                TokenTypes::BOOL { val } => {
                    match i_type {
                        Types::Bool => {}
                        _ => {
                            eprintln!(
                                "Line {}: Expected {:?}, but got expression of type Bool",
                                valtoken.line_num, i_type
                            );
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
                            eprintln!(
                                "Line {}: Expected {:?}, but got expression of type String",
                                valtoken.line_num, i_type
                            );
                            return None;
                        }
                    }
                    value = Value::Lit(Literal::Text(TextLit {
                        value: text.clone(),
                    }));
                }
                _ => {
                    eprintln!(
                        "Line {}: Expected a value after EQ, but got {:?}",
                        valtoken.line_num, valtoken
                    );
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
            return Some(DecAssignNode {
                ident,
                i_type: i_type.clone(),
            });
        } else {
            eprintln!("Problem parsing delcaration");
            return None;
        }
    }

    fn parse_number(&mut self) -> Option<Number> {
        let next = self.iter.next()?;
        let left: Number = {
            match &next.variant {
                TokenTypes::NUMBER { val } => Number::Lit(NumLiteral { val: *val }),
                TokenTypes::IDENT {
                    name,
                    isptr: _,
                    isref: _,
                } => {
                    let Some(numdent) = self.symbols.search_down(&name) else {
                        eprintln!("Line {}: No identifier {} found", next.line_num, name);
                        return None;
                    };

                    match numdent.i_type {
                        Types::Number => Number::Ident(numdent),
                        Types::Function => match *(numdent.value?) {
                            Value::Func(somefunc) => {
                                if somefunc.ret != Types::Number {
                                    eprintln!("Mismatched types");
                                    return None;
                                }
                                let callnode = self.parse_call(somefunc)?;
                                Number::Call(callnode)
                            }
                            _ => {
                                panic!();
                            }
                        },
                        _ => {
                            eprintln!("Mismatched types");
                            return None;
                        }
                    }
                }
                _ => {
                    return None;
                }
            }
        };

        let next = self.iter.peek()?;
        let op = {
            match next.variant {
                TokenTypes::SEMI => {
                    return Some(left);
                }
                TokenTypes::DOT => {
                    return Some(left);
                }
                TokenTypes::MOD => Operators::Mod,
                TokenTypes::PLUS => Operators::Plus,
                TokenTypes::MINUS => Operators::Minus,
                TokenTypes::LCURLY => {
                    return Some(left);
                }
                _ => {
                    eprintln!("Line {}: Invalid operator.", next.line_num);
                    return None;
                }
            }
        };
        self.iter.next();

        let next = self.iter.next()?;
        let right = {
            match &next.variant {
                TokenTypes::IDENT {
                    name,
                    isptr: _,
                    isref: _,
                } => {
                    let Some(numdent) = self.symbols.search_down(&name) else {
                        eprintln!(
                            "Line {}: No identifier {} found in current scope",
                            next.line_num, name
                        );
                        return None;
                    };
                    match numdent.i_type {
                        Types::Number => Number::Ident(numdent),
                        _ => {
                            eprintln!("Line {}: Mismatched types", next.line_num);
                            return None;
                        }
                    }
                }
                TokenTypes::NUMBER { val } => Number::Lit(NumLiteral { val: *val }),
                _ => {
                    return None;
                }
            }
        };
        let num_expression = NumExpression {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
        };
        Some(Number::Exp(Box::new(num_expression)))
    }

    fn parse_bool(&mut self) -> Option<Bool> {
        let Some(next) = self.iter.next() else {
            eprintln!("Could not parse bool");
            return None;
        };
        match &next.variant {
            TokenTypes::BOOL { val } => {
                let inner = {
                    if *val {
                        BoolLiteral::True
                    } else {
                        BoolLiteral::False
                    }
                };
                Some(Bool::Lit(inner))
            }
            TokenTypes::IDENT {
                name,
                isptr: _,
                isref: _,
            } => {
                let Some(node) = self.symbols.search_down(&name) else {
                    eprintln!(
                        "Line {}: No identifier {} found in current scope",
                        next.line_num, name
                    );
                    return None;
                };
                if node.i_type == Types::Bool {
                    Some(Bool::Ident(node.clone()))
                } else if node.i_type == Types::Function {
                    let lbrac = self.iter.next()?;
                    if lbrac.variant != TokenTypes::LBRACKET {
                        eprintln!(
                            "Line {}: Expected '(', found {} ",
                            lbrac.line_num,
                            lbrac.variant_name()
                        );
                    }
                    match *node.value? {
                        Value::Func(func) => {
                            let callnode = self.parse_call(func)?;
                            Some(Bool::Call(callnode))
                        }
                        _ => panic!(),
                    }
                } else {
                    Some(Bool::Expr(self.parse_bool_expression(node.clone())?))
                }
            }
            _ => todo!(),
        }
    }

    fn parse_for(&mut self, functype: Types) -> Option<ForNode> {
        let ident = self.iter.next()?;

        let ident: IdentifierNode = match &ident.variant {
            TokenTypes::IDENT {
                name,
                isptr: _,
                isref: _,
            } => IdentifierNode {
                name: name.to_string(),
                i_type: Types::Number,
                value: None,
            },
            _ => {
                return None;
            }
        };

        let inkw = self.iter.next()?;
        if inkw.variant != TokenTypes::IN {
            return None;
        }

        let start = self.parse_number()?;

        let (dot1, dot2) = (self.iter.next()?, self.iter.next()?);
        if !(dot1.variant == dot2.variant && dot2.variant == TokenTypes::DOT) {
            return None;
        }

        let end = self.parse_number()?;

        let lcurly = self.iter.next()?;
        if lcurly.variant != TokenTypes::LCURLY {
            return None;
        }

        let body = self.parse_until(
            TokenTypes::RCURLY,
            Some(vec![ident.clone()]),
            Some(functype),
        )?;

        let range = RangeNode { start, end };

        Some(ForNode {
            dec: ident,
            range,
            body,
        })
    }

    fn parse_function(&mut self, name: &String) -> Option<Function> {
        let mut fnparams: Option<Vec<IdentifierNode>> = None;
        let mut hierdiefunksie: Option<Function> = None;
        let mut rettype = Types::Nothing;
        if let Some(lbrac) = self.iter.next() {
            if !Self::val_token(lbrac, "LBRACKET") {
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
            if !Self::val_token(&arrow, "ARROW") {
                return None;
            }
        } else {
            return None;
        }

        if let Some(rettipe) = self.iter.next() {
            match rettipe.variant {
                TokenTypes::NOTHINGTYPE => {}
                TokenTypes::NUMTYPE(isptr) => {
                    rettype = {
                        if isptr {
                            Types::Pointer(Box::new(Types::Number))
                        } else {
                            Types::Number
                        }
                    }
                }
                TokenTypes::TEXTTYPE(isptr) => {
                    rettype = {
                        if isptr {
                            Types::Pointer(Box::new(Types::String))
                        } else {
                            Types::String
                        }
                    }
                }
                TokenTypes::BOOLTYPE(isptr) => {
                    rettype = {
                        if isptr {
                            Types::Pointer(Box::new(Types::Bool))
                        } else {
                            Types::Bool
                        }
                    }
                }
                _ => {
                    eprintln!("line {}: Expected RETURN TYPE of function. \nIf your function does not return, use 'Nothing'", rettipe.line_num);
                    return None;
                }
            }
        }

        if let Some(lcurl) = self.iter.next() {
            if !Self::val_token(&lcurl, "LCURLY") {
                return None;
            }
        }

        if let Some(block) = self.parse_until(
            TokenTypes::RCURLY,
            Some(fnparams.clone())?,
            Some(rettype.clone()),
        ) {
            let func = Function {
                name: name.clone(),
                params: Some(fnparams)??,
                ret: rettype.clone(),
                body: block,
            };
            hierdiefunksie = Some(func.clone());
        }
        hierdiefunksie
    }

    fn val_token(base: &Token, target: &str) -> bool {
        if !(base.variant_name() == target) {
            eprintln!(
                "Line {}: Expected {}, but found {}",
                base.line_num,
                target,
                base.variant_name()
            );
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
                TokenTypes::IDENT {
                    name,
                    isptr: _,
                    isref: _,
                } => {
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
                            eprintln!(
                                "Line {}: Expected COLON, found {:?}",
                                colon.line_num,
                                colon.variant_name()
                            );
                            return None;
                        }
                    } else {
                        eprintln!("Unexpected EOF");
                        return None;
                    }
                }
                TokenTypes::COMMA => {
                    continue;
                }
                _ => {
                    eprintln!(
                        "Line {}: Expected parameter, found {}",
                        param.line_num,
                        param.variant_name()
                    );
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
                TokenTypes::NUMTYPE(isptr) => {
                    if *isptr {
                        Some(Types::Pointer(Box::new(Types::Number)))
                    } else {
                        Some(Types::Number)
                    }
                }
                TokenTypes::TEXTTYPE(isptr) => {
                    if *isptr {
                        Some(Types::Pointer(Box::new(Types::String)))
                    } else {
                        Some(Types::String)
                    }
                }
                TokenTypes::BOOLTYPE(isptr) => {
                    if *isptr {
                        Some(Types::Pointer(Box::new(Types::Bool)))
                    } else {
                        Some(Types::Bool)
                    }
                }
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

    fn parse_conditional(&mut self, functype: Types) -> Option<ConditionalNode> {
        let mut thisbool = Bool::Lit(BoolLiteral::False);
        if let Some(next) = self.iter.next() {
            match &next.variant {
                TokenTypes::IDENT {
                    name,
                    isptr: _,
                    isref: _,
                } => {
                    if let Some(boolident) = self.symbols.search_down(&name) {
                        if boolident.i_type == Types::Bool {
                            thisbool = Bool::Ident(boolident.clone());
                        } else {
                            // case of ident after if, will check for bool
                            // expression eg. ident1 == ident2
                            let Some(expr) = self.parse_bool_expression(boolident.clone()) else {
                                eprintln!(
                                    "Line {}: Could not parse bool expression.",
                                    next.line_num
                                );
                                return None;
                            };
                            thisbool = Bool::Expr(expr);
                        }
                    } else {
                        eprintln!("Could not find identifier {name} in current scope");
                        return None;
                    }
                }
                TokenTypes::BOOL { val } => {
                    thisbool = {
                        if *val {
                            Bool::Lit(BoolLiteral::True)
                        } else {
                            Bool::Lit(BoolLiteral::False)
                        }
                    };
                }
                _ => {
                    eprintln!(
                        "Line {}: Expected bool literal or bool identifier.",
                        next.line_num
                    );
                    return None;
                }
            }
            let Some(lcurly) = self.iter.next() else {
                eprintln!("Unexpected EOF");
                return None;
            };
            if lcurly.variant == TokenTypes::LCURLY {
                let Some(body) = self.parse_until(TokenTypes::RCURLY, None, Some(functype.clone()))
                else {
                    eprintln!(
                        "Line {}: Could not parse conditional body.",
                        lcurly.line_num
                    );
                    return None;
                };
                let s_else = self.iter.peek()?;
                if s_else.variant != TokenTypes::ELSE {
                    let condnode = ConditionalNode {
                        condition: thisbool,
                        body,
                        i_else: None,
                    };
                    return Some(condnode);
                }
                let s_else = self.iter.next()?;
                let Some(lcurly) = self.iter.next() else {
                    eprintln!("Line {}: Expected '{{'", s_else.line_num);
                    return None;
                };
                if lcurly.variant != TokenTypes::LCURLY {
                    eprintln!("Line {}: Expected '{{'", lcurly.line_num);
                    return None;
                }
                let Some(elsebody) = self.parse_until(TokenTypes::RCURLY, None, Some(functype))
                else {
                    eprintln!("Line {}: Could not parse else body", lcurly.line_num);
                    return None;
                };
                let condnode = ConditionalNode {
                    condition: thisbool,
                    body,
                    i_else: Some(elsebody),
                };
                return Some(condnode);
            }
        }
        eprintln!("Unexpected EOF");
        None
    }

    //TODO: why is it getting here when declaring a boolean in the body of an if statement?
    fn parse_bool_expression(&mut self, left: IdentifierNode) -> Option<BoolExpr> {
        let mut op = BoolOps::EQ;
        let Some(next) = self.iter.next() else {
            return None;
        };
        let mut right = Token::new(TokenTypes::NULL, 0);
        match next.variant {
            TokenTypes::BOOLEQ => {
                op = BoolOps::EQ;
            }
            TokenTypes::GREATER => {
                op = BoolOps::Greater;
            }
            TokenTypes::LESSER => {
                op = BoolOps::Lesser;
            }
            _ => {
                eprintln!("Unexpected boolean operator");
                return None;
            }
        }

        let Some(rightc) = self.iter.next() else {
            eprintln!(
                "Line {}: Expected an expression to the right of '=='",
                next.line_num
            );
            return None;
        };
        right = rightc.clone();

        match &right.variant {
            TokenTypes::IDENT {
                name,
                isptr: _,
                isref: _,
            } => {
                let Some(rightnode) = self.symbols.search_down(&name) else {
                    eprintln!(
                        "Line {}: Could not find identifier {} in the current scope",
                        right.line_num, name,
                    );
                    return None;
                };
                if rightnode.i_type != left.i_type {
                    eprintln!(
                        "Line {}: Cannot compare values of different types.",
                        right.line_num
                    );
                    return None;
                }
                let expr = BoolExpr {
                    left: Box::new(Value::Ident(left)),
                    operator: op,
                    right: Box::new(Value::Ident(rightnode.clone())),
                };
                return Some(expr);
            }
            _ => {
                eprintln!(
                    "Line {}: Only identifers can be used in boolean expressions",
                    right.line_num
                );
                return None;
            }
        }
    }

    pub fn parse_call(&mut self, func: Function) -> Option<CallNode> {
        let mut args = Vec::new();
        let mut nocomma = false;
        while let Some(param) = self.iter.next() {
            match &param.variant {
                TokenTypes::RBRACKET => {
                    break;
                }
                TokenTypes::IDENT {
                    name,
                    isptr: _,
                    isref: _,
                } => {
                    args.push(self.parse_arg_ident(name, param.line_num)?);
                }
                TokenTypes::NUMBER { val } => {
                    args.push(Value::Lit(Literal::Num(NumLiteral { val: val.clone() })));
                }
                TokenTypes::TEXT { text: text1 } => {
                    args.push(Value::Lit(Literal::Text(TextLit {
                        value: text1.clone(),
                    })));
                }
                TokenTypes::BOOL { val } => {
                    let inner = {
                        if *val {
                            BoolLiteral::True
                        } else {
                            BoolLiteral::False
                        }
                    };
                    args.push(Value::Lit(Literal::Bool(inner)));
                }
                _ => {}
            }
            let comma = self.iter.peek()?;
            if comma.variant == TokenTypes::COMMA {
                self.iter.next();
            } else if comma.variant == TokenTypes::RBRACKET {
                self.iter.next();
                break;
            }
        }

        for val in &args {
            match val {
                Value::Nothing => panic!("not valid"),
                _ => {}
            }
        }
        // dbg!(args.clone());

        if func.name == String::from("showme") {
            let params_res: Option<Vec<IdentifierNode>> = args
                .into_iter()
                .map(|val| IdentifierNode::from(val.clone()))
                .collect();
            let params = params_res?;
            return Some(CallNode {
                func: func.clone(),
                params,
            });
        }

        if args.len() != func.params.len() {
            eprintln!(
                "Wrong number of arguments for call to function {}",
                func.name
            );
            return None;
        }
        let mut params_res: Option<Vec<IdentifierNode>> = args
            .into_iter()
            .map(|val| IdentifierNode::from(val.clone()))
            .collect();
        let params = params_res?;
        for (index, param) in func.params.iter().enumerate() {
            if param.i_type != params[index].i_type {
                return None;
            }
        }
        Some(CallNode {
            func: func.clone(),
            params,
        })
    }

    fn parse_arg_ident(&self, name: &String, line: usize) -> Option<Value> {
        let Some(idnode) = self.symbols.search_down(name) else {
            eprintln!("Line {line}: No identifier {name} found");
            return None;
        };
        match idnode.i_type {
            Types::Function | Types::Nothing => None,
            _ => Some(Value::Ident(idnode.clone())),
        }
    }
}
