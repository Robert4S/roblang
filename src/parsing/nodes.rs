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

    pub fn search_down(&self, name: &String) -> Option<IdentifierNode> {
        for table in self.stack.iter().rev() {
            if let Some(ident) = table.get(name) {
                return Some(ident.clone());
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub children: Vec<StatementNode>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            children: Vec::new(),
        }
    }

    pub fn check_top(&self) -> Option<()> {
        let mut maincheck = false;
        for child in &self.children {
            match child {
                StatementNode::DeclareAssign(node) => {
                    if node.i_type == Types::Function {
                        if node.ident.name == "main".to_string() {
                            maincheck = true;
                        }
                        continue;
                    }
                    eprintln!("Top level code coming soon");
                    return None;
                }
                _ => {
                    eprintln!("Top level code coming soon");
                    return None;
                }
            }
        }
        if maincheck {
            Some(())
        } else {
            eprintln!("No main function found");
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum StatementNode {
    Assign(AssignNode),
    Declare(DeclareNode),
    DeclareAssign(DecAssignNode),
    Call(CallNode),
    Conditional(ConditionalNode),
    Return(ReturnNode),
    Inline(InlineC),
}
#[derive(Debug, Clone)]
pub struct InlineC(pub String);

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
    Nothing,
}

#[derive(Debug, Clone)]
pub struct BlockNode {
    pub children: Vec<StatementNode>,
}

#[derive(Debug, Clone)]
pub struct IdentifierNode {
    pub name: String,
    pub i_type: Types,
    pub value: Option<Box<Value>>,
}

struct ParseError;

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

    pub fn from(val: Value) -> Option<Self> {
        let i_type = {
            match &val {
                Value::Ident(node) => node.i_type.clone(),
                Value::Lit(lit) => match lit {
                    Literal::Num(_) => Types::Number,
                    Literal::Bool(_) => Types::Bool,
                    Literal::Text(_) => Types::String,
                },
                Value::Expr(expr) => match expr {
                    Expression::Num(_) => Types::Number,
                    Expression::Bool(_) => Types::Bool,
                },
                Value::Nothing => {
                    return None;
                }
                Value::Func(_) => Types::Function,
                Value::Call(node) => node.func.ret.clone(),
            }
        };
        Some(IdentifierNode {
            i_type,
            value: Some(Box::new(val.clone())),
            name: String::from(""),
        })
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Lit(Literal),
    Expr(Expression),
    Ident(IdentifierNode),
    Func(Function),
    Call(CallNode),
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
pub struct Function {
    pub name: String,
    pub params: Vec<IdentifierNode>,
    pub ret: Types,
    pub body: BlockNode,
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

#[derive(Debug, Clone)]
pub struct AssignNode {
    pub ident: IdentifierNode,
}

#[derive(Debug, Clone)]
pub struct DecAssignNode {
    pub ident: IdentifierNode,
    pub i_type: Types,
}

#[derive(Debug, Clone)]
pub struct DeclareNode {
    pub ident: IdentifierNode,
    pub i_type: Types,
}

#[derive(Debug, Clone)]
pub struct CallNode {
    pub func: Function,
    pub params: Vec<IdentifierNode>,
}

#[derive(Debug, Clone)]
pub struct ConditionalNode {
    pub condition: Bool,
    pub body: BlockNode,
    pub i_else: Option<BlockNode>,
}

#[derive(Debug, Clone)]
pub enum Bool {
    Lit(BoolLiteral),
    Expr(BoolExpr),
    Ident(IdentifierNode),
    Call(CallNode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoolLiteral {
    True,
    False,
}

#[derive(Debug, Clone)]
pub struct BoolExpr {
    pub left: Box<Value>,
    pub operator: BoolOps,
    pub right: Box<Value>,
}

#[derive(Debug, Clone)]
pub enum BoolOps {
    EQ,
    Greater,
    Lesser,
}

#[derive(Debug, Clone)]
pub struct ReturnNode {
    pub value: Value,
}

#[derive(Debug)]
pub struct NumExpressionNode {
    pub value: i32,
}

#[derive(Debug, Clone)]
pub struct TextLit {
    pub value: String,
}

pub fn print_tree(node: &StatementNode, indent: usize) {
    let indentation = " ".repeat(indent * 3); // 2 spaces per indentation level
    match node {
        StatementNode::Assign(node) => println!("{}Assign: {:?}", indentation, node),
        StatementNode::Declare(node) => println!("{}Declare: {:?}", indentation, node),
        StatementNode::DeclareAssign(node) => match node.ident.i_type {
            Types::Function => {
                let Some(inner) = &node.ident.value else {
                    todo!()
                };
                match inner.as_ref() {
                    Value::Func(funcnode) => {
                        let children = &funcnode.body.children;
                        println!(
                            "Declare function {} with params {:?}",
                            node.ident.name, funcnode.params
                        );
                        print_program(children, indent + 1);
                    }
                    _ => {}
                }
            }
            _ => {
                println!("{}DeclareAssign: {:?}", indentation, node)
            }
        },
        StatementNode::Call(node) => println!("{}Call: {:?}", indentation, node),
        StatementNode::Conditional(node) => {
            println!("{}Conditional: {:?}", indentation, node);
            print_program(&node.body.children, indent + 1);
        }
        StatementNode::Return(node) => println!("{}Return: {:?}", indentation, node),
        StatementNode::Inline(inline) => println!("{}Inline: {:?}", indentation, inline),
    }
}

pub fn print_program(program: &Vec<StatementNode>, indent: usize) {
    for child in program {
        print_tree(child, indent);
    }
}
