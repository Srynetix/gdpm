use super::{expr::Expr, Block};

#[derive(PartialEq, Clone, Debug)]
pub struct Pass;

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
    pub source: Expr<'a>,
    pub op: AssignOp,
    pub value: Expr<'a>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct ReturnStmt<'a>(pub Expr<'a>);

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
pub struct ForStmt<'a>(pub Condition<'a>);

#[derive(PartialEq, Clone, Debug)]
pub struct MatchStmt<'a> {
    pub expr: Expr<'a>,
    pub cases: Vec<MatchCaseStmt<'a>>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct MatchCaseStmt<'a>(pub Condition<'a>);

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
pub enum Stmt<'a> {
    If(IfStmt<'a>),
    While(WhileStmt<'a>),
    For(ForStmt<'a>),
    Match(MatchStmt<'a>),
    Assign(AssignStmt<'a>),
    Return(ReturnStmt<'a>),
    Pass(Pass),
}
