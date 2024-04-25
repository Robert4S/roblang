use crate::parsing::nodes::*;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::io::Error;

pub struct Generator {
    root: Program,
}

impl Generator {
    pub fn new(prog: Program) -> Self {
        Generator { root: prog }
    }

    fn c_out(&self) -> Option<String> {
        let Some(res) = self.root.c_out() else {
            eprintln!("Generator c_out on call to program c_out");
            return None;
        };
        Some(res)
    }

    fn baseimports() -> String {
        String::from("#include \"robIO.h\"\n")
    }

    pub fn write(&self) -> std::io::Result<Option<()>> {
        let Some(writematerial) = self.c_out() else {
            return Ok(None);
        };
        let mut newfile = fs::File::create("out.c")?;
        newfile.write_all(Self::baseimports().as_bytes());
        newfile.write_all(writematerial.as_bytes())?;
        Ok(Some(()))
    }
}

impl Program {
    pub fn c_out(&self) -> Option<String> {
        let mut res = String::from("");
        for child in &self.children {
            match child {
                StatementNode::DeclareAssign(node) => {
                    let Some(nodeout) = node.c_out() else {
                        eprintln!("Program c_out on call to declare assign c_out");
                        return None;
                    };
                    res.push_str(&nodeout);
                }
                _ => {}
            }
        }
        Some(res)
    }
}

impl ReturnNode {
    pub fn c_out(&self) -> Option<String> {
        let mut res = String::from("");
        match &self.value {
            Value::Ident(node) => {
                res = format!("return {};\n", node.name.clone());
            }
            _ => {}
        }
        Some(res)
    }
}

impl BlockNode {
    pub fn c_out(&self) -> Option<String> {
        let mut res = String::from("");
        for stmt in &self.children {
            match stmt {
                StatementNode::DeclareAssign(node) => {
                    let Some(outstr) = node.c_out() else {
                        eprintln!("Generator blocknode c_out for DecAssignNode failed");
                        return None;
                    };
                    res.push_str(&outstr);
                }
                StatementNode::Return(node) => {
                    let Some(nodeout) = node.c_out() else {
                        eprintln!("Generation failed in program c_out upon return node c_out call");
                        return None;
                    };
                    res.push_str(&nodeout);
                }
                StatementNode::Conditional(node) => {
                    let Some(nodeout) = node.c_out() else {
                        eprintln!("Generation failed in blocknode c_out for CondNode");
                        return None;
                    };
                    res.push_str(&nodeout);
                }
                StatementNode::Call(somecall) => {
                    let Some(mut callstr) = somecall.c_out() else {
                        eprintln!("Declare assign c_out upon call to call c_out");
                        return None;
                    };
                    let fstr = format!("{callstr};",);
                    res.push_str(&fstr);
                }
                StatementNode::Inline(someinline) => {
                    let InlineC(inlc) = someinline;
                    res.push_str(inlc);
                }
                _ => {}
            }
        }
        res.push('\n');
        Some(res)
    }
}

