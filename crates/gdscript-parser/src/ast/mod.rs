mod decl;
mod expr;
mod stmt;

pub use decl::*;
pub use expr::*;
pub use stmt::*;

#[derive(PartialEq, Clone, Debug)]
pub enum Line<'a> {
    Stmt(Stmt<'a>),
    Decl(Decl<'a>),
    Expr(Expr<'a>),
    Comment(Comment<'a>),
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
