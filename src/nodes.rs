use crate::data::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct SymbolTable {
    table: HashMap<String, IdentifierNode>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            table: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, ident: IdentifierNode) {
        self.table.insert(name, ident);
    }

    pub fn contains(&self, name: &String) -> bool {
        self.table.contains_key(name)
    }

    pub fn get(&self, name: &String) -> Option<&IdentifierNode> {
        self.table.get(name)
    }
}

#[derive(Debug)]
pub struct SymbolStack {
    stack: Vec<SymbolTable>,
}

impl SymbolStack {
    pub fn new() -> Self {
        SymbolStack { stack: Vec::new() }
    }

    pub fn push(&mut self, table: SymbolTable) {
        self.stack.push(table);
    }

    pub fn pop(&mut self) -> Option<SymbolTable> {
        self.stack.pop()
    }

    pub fn current(&self) -> Option<&SymbolTable> {
        self.stack.last()
    }

    pub fn current_mut(&mut self) -> Option<&mut SymbolTable> {
        self.stack.last_mut()
    }
}

#[derive(Debug)]
pub struct Program {
    pub children: Vec<StatementNode>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            children: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum StatementNode {
    Assign(AssignNode),
    Declare(DeclareNode),
    DeclareAssign(DecAssignNode),
    Call(CallNode),
    Conditional(ConditionalNode),
    Return(ReturnNode),
}

#[derive(Debug, Clone)]
pub enum Number {
    Lit(NumLiteral),
    Exp(Box<NumExpression>),
}

#[derive(Debug, Clone)]
pub struct NumExpression {
    pub left: Box<Number>,
    pub operator: Operators,
    pub right: Box<Number>,
}

impl NumExpression {
    pub fn new(left: Number, operator: Operators, right: Number) -> Self {
        let left = Box::new(left);
        let right = Box::new(right);
        NumExpression {
            left,
            operator,
            right,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operators {
    Plus,
    Minus,
}

#[derive(Debug, Clone)]
pub struct NumLiteral {
    pub val: i32,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Types {
    Number,
    String,
    Bool,
    Function,
}

#[derive(Debug)]
pub struct BlockNode {
    pub start: String,
    pub children: Vec<StatementNode>,
    pub end: String,
}

#[derive(Debug, Clone)]
pub struct IdentifierNode {
    pub name: String,
    pub i_type: Types,
    pub value: Option<Box<Value>>,
}

impl IdentifierNode {
    pub fn new(name: &String, i_type: &Types, val: Value) -> Self {
        let i_type = i_type.clone();
        let valbox = Box::new(val);
        IdentifierNode {
            name: name.clone(),
            i_type,
            value: Some(valbox),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Lit(Literal),
    Expr(Expression),
    Ident(IdentifierNode),
    Nothing,
}

impl Value {
    pub fn is_nothing(&self) -> bool {
        match self {
            Value::Nothing => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Num(NumExpression),
    Bool(BoolExpr),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Num(NumLiteral),
    Bool(BoolLiteral),
    Text(TextLit),
}

#[derive(Debug)]
pub struct AssignNode {
    pub ident: IdentifierNode,
}

#[derive(Debug)]
pub struct DecAssignNode {
    pub ident: IdentifierNode,
    pub i_type: Types,
}

#[derive(Debug)]
pub struct DeclareNode {
    pub ident: IdentifierNode,
    pub i_type: Types,
}

#[derive(Debug)]
pub struct CallNode {
    pub func: IdentifierNode,
    pub params: Vec<IdentifierNode>,
}

#[derive(Debug)]
pub struct ConditionalNode {
    pub condition: Bool,
    pub block: BlockNode,
}

#[derive(Debug, Clone)]
pub enum Bool {
    Lit(BoolLiteral),
    Expr(BoolExpr),
}

#[derive(Debug, Clone)]
pub enum BoolLiteral {
    True,
    False,
}

#[derive(Debug, Clone)]
pub struct BoolExpr {
    left: Box<Value>,
    operator: BoolOps,
    right: Box<Value>,
}

#[derive(Debug, Clone)]
pub enum BoolOps {
    EQ,
    Greater,
    Lesser,
}

#[derive(Debug)]
pub struct ReturnNode {
    pub value: Option<Value>,
}

#[derive(Debug)]
pub struct NumExpressionNode {
    pub value: i32,
}

#[derive(Debug, Clone)]
pub struct TextLit {
    pub value: String,
}