impl DecAssignNode {
    fn c_out(&self) -> Option<String> {
        if let Some(id_value) = self.ident.value.clone() {
            match *id_value {
                Value::Lit(somelit) => match somelit {
                    Literal::Num(number) => {
                        let text = format!("int {} = {};\n", self.ident.name, number.val);
                        return Some(text);
                    }
                    Literal::Text(text) => {
                        let out = format!("char {}[] = \"{}\";\n", self.ident.name, text.value);
                        return Some(out);
                    }
                    Literal::Bool(inner) => {
                        let valtext = {
                            match inner {
                                BoolLiteral::True => "true",
                                BoolLiteral::False => "false",
                            }
                        };
                        let text = format!("bool {} = {};\n", self.ident.name, valtext);
                        return Some(text);
                    }
                },
                Value::Ident(someident) => match someident.i_type {
                    Types::Number => {
                        let out = format!("int {} = {};\n", self.ident.name, someident.name);
                        return Some(out);
                    }
                    Types::String => {
                        let out = format!(
                            "char {}[sizeof({})];\n\
                                strcpy({}, {});\n",
                            self.ident.name, someident.name, self.ident.name, someident.name
                        );
                        return Some(out);
                    }
                    Types::Bool => {
                        let out = format!("bool {} = {};\n", self.ident.name, someident.name);
                        return Some(out);
                    }
                    Types::Function => {
                        eprintln!("Functions as values coming soon");
                        return None;
                    }
                    Types::Nothing => {
                        eprintln!("Oops! A nothing type should not have made it this far. Please submit an issue on github.");
                        return None;
                    }
                },
                Value::Func(somefunc) => {
                    let Some(funcstr) = somefunc.c_out() else {
                        eprintln!("Delcare assign c_out upon call of function c_out");
                        return None;
                    };
                    return Some(funcstr);
                }
                Value::Call(somecall) => {
                    let Some(mut callstr) = somecall.c_out() else {
                        eprintln!("Declare assign c_out upon call to call c_out");
                        return None;
                    };
                    let (prefix, suffix) = {
                        match somecall.func.ret {
                            Types::Bool => ("bool", ""),
                            Types::String => ("char", "[]"),
                            Types::Number => ("int", ""),
                            Types::Function => todo!(),
                            Types::Nothing => todo!(),
                        }
                    };
                    let out = format!("{prefix} {}{suffix} = {callstr};\n", self.ident.name);
                    return Some(out);
                }
                Value::Expr(someexpr) => {
                    let Some(exprstr) = someexpr.c_out() else {
                        eprintln!("Could not generate code for expression");
                        return None;
                    };
                    let prefix = match someexpr {
                        Expression::Num(somenum) => "int",
                        Expression::Bool(somebool) => "bool",
                    };
                    return Some(format!("{prefix} {} = {exprstr};\n", self.ident.name));
                }
                _ => {}
            }
        }
        None
    }
}

impl ConditionalNode {
    pub fn c_out(&self) -> Option<String> {
        let upper = {
            match &self.condition {
                Bool::Lit(somelit) => {
                    let cond = {
                        if *somelit == BoolLiteral::True {
                            "true"
                        } else {
                            "false"
                        }
                    };
                    format!("if ({})", cond)
                }
                Bool::Ident(someident) => {
                    format!("if ({})", someident.name)
                }
                Bool::Expr(expr) => {
                    format!("if {}", expr.c_out()?)
                }
                Bool::Call(call) => {
                    format!("if {}", call.c_out()?)
                }
            }
        };
        let body = self.body.c_out()?;
        let els_body = {
            match &self.i_else {
                Some(block) => {
                    let temp_bod = block.c_out()?;
                    format!("else {{\n{} }}\n", temp_bod)
                }
                _ => String::from(""),
            }
        };
        let out = format!("{} {{\n{} }} {}\n", upper, body, els_body);
        Some(out)
    }
}

impl BoolExpr {
    pub fn c_out(&self) -> Option<String> {
        let op = {
            match self.operator {
                BoolOps::EQ => "==",
                BoolOps::Lesser => "<",
                BoolOps::Greater => ">",
            }
        };
        Some(format!(
            "({} {} {})",
            self.left.c_out()?,
            op,
            self.right.c_out()?
        ))
    }
}

impl Value {
    pub fn c_out(&self) -> Option<String> {
        match self {
            Value::Lit(somelit) => Some(somelit.c_out())?,
            Value::Ident(someident) => Some(someident.name.clone()),
            Value::Expr(expr) => Some(expr.c_out())?,
            _ => None,
        }
    }
}

impl Expression {
    pub fn c_out(&self) -> Option<String> {
        match self {
            Expression::Bool(somebool) => Some(somebool.c_out()?),
            Expression::Num(somenum) => Some(somenum.c_out()?),
        }
    }
}

impl NumExpression {
    pub fn c_out(&self) -> Option<String> {
        let left = match *(self.left.clone()) {
            Number::Lit(somelit) => Literal::Num(somelit).c_out()?,
            Number::Exp(somexpr) => somexpr.c_out()?,
            Number::Call(somecall) => somecall.c_out()?,
            Number::Ident(someident) => someident.c_out()?
        };
        let oper = match self.operator {
            Operators::Plus => "+",
            Operators::Minus => "-",
        };
        let right = match *(self.right.clone()) {
            Number::Lit(somelit) => Literal::Num(somelit).c_out()?,
            Number::Exp(somexpr) => somexpr.c_out()?,
            Number::Call(somecall) => somecall.c_out()?,
            Number::Ident(someident) => someident.c_out()?,
        };
        Some(format!("{left} {oper} {right}"))
    }
}

