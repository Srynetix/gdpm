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
    AttrExpr(AttrExpr<'a>),
}

impl<'a> Value<'a> {
    pub fn ident(ident: &'a str) -> Self {
        Self::AttrExpr(AttrExpr(vec![AttrNode::Name(ident)]))
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

    pub fn attr_expr(expr: AttrExpr<'a>) -> Self {
        Self::AttrExpr(expr)
    }

    pub fn object(pairs: Vec<Pair<'a>>) -> Self {
        Self::Object(Object(pairs))
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
    pub r#type: Option<VarType<'a>>,
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
pub struct Comment<'a>(pub &'a str);

#[derive(PartialEq, Clone, Debug)]
pub struct Pair<'a>(pub Value<'a>, pub Expr<'a>);

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
pub enum AttrNode<'a> {
    Name(&'a str),
    String(&'a str),
    Index(Expr<'a>),
    Parens(Expr<'a>),
    FuncCall(FunctionCall<'a>),
}

#[derive(PartialEq, Clone, Debug)]
pub struct AttrExpr<'a>(pub Vec<AttrNode<'a>>);

impl<'a> AttrExpr<'a> {
    pub fn new() -> Self {
        AttrExpr(vec![])
    }

    pub fn with_string(mut self, value: &'a str) -> Self {
        self.0.push(AttrNode::String(value));
        self
    }

    pub fn with_name(mut self, value: &'a str) -> Self {
        self.0.push(AttrNode::Name(value));
        self
    }

    pub fn with_index(mut self, value: Expr<'a>) -> Self {
        self.0.push(AttrNode::Index(value));
        self
    }

    pub fn with_parens(mut self, value: Expr<'a>) -> Self {
        self.0.push(AttrNode::Parens(value));
        self
    }

    pub fn with_func_call(mut self, value: FunctionCall<'a>) -> Self {
        self.0.push(AttrNode::FuncCall(value));
        self
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct DottedIdent<'a>(pub &'a str);
pub type VarType<'a> = DottedIdent<'a>;

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
pub struct Line<'a>(pub Vec<LineFragment<'a>>);

impl<'a> Line<'a> {
    pub fn new_fragment(fragment: LineFragment<'a>) -> Self {
        Line(vec![fragment])
    }

    pub fn with_fragment(mut self, fragment: LineFragment<'a>) -> Self {
        self.0.push(fragment);
        self
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct Block<'a>(pub Vec<Line<'a>>);

impl<'a> Block<'a> {
    pub fn new_line(line: Line<'a>) -> Self {
        Block(vec![line])
    }

    pub fn with_line(mut self, line: Line<'a>) -> Self {
        self.0.push(line);
        self
    }
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

#[derive(PartialEq, Clone, Debug)]
pub struct Pass;

#[derive(PartialEq, Clone, Debug)]
pub enum Stmt<'a> {
    If(IfStmt<'a>),
    While(WhileStmt<'a>),
    For(ForStmt<'a>),
    Match(MatchStmt<'a>),
    Assign(AssignStmt<'a>),
    Return(ReturnStmt<'a>),
    Pass(Pass),
}

#[derive(PartialEq, Clone, Debug)]
pub enum AssignOp {
    Assign,
    AssignAdd,
    AssignSub,
    AssignMul,
    AssignDiv,
    AssignMod,
}

#[derive(PartialEq, Clone, Debug)]
pub struct AssignStmt<'a> {
    pub attr: AttrExpr<'a>,
    pub op: AssignOp,
    pub value: Expr<'a>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ReturnStmt<'a>(pub Expr<'a>);

///////////////
// CONDITIONALS

#[derive(PartialEq, Clone, Debug)]
pub struct IfStmt<'a> {
    pub if_branch: Condition<'a>,
    pub elif_branches: Vec<ElifStmt<'a>>,
    pub else_branch: Option<ElseStmt<'a>>,
}

impl<'a> IfStmt<'a> {
    pub fn new(expr: Expr<'a>, block: Block<'a>) -> Self {
        Self {
            if_branch: Condition::new(expr, block),
            elif_branches: vec![],
            else_branch: None,
        }
    }

    pub fn with_elif(mut self, expr: Expr<'a>, block: Block<'a>) -> Self {
        self.elif_branches
            .push(ElifStmt(Condition::new(expr, block)));
        self
    }

    pub fn with_else(mut self, block: Block<'a>) -> Self {
        self.else_branch = Some(ElseStmt(block));
        self
    }
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

impl<'a> Condition<'a> {
    pub fn new(expr: Expr<'a>, block: Block<'a>) -> Self {
        Self { expr, block }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct ForStmt<'a>(pub Condition<'a>);

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
    pub return_type: Option<VarType<'a>>,
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

//////////
// CLASSES

#[derive(PartialEq, Clone, Debug)]
pub struct ClassDecl<'a> {
    pub name: Ident<'a>,
    pub block: Block<'a>,
}
