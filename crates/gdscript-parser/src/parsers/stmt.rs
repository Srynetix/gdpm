use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, space0, space1},
    combinator::{map, opt, value},
    error::context,
    multi::many0,
    sequence::{pair, preceded, terminated, tuple},
};
use nom_tracable::tracable_parser;

use crate::ast::{
    AssignOp, AssignStmt, Condition, ElifStmt, ElseStmt, ForStmt, IfStmt, MatchCaseStmt, MatchStmt,
    Pass, ReturnStmt, Stmt, WhileStmt,
};
use crate::types::{Res, Span};

use super::{
    base::{more_indent, parse_many_empty_lines, same_indent, ws},
    expr::parse_expr,
    parse_indented_block,
};

#[tracable_parser]
pub fn parse_stmt(i: Span, indent: usize) -> Res<Stmt> {
    let parse = alt((
        map(|i| parse_if_stmt(i, indent), Stmt::If),
        map(|i| parse_while_stmt(i, indent), Stmt::While),
        map(|i| parse_for_stmt(i, indent), Stmt::For),
        map(|i| parse_match_stmt(i, indent), Stmt::Match),
        map(parse_return_stmt, Stmt::Return),
        map(parse_assign_stmt, Stmt::Assign),
        map(parse_pass, Stmt::Pass),
    ));

    context("stmt", parse)(i)
}

#[tracable_parser]
pub fn parse_pass(i: Span) -> Res<Pass> {
    context("pass", value(Pass, tag("pass")))(i)
}

#[tracable_parser]
pub fn parse_assign_op(i: Span) -> Res<AssignOp> {
    let parse = alt((
        value(AssignOp::Assign, tag("=")),
        value(AssignOp::AssignAdd, tag("+=")),
        value(AssignOp::AssignSub, tag("-=")),
        value(AssignOp::AssignMul, tag("*=")),
        value(AssignOp::AssignDiv, tag("/=")),
        value(AssignOp::AssignMod, tag("%=")),
    ));

    context("assign_op", parse)(i)
}

#[tracable_parser]
pub fn parse_assign_stmt(i: Span) -> Res<AssignStmt> {
    let parse = map(
        tuple((ws(parse_expr), ws(parse_assign_op), ws(parse_expr))),
        |(source, op, value)| AssignStmt { source, op, value },
    );

    context("assign_stmt", parse)(i)
}

#[tracable_parser]
pub fn parse_return_stmt(i: Span) -> Res<ReturnStmt> {
    let parse = map(
        preceded(pair(tag("return"), space1), parse_expr),
        ReturnStmt,
    );

    context("return_stmt", parse)(i)
}

#[tracable_parser]
pub fn parse_if_stmt(i: Span, indent: usize) -> Res<IfStmt> {
    let parse = preceded(tag("if"), |i| parse_condition(i, indent));

    context(
        "if_stmt",
        map(
            tuple((
                parse,
                many0(preceded(
                    |i| same_indent(i, indent),
                    |i| parse_elif_stmt(i, indent),
                )),
                opt(preceded(
                    |i| same_indent(i, indent),
                    |i| parse_else_stmt(i, indent),
                )),
            )),
            |(ifb, elifb, elseb)| IfStmt {
                if_branch: ifb,
                elif_branches: elifb,
                else_branch: elseb,
            },
        ),
    )(i)
}

#[tracable_parser]
pub fn parse_elif_stmt(i: Span, indent: usize) -> Res<ElifStmt> {
    let parse = preceded(tag("elif"), |i| parse_condition(i, indent));

    context("elif_stmt", map(parse, ElifStmt))(i)
}

#[tracable_parser]
pub fn parse_else_stmt(i: Span, indent: usize) -> Res<ElseStmt> {
    let parse = preceded(terminated(tag("else"), ws(char(':'))), |i| {
        parse_indented_block(i, indent)
    });

    context("else_stmt", map(parse, ElseStmt))(i)
}

#[tracable_parser]
pub fn parse_while_stmt(i: Span, indent: usize) -> Res<WhileStmt> {
    let parse = preceded(pair(tag("while"), space1), |i| parse_condition(i, indent));

    context("while_stmt", map(parse, WhileStmt))(i)
}

#[tracable_parser]
pub fn parse_for_stmt(i: Span, indent: usize) -> Res<ForStmt> {
    let parse = preceded(pair(tag("for"), space1), |i| parse_condition(i, indent));

    context("for_stmt", map(parse, ForStmt))(i)
}

#[tracable_parser]
pub fn parse_match_stmt(i: Span, indent: usize) -> Res<MatchStmt> {
    context("match_stmt", |i| {
        let mut cases = vec![];
        let (i, expr) = terminated(
            preceded(pair(tag("match"), space1), parse_expr),
            ws(char(':')),
        )(i)?;
        let (i, _) = line_ending(i)?;
        let (i, indent) = more_indent(i, indent)?;

        let mut this_i = i;
        loop {
            let (i, _) = parse_many_empty_lines(this_i)?;
            this_i = i;

            match same_indent(this_i, indent) {
                Ok((i, _)) => {
                    this_i = i;

                    let (i, c) = parse_match_case_stmt(this_i, indent)?;
                    cases.push(c);
                    this_i = i;
                }
                Err(e) => match e {
                    nom::Err::Error(_) => break,
                    e => return Err(e),
                },
            }
        }

        Ok((this_i, MatchStmt { expr, cases }))
    })(i)
}

#[tracable_parser]
pub fn parse_match_case_stmt(i: Span, indent: usize) -> Res<MatchCaseStmt> {
    context(
        "match_case_stmt",
        map(|i| parse_condition(i, indent), MatchCaseStmt),
    )(i)
}

#[tracable_parser]
pub fn parse_condition(i: Span, indent: usize) -> Res<Condition> {
    let parse_cond = terminated(preceded(space0, parse_expr), ws(char(':')));

    context(
        "condition",
        map(
            pair(parse_cond, |i| parse_indented_block(i, indent)),
            |(e, b)| Condition { expr: e, block: b },
        ),
    )(i)
}
