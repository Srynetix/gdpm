use nom::{
    branch::alt,
    character::complete::{char, line_ending},
    combinator::{all_consuming, map, opt},
    error::context,
    multi::{many0, many1},
    sequence::{pair, preceded, terminated},
};
use nom_tracable::tracable_parser;

use crate::ast::{Block, Line};
use crate::types::{Res, Span};

mod base;
mod decl;
mod expr;
mod stmt;

pub use base::*;
pub use decl::*;
pub use expr::*;
pub use stmt::*;

#[tracable_parser]
pub fn parse_indented_block(i: Span, indent: usize) -> Res<Block> {
    context("indented_block", |i| {
        preceded(pair(opt(parse_comment), many1(line_ending)), |i| {
            more_indent(i, indent)
        })(i)
        .and_then(|(i, indent)| parse_block(i, indent))
    })(i)
}

#[tracable_parser]
pub fn parse_line(i: Span, indent: usize) -> Res<Line> {
    let parse = terminated(
        alt((
            map(|i| parse_decl(i, indent), Line::Decl),
            map(|i| parse_stmt(i, indent), Line::Stmt),
            map(parse_expr, Line::Expr),
            map(parse_comment, Line::Comment),
        )),
        many0(alt((map(parse_comment, |_| ()), map(char(';'), |_| ())))),
    );
    let parse = preceded(|i| same_indent(i, indent), parse);

    context("line", parse)(i)
}

#[tracable_parser]
pub fn parse_block(i: Span, indent: usize) -> Res<Block> {
    context("block", |i| {
        let mut lines = vec![];
        let mut this_i = i;
        loop {
            let (i, _) = parse_many_empty_lines(this_i)?;
            this_i = i;

            // Line
            match parse_line(this_i, indent) {
                Ok((i, line)) => {
                    lines.push(line);
                    this_i = i;
                }
                Err(e) => match e {
                    nom::Err::Error(_) => break,
                    e => return Err(e),
                },
            }
        }

        Ok((this_i, Block(lines)))
    })(i)
}

#[tracable_parser]
pub fn parse_file(i: Span) -> Res<Block> {
    let parse = all_consuming(terminated(move |i| parse_block(i, 0), many0(line_ending)));

    context("file", parse)(i)
}
