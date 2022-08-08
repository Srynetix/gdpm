#[derive(PartialEq, Clone, Debug)]
pub enum Value<'a> {
    Null(Null),
    Boolean(Boolean),
    Int(Int),
    Float(Float),
    String(GdString<'a>),
    NodePath(NodePath<'a>),
    Array(Array<'a>),
    Object(Object<'a>),
    Ident(Ident<'a>),
    FunctionCall(FunctionCall<'a>),
}

impl<'a> Value<'a> {
    pub fn ident(ident: &'a str) -> Self {
        Self::Ident(Ident(ident))
    }

    pub fn int(int_v: i64) -> Self {
        Self::Int(Int(int_v))
    }

    pub fn float(float_v: f64) -> Self {
        Self::Float(Float(float_v))
    }

    pub fn string(str_v: &'a str) -> Self {
        Self::String(GdString(str_v))
    }

    pub fn array(arr_v: Vec<Expr<'a>>) -> Self {
        Self::Array(Array(arr_v))
    }

    pub fn object(pairs: Vec<Pair<'a>>) -> Self {
        Self::Object(Object(pairs))
    }

    pub fn func_call(func_call: FunctionCall<'a>) -> Self {
        Self::FunctionCall(func_call)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Null;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Boolean(pub bool);

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Int(pub i64);

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Float(pub f64);

#[derive(PartialEq, Clone, Debug)]
pub struct Ident<'a>(pub &'a str);

#[derive(PartialEq, Clone, Debug)]
pub struct GdString<'a>(pub &'a str);

#[derive(PartialEq, Clone, Debug)]
pub struct NodePath<'a>(pub &'a str);

#[derive(PartialEq, Clone, Debug)]
pub struct Array<'a>(pub Vec<Expr<'a>>);

#[derive(PartialEq, Clone, Debug)]
pub struct Object<'a>(pub Vec<Pair<'a>>);

#[derive(PartialEq, Clone, Debug)]
pub struct Comment<'a>(pub &'a str);

#[derive(PartialEq, Clone, Debug)]
pub struct Pair<'a>(pub Expr<'a>, pub Expr<'a>);

#[derive(PartialEq, Clone, Debug)]
pub struct FunctionCall<'a> {
    pub name: &'a str,
    pub args: Vec<Expr<'a>>,
}

impl<'a> FunctionCall<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name, args: vec![] }
    }

    pub fn with_arg(mut self, arg: Expr<'a>) -> Self {
        self.args.push(arg);
        self
    }

    pub fn with_args(mut self, args: Vec<Expr<'a>>) -> Self {
        self.args = args;
        self
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct DottedIdent<'a>(pub &'a str);

#[derive(PartialEq, Clone, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BinAnd,
    BinOr,
    BinXor,
    Attr,
    Index,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    Is,
    In,
    As,
}

#[derive(PartialEq, Clone, Debug)]
pub enum UnOp {
    Plus,
    Minus,
    Not,
}

#[derive(PartialEq, Clone, Debug)]
pub struct BinExpr<'a> {
    pub a: Expr<'a>,
    pub b: Expr<'a>,
    pub op: BinOp,
}

#[derive(PartialEq, Clone, Debug)]
pub struct UnExpr<'a> {
    pub a: Expr<'a>,
    pub op: UnOp,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Expr<'a> {
    Value(Value<'a>),
    Bin(Box<BinExpr<'a>>),
    Un(Box<UnExpr<'a>>),
}

impl<'a> Expr<'a> {
    pub fn bin(a: Self, op: BinOp, b: Self) -> Self {
        Self::Bin(Box::new(BinExpr { a, b, op }))
    }

    pub fn un(op: UnOp, a: Self) -> Self {
        Self::Un(Box::new(UnExpr { a, op }))
    }

    pub fn value(v: Value<'a>) -> Self {
        Self::Value(v)
    }
}
