use nom::{
    bytes::complete::is_not,
    character::complete::{char, line_ending, multispace0, space0},
    combinator::{map, opt, peek, value},
    error::{context, VerboseError, VerboseErrorKind},
    multi::many0,
    sequence::{delimited, preceded, terminated},
};
use nom_locate::LocatedSpan;
use nom_tracable::{tracable_parser, TracableInfo};

use crate::{
    ast::Comment,
    types::{Res, Span},
};

pub fn new_span(input: &str) -> Span {
    let info = TracableInfo::new();
    LocatedSpan::new_extra(input, info)
}

pub fn wslnoc<'a, F: 'a, O>(inner: F) -> impl FnMut(Span<'a>) -> Res<O>
where
    F: FnMut(Span<'a>) -> Res<O>,
{
    delimited(ms0noc, inner, ms0noc)
}

#[tracable_parser]
pub fn ms0noc(i: Span) -> Res<()> {
    context("ms0noc", |i| {
        let (i, _) = multispace0(i)?;
        match terminated(parse_comment, opt(line_ending))(i) {
            Ok((i, _)) => return ms0noc(i),
            Err(e) => match e {
                nom::Err::Error(_) => (),
                e => return Err(e),
            },
        }

        Ok((i, ()))
    })(i)
}

pub fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(Span<'a>) -> Res<O>
where
    F: FnMut(Span<'a>) -> Res<O>,
{
    delimited(space0, inner, space0)
}

#[tracable_parser]
pub fn parse_comment(i: Span) -> Res<Comment> {
    let parse = map(
        preceded(ws(char('#')), opt(is_not("\n\r"))),
        |s: Option<Span>| Comment(s.map(|x| x.trim()).unwrap_or_default()),
    );
    context("comment", parse)(i)
}

#[tracable_parser]
pub fn parse_empty_line(i: Span) -> Res<()> {
    context("empty_line", terminated(value((), space0), line_ending))(i)
}

#[tracable_parser]
pub fn parse_many_empty_lines(i: Span) -> Res<()> {
    context("many_empty_lines", |i| {
        let mut this_i = i;
        loop {
            match parse_empty_line(this_i) {
                Ok((i, _)) => {
                    this_i = i;
                    continue;
                }
                Err(e) => match e {
                    nom::Err::Error(_) => break,
                    e => return Err(e),
                },
            }
        }
        Ok((this_i, ()))
    })(i)
}

#[tracable_parser]
pub fn more_indent(i: Span, indent: usize) -> Res<usize> {
    context("more_indent", |i| {
        let (s, parsed) = scan_indentation(i)?;
        if parsed > indent {
            Ok((s, parsed))
        } else {
            Err(nom::Err::Error(VerboseError {
                errors: vec![(
                    s,
                    VerboseErrorKind::Context("should be a greater indentation"),
                )],
            }))
        }
    })(i)
}

#[tracable_parser]
pub fn same_indent(i: Span, indent: usize) -> Res<()> {
    context("same_indent", |i| {
        let (s, parsed) = parse_indentation(i)?;
        if parsed == indent {
            Ok((s, ()))
        } else {
            Err(nom::Err::Error(VerboseError {
                errors: vec![(s, VerboseErrorKind::Context("not the same indentation"))],
            }))
        }
    })(i)
}

#[tracable_parser]
pub fn parse_indentation(i: Span) -> Res<usize> {
    context("parse_indentation", |i| {
        let (s, spaces) = many0(char(' '))(i)?;
        let indent_level = spaces.len();
        Ok((s, indent_level))
    })(i)
}

#[tracable_parser]
pub fn scan_indentation(i: Span) -> Res<usize> {
    context("scan_indentation", |i| {
        let (s, spaces) = peek(many0(char(' ')))(i)?;
        let indent_level = spaces.len();
        Ok((s, indent_level))
    })(i)
}
