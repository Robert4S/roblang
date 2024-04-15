use crate::parsing::nodes::*;

impl BlockNode {
    pub fn c_out(&self) -> Option<String> {
        todo!()
    }
}

impl DecAssignNode {
    fn c_out(&self) -> Option<String> {
        if let Some(id_value) = self.ident.value.clone() {
            match *id_value {
                Value::Lit(somelit) => {
                    match somelit {
                        Literal::Num(number) => {
                            let text = format!("int {} = {};\n", self.ident.name, number.val);

                            return Some(text);
                        }
                        Literal::Text(text) => {
                            let out = format!("char {}[] = {};\n", self.ident.name, text.value);
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
                    }
                }
                Value::Ident(someident) => {
                    match someident.i_type {
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
                            let out = format!("bool {} = {}", self.ident.name, someident.name);
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
                    }
                }
                Value::Func(somefunc) => {
                    let fntype = {
                        match somefunc.ret {
                            Types::Number => "int",
                            Types::String => "char[]",
                            Types::Bool => "bool",
                            _ => {
                                eprintln!("Functions as values coming soon");
                                return None;
                            }
                        }
                    };
                    let Some(body) = somefunc.c_out() else { todo!() };
                    let out = format!("{} {} {{  {} }} ", fntype, self.ident.name, body);
                    return Some(out);
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
                    format!("if {}", cond)
                },
                Bool::Ident(someident) => {
                    format!("if {}", someident.name)
                },
                Bool::Expr(_) => {
                    eprintln!("A bool expression should not have made it this far.");
                    return None;
                }
            }
        };
        let Some(body) = self.body.c_out() else {todo!()};
        let out = format!("{} {{ {} }}", upper, body);
        Some(out)
    }
}

impl Function {
    pub fn c_out(&self) -> Option<String> {
        let fntype = {
            match self.ret {
                Types::Number => "int",
                Types::String => "char[]",
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
        let out = format!("{} {}({}) {{ {} }}", fntype, self.name, params, body);
        Some(out)
    }
    fn params_c_out(&self) -> Option<String> {
        todo!()
    }
}