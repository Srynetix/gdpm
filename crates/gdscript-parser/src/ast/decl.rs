use super::{
    expr::{DottedIdent, Expr, Ident, Value},
    Block,
};

#[derive(PartialEq, Clone, Debug)]
pub enum VarModifier {
    OnReady,
    Export,
}

#[derive(PartialEq, Clone, Debug)]
pub struct VarDecl<'a> {
    pub modifier: Option<VarModifier>,
    pub name: &'a str,
    pub infer: bool,
    pub r#type: Option<DottedIdent<'a>>,
    pub value: Option<Expr<'a>>,
    pub set_func: Option<&'a str>,
    pub get_func: Option<&'a str>,
}

impl<'a> VarDecl<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            modifier: None,
            name,
            infer: false,
            r#type: None,
            value: None,
            set_func: None,
            get_func: None,
        }
    }

    pub fn with_set_func(mut self, func: &'a str) -> Self {
        self.set_func = Some(func);
        self
    }

    pub fn with_get_func(mut self, func: &'a str) -> Self {
        self.get_func = Some(func);
        self
    }

    pub fn with_infer(mut self, value: bool) -> Self {
        self.infer = value;
        self
    }

    pub fn with_modifier(mut self, modifier: VarModifier) -> Self {
        self.modifier = Some(modifier);
        self
    }

    pub fn with_type(mut self, r#type: &'a str) -> Self {
        self.r#type = Some(DottedIdent(r#type));
        self
    }

    pub fn with_value(mut self, value: Expr<'a>) -> Self {
        self.value = Some(value);
        self
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ConstDecl<'a> {
    pub name: &'a str,
    pub infer: bool,
    pub r#type: Option<DottedIdent<'a>>,
    pub value: Expr<'a>,
}

impl<'a> ConstDecl<'a> {
    pub fn new(name: &'a str, value: Expr<'a>) -> Self {
        Self {
            name,
            infer: false,
            r#type: None,
            value,
        }
    }

    pub fn with_infer(mut self, value: bool) -> Self {
        self.infer = value;
        self
    }

    pub fn with_type(mut self, r#type: &'a str) -> Self {
        self.r#type = Some(DottedIdent(r#type));
        self
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct EnumDecl<'a> {
    pub name: &'a str,
    pub variants: Vec<EnumVariant<'a>>,
}

impl<'a> EnumDecl<'a> {
    pub fn new(name: &'a str, variants: Vec<EnumVariant<'a>>) -> Self {
        Self { name, variants }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct EnumVariant<'a> {
    pub name: &'a str,
    pub value: Option<Value<'a>>,
}

impl<'a> EnumVariant<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name, value: None }
    }

    pub fn with_value(mut self, value: Value<'a>) -> Self {
        self.value = Some(value);
        self
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct FunctionArg<'a> {
    pub name: Ident<'a>,
    pub r#type: Option<DottedIdent<'a>>,
    pub default: Option<Expr<'a>>,
}

impl<'a> FunctionArg<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name: Ident(name),
            r#type: None,
            default: None,
        }
    }

    pub fn new_typed(name: &'a str, r#type: &'a str) -> Self {
        Self {
            name: Ident(name),
            r#type: Some(DottedIdent(r#type)),
            default: None,
        }
    }

    pub fn with_value(mut self, value: Expr<'a>) -> Self {
        self.default = Some(value);
        self
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum FunctionModifier {
    Static,
    Remote,
    Master,
    Puppet,
    RemoteSync,
    MasterSync,
    PuppetSync,
}

#[derive(PartialEq, Clone, Debug)]
pub struct FunctionDecl<'a> {
    pub modifier: Option<FunctionModifier>,
    pub name: Ident<'a>,
    pub args: Vec<FunctionArg<'a>>,
    pub return_type: Option<DottedIdent<'a>>,
    pub block: Block<'a>,
}

impl<'a> FunctionDecl<'a> {
    pub fn new(
        name: &'a str,
        args: Vec<FunctionArg<'a>>,
        return_type: Option<DottedIdent<'a>>,
        block: Block<'a>,
    ) -> Self {
        Self {
            modifier: None,
            name: Ident(name),
            args,
            return_type,
            block,
        }
    }

    pub fn with_modifier(mut self, modifier: FunctionModifier) -> Self {
        self.modifier = Some(modifier);
        self
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ExtendsDecl<'a>(pub &'a str);

#[derive(PartialEq, Clone, Debug)]
pub struct ClassNameDecl<'a>(pub &'a str);

#[derive(PartialEq, Clone, Debug)]
pub struct SignalDecl<'a> {
    pub name: Ident<'a>,
    pub args: Vec<Ident<'a>>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ClassDecl<'a> {
    pub name: Ident<'a>,
    pub block: Block<'a>,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Decl<'a> {
    Var(VarDecl<'a>),
    Const(ConstDecl<'a>),
    Extends(ExtendsDecl<'a>),
    ClassName(ClassNameDecl<'a>),
    Enum(EnumDecl<'a>),
    Function(FunctionDecl<'a>),
    Class(ClassDecl<'a>),
    Signal(SignalDecl<'a>),
}
