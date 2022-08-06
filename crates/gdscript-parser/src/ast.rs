#[derive(PartialEq, Clone, Debug)]
pub enum Value<'a> {
    Null(Null),
    Boolean(Boolean),
    Int(Int),
    Float(Float),
    Ident(Ident<'a>),
    String(GdString<'a>),
    NodePath(NodePath<'a>),
    Array(Array<'a>),
    Object(Object<'a>),
    FuncCall(FunctionCall<'a>),
    AttrExpr(AttrExpr<'a>),
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
pub struct Array<'a>(pub Vec<Value<'a>>);

#[derive(PartialEq, Clone, Debug)]
pub struct Object<'a>(pub Vec<Pair<'a>>);

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
    pub r#type: Option<VarType<'a>>,
    pub value: Option<Expr<'a>>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Comment<'a>(pub &'a str);

#[derive(PartialEq, Clone, Debug)]
pub struct Pair<'a>(pub Value<'a>, pub Value<'a>);

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
pub struct FunctionCall<'a> {
    pub name: &'a str,
    pub args: Vec<Expr<'a>>,
}

impl<'a> FunctionCall<'a> {
    pub fn new(name: &'a str, args: Vec<Expr<'a>>) -> Self {
        Self { name, args }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum AttrNode<'a> {
    Name(&'a str),
    Index(usize),
    FuncCall(FunctionCall<'a>),
}

#[derive(PartialEq, Clone, Debug)]
pub struct AttrExpr<'a>(pub Vec<AttrNode<'a>>);

#[derive(PartialEq, Clone, Debug)]
pub struct DottedIdent<'a>(pub &'a str);
pub type VarType<'a> = DottedIdent<'a>;

#[derive(PartialEq, Clone, Debug)]
pub enum Decl<'a> {
    Var(VarDecl<'a>),
    Extends(ExtendsDecl<'a>),
    ClassName(ClassNameDecl<'a>),
    Function(FunctionDecl<'a>),
    Class(ClassDecl<'a>),
    Signal(SignalDecl<'a>),
}

#[derive(PartialEq, Clone, Debug)]
pub enum LineFragment<'a> {
    Stmt(Stmt<'a>),
    Decl(Decl<'a>),
    Expr(Expr<'a>),
    Comment(Comment<'a>),
}

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
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    Is,
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
pub struct Line<'a>(pub Vec<LineFragment<'a>>);

#[derive(PartialEq, Clone, Debug)]
pub struct Block<'a>(pub Vec<Line<'a>>);

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

#[derive(PartialEq, Clone, Debug)]
pub struct Pass;

#[derive(PartialEq, Clone, Debug)]
pub enum Stmt<'a> {
    If(IfStmt<'a>),
    While(WhileStmt<'a>),
    For(ForStmt<'a>),
    Match(MatchStmt<'a>),
    Pass(Pass),
}

#[derive(PartialEq, Clone, Debug)]
pub enum AssignOp {}

#[derive(PartialEq, Clone, Debug)]
pub struct AssignStmt<'a> {
    pub attr: AttrExpr<'a>,
    pub op: AssignOp,
    pub value: Expr<'a>,
}

///////////////
// CONDITIONALS

#[derive(PartialEq, Clone, Debug)]
pub struct IfStmt<'a> {
    pub if_branch: Condition<'a>,
    pub elif_branches: Vec<ElifStmt<'a>>,
    pub else_branch: Option<ElseStmt<'a>>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ElifStmt<'a>(pub Condition<'a>);

#[derive(PartialEq, Clone, Debug)]
pub struct ElseStmt<'a>(pub Block<'a>);

#[derive(PartialEq, Clone, Debug)]
pub struct WhileStmt<'a>(pub Condition<'a>);

#[derive(PartialEq, Clone, Debug)]
pub struct Condition<'a> {
    pub expr: Expr<'a>,
    pub block: Block<'a>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ForStmt<'a> {
    pub expr: Expr<'a>,
    pub in_expr: Expr<'a>,
    pub block: Block<'a>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct MatchStmt<'a> {
    pub expr: Expr<'a>,
    pub cases: Vec<MatchCaseStmt<'a>>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct MatchCaseStmt<'a>(pub Condition<'a>);

/////////////
// FUNCTIONS

#[derive(PartialEq, Clone, Debug)]
pub struct FunctionArg<'a> {
    pub name: Ident<'a>,
    pub r#type: Option<VarType<'a>>,
    pub default: Option<Expr<'a>>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct FunctionDecl<'a> {
    pub name: Ident<'a>,
    pub args: Vec<FunctionArg<'a>>,
    pub return_type: Option<VarType<'a>>,
    pub block: Block<'a>,
}

//////////
// CLASSES

#[derive(PartialEq, Clone, Debug)]
pub struct ClassDecl<'a> {
    pub name: Ident<'a>,
    pub block: Block<'a>,
}