impl Literal {
    pub fn c_out(&self) -> Option<String> {
        match self {
            Literal::Bool(somebool) => {
                if *somebool == BoolLiteral::True {
                    Some(String::from("true"))
                } else {
                    Some(String::from("false"))
                }
            }
            Literal::Num(somenum) => Some(format!("{}", somenum.val)),
            Literal::Text(sometext) => Some(sometext.value.clone()),
        }
    }
}

impl Function {
    pub fn c_out(&self) -> Option<String> {
        let fntype = {
            match self.ret {
                Types::Number => "int",
                Types::String => "char*",
                Types::Bool => "bool",
                _ => {
                    eprintln!("Functions as values coming soon");
                    return None;
                }
            }
        };
        let Some(params) = self.params_c_out() else {
            eprintln!("Params were not parsed correctly");
            return None;
        };
        let Some(body) = self.body.c_out() else {
            eprintln!("Could not emit code for function body");
            return None;
        };
        let out = format!("{} {}({}) {{\n {} }}\n", fntype, self.name, params, body);
        Some(out)
    }

    fn params_c_out(&self) -> Option<String> {
        let mut paramstr = String::new();
        let paramslen = self.params.len();
        for (index, param) in self.params.iter().enumerate() {
            let (prefix, postfix) = {
                match param.i_type {
                    Types::Bool => ("bool", ""),
                    Types::String => ("char", "[]"),
                    Types::Number => ("int", ""),
                    _ => {
                        eprintln!("OOPS");
                        return None;
                    }
                }
            };
            let varname = {
                if index != paramslen - 1 {
                    format!("{prefix} {}{postfix}, ", param.name)
                } else {
                    format!("{prefix} {}{postfix}", param.name)
                }
            };
            paramstr.push_str(&varname);
        }
        Some(paramstr)
    }
}

impl IdentifierNode {
    pub fn arg_c_out(&self) -> Option<String> {
        let valtxt = self.clone().value?.c_out()?;
        Some(format!("{},", valtxt))
    }

    pub fn lit_c_out(&self) -> Option<String> {
        match *(self.value.clone()?) {
            Value::Lit(somelit) => match somelit {
                Literal::Text(sometext) => Some(format!("\"{}\",", sometext.value.clone())),
                _ => None,
            },
            Value::Ident(someident) => Some(someident.arg_c_out()?),
            _ => None,
        }
    }

    pub fn c_out(&self) -> Option<String> {
        Some(format!("{}", self.name))
    }
}

impl CallNode {
    pub fn c_out(&self) -> Option<String> {
        match self.func.name.as_str() {
            "showme" => {
                return Some(self.showme_c_out()?);
            }
            _ => {}
        }
        let mut args = String::new();
        for arg in &self.params {
            args.push_str(&arg.arg_c_out()?);
        }
        args.remove(args.len() - 1);
        Some(format!("{}({args})", self.func.name))
    }

    pub fn showme_c_out(&self) -> Option<String> {
        if self.params.len() == 0 {
            return Some(String::from("printf();\n"));
        } else if self.params.len() == 1 {
            eprintln!("...");
            let mut temp = self.params[0].lit_c_out()?;
            temp.remove(temp.len() - 1);
            return Some(format!("printf({});\n", temp));
        }

        let mut argstr = String::new();
        let mut i = 0;
        for arg in &self.params {
            if i == 0 {
                argstr.push_str(&self.params[0].lit_c_out()?);
                i += 1;
                continue;
            }

            match *(arg.clone().value?) {
                Value::Ident(someident) => {
                    let temp = format!("{},", someident.name.clone());
                    argstr.push_str(temp.as_str());
                }
                _ => {
                    eprintln!("Can only format identifiers of the same type.");
                    return None;
                }
            }
        }
        let base = {
            match self.params[1].i_type {
                Types::Number => format!("showme_int({} INT_MAX);\n", argstr),
                Types::String => format!("showme_text({} NULL);\n", argstr),
                _ => {
                    eprintln!("\n\n\nShowme is still experimental, and can only be used with Text or Number types. It is better to use an inlined printf() call.\n\n\n");
                    return None;
                }
            }
        };
        Some(base)
    }
}
