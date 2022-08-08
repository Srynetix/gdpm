use nom::{
    branch::alt,
    bytes::complete::{escaped, tag},
    character::complete::{alpha1, alphanumeric1, char, digit1, none_of, one_of, space0},
    combinator::{map, map_res, opt, peek, recognize, value},
    error::context,
    multi::{many0, many0_count, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};
use nom_tracable::tracable_parser;

use crate::ast::{
    Array, BinExpr, BinOp, Boolean, DottedIdent, Expr, Float, FunctionCall, GdString, Ident, Int,
    NodePath, Null, Object, Pair, UnExpr, UnOp, Value,
};
use crate::types::{Res, Span};

use super::base::{ms0noc, wslnoc};

#[tracable_parser]
pub fn parse_ident(i: Span) -> Res<Ident> {
    context(
        "ident",
        map(
            recognize(pair(
                alt((alpha1, tag("_"))),
                many0_count(alt((alphanumeric1, tag("_")))),
            )),
            |r: Span| Ident(*r),
        ),
    )(i)
}

#[tracable_parser]
pub fn parse_int(i: Span) -> Res<Int> {
    context(
        "int",
        map(
            alt((
                preceded(
                    tag("0x"),
                    map_res(alphanumeric1, |s: Span| i64::from_str_radix(*s, 16)),
                ),
                map_res(digit1, |s: Span| s.parse::<i64>()),
            )),
            Int,
        ),
    )(i)
}

#[tracable_parser]
pub fn parse_null(i: Span) -> Res<Null> {
    context("null", value(Null, tag("null")))(i)
}

#[tracable_parser]
pub fn parse_float(i: Span) -> Res<Float> {
    let parse = map_res(recognize(tuple((digit1, char('.'), digit1))), |s: Span| {
        s.parse::<f64>().map(Float)
    });
    context("float", parse)(i)
}

#[tracable_parser]
pub fn parse_boolean(i: Span) -> Res<Boolean> {
    let parse_true = value(Boolean(true), alt((tag("true"), tag("True"))));
    let parse_false = value(Boolean(false), alt((tag("false"), tag("False"))));
    context("boolean", alt((parse_true, parse_false)))(i)
}

#[tracable_parser]
pub fn parse_string(i: Span) -> Res<GdString> {
    let parse = map(alt((parse_quoted_single, parse_quoted_double)), |v| {
        GdString(*v)
    });

    context("string", parse)(i)
}

#[tracable_parser]
pub fn parse_dotted_ident(i: Span) -> Res<DottedIdent> {
    context(
        "dotted_ident",
        map(
            recognize(separated_list1(char('.'), parse_ident)),
            |x: Span| DottedIdent(*x),
        ),
    )(i)
}

fn parse_quoted_single(i: Span) -> Res<Span> {
    context("quoted_single", |i| {
        let esc = escaped(none_of("\\\'"), '\\', one_of("\'n\\"));
        let esc_or_empty = alt((esc, tag("")));
        let res = delimited(char('\''), esc_or_empty, char('\''))(i)?;
        Ok(res)
    })(i)
}

fn parse_quoted_double(i: Span) -> Res<Span> {
    context("quoted_double", |i| {
        let esc = escaped(none_of("\\\""), '\\', one_of("\"n\\"));
        let esc_or_empty = alt((esc, tag("")));
        let res = delimited(char('"'), esc_or_empty, char('"'))(i)?;
        Ok(res)
    })(i)
}

#[tracable_parser]
pub fn parse_node_path(i: Span) -> Res<NodePath> {
    let parse_ident_path = recognize(separated_list1(char('/'), parse_ident));
    let parse_string_path = recognize(parse_string);
    let parse = map(
        recognize(pair(char('$'), alt((parse_ident_path, parse_string_path)))),
        |v| NodePath(*v),
    );

    context("node_path", parse)(i)
}

#[tracable_parser]
pub fn parse_array(i: Span) -> Res<Array> {
    let inner_parse = map(
        terminated(
            separated_list0(wslnoc(char(',')), parse_expr),
            opt(wslnoc(char(','))),
        ),
        Array,
    );
    let parse = preceded(
        pair(char('['), ms0noc),
        terminated(inner_parse, pair(ms0noc, char(']'))),
    );

    context("array", parse)(i)
}

#[tracable_parser]
pub fn parse_object(i: Span) -> Res<Object> {
    let pairs = map(
        terminated(
            separated_list0(wslnoc(char(',')), parse_pair),
            opt(wslnoc(char(','))),
        ),
        Object,
    );
    let parse = preceded(
        pair(char('{'), ms0noc),
        terminated(pairs, pair(ms0noc, char('}'))),
    );

    context("object", parse)(i)
}

#[tracable_parser]
pub fn parse_pair(i: Span) -> Res<Pair> {
    context(
        "pair",
        map(
            separated_pair(wslnoc(parse_expr), char(':'), wslnoc(parse_expr)),
            |(k, v)| Pair(k, v),
        ),
    )(i)
}

#[tracable_parser]
pub fn parse_function_call(i: Span) -> Res<FunctionCall> {
    let parse = map(
        pair(
            parse_ident,
            delimited(
                pair(char('('), ms0noc),
                opt(separated_list0(wslnoc(char(',')), parse_expr)),
                pair(ms0noc, char(')')),
            ),
        ),
        |(name, args)| FunctionCall::new(name.0).with_args(args.unwrap_or_default()),
    );

    context("function_call", parse)(i)
}

#[tracable_parser]
pub fn parse_expr(i: Span) -> Res<Expr> {
    context("expr", parse_expr_logic_expr)(i)
}

#[tracable_parser]
pub fn parse_expr_logic_expr(i: Span) -> Res<Expr> {
    context("expr_logic_expr", |i| {
        let (i, num1) = parse_expr_math_expr(i)?;
        let (i, exprs) = many0(tuple((
            map(
                wslnoc(alt((
                    alt((
                        tag("."),
                        tag("&&"),
                        tag("||"),
                        tag(">="),
                        tag("<="),
                        tag(">"),
                        tag("<"),
                        tag("=="),
                        tag("!="),
                    )),
                    terminated(
                        alt((tag("and"), tag("or"), tag("is"), tag("in"), tag("as"))),
                        peek(one_of("([\n\t ")),
                    ),
                ))),
                |k: Span| *k,
            ),
            parse_expr_logic_expr,
        )))(i)?;

        Ok((i, parse_expr_rec(num1, exprs)))
    })(i)
}

#[tracable_parser]
pub fn parse_expr_math_expr(i: Span) -> Res<Expr> {
    context("expr_math_expr", |i| {
        let (i, num1) = parse_expr_term(i)?;
        let (i, exprs) = many0(tuple((
            map(wslnoc(alt((tag("+"), tag("-")))), |k: Span| *k),
            parse_expr_math_expr,
        )))(i)?;

        Ok((i, parse_expr_rec(num1, exprs)))
    })(i)
}

pub fn parse_expr_rec<'a>(a: Expr<'a>, rem: Vec<(&str, Expr<'a>)>) -> Expr<'a> {
    rem.into_iter()
        .fold(a, |acc, val| parse_expr_binop(val, acc))
}

#[tracable_parser]
pub fn parse_expr_un(i: Span) -> Res<Expr> {
    context("expr_un", |i| {
        let (i, op) = opt(map(
            wslnoc(alt((tag("-"), tag("+"), tag("!")))),
            |k: Span| *k,
        ))(i)?;
        let (i, num) = parse_expr_index(i)?;

        match op {
            Some(op) => Ok((i, parse_expr_unop(op, num))),
            None => Ok((i, num)),
        }
    })(i)
}

#[tracable_parser]
pub fn parse_expr_term(i: Span) -> Res<Expr> {
    context("expr_term", |i| {
        let (i, num1) = parse_expr_un(i)?;
        let (i, exprs) = many0(tuple((
            map(
                wslnoc(alt((
                    tag("*"),
                    tag("/"),
                    tag("%"),
                    tag("|"),
                    tag("&"),
                    tag("^"),
                ))),
                |k: Span| *k,
            ),
            parse_expr_term,
        )))(i)?;

        Ok((i, parse_expr_rec(num1, exprs)))
    })(i)
}

#[tracable_parser]
pub fn parse_expr_index(i: Span) -> Res<Expr> {
    context("expr_index", |i| {
        let (i, num1) = parse_expr_operation(i)?;
        let (i, exprs) = many0(delimited(
            pair(char('['), ms0noc),
            parse_expr_math_expr,
            pair(ms0noc, char(']')),
        ))(i)?;

        let final_expr = exprs
            .into_iter()
            .fold(num1, |acc, val| Expr::bin(acc, BinOp::Index, val));

        Ok((i, final_expr))
    })(i)
}

#[tracable_parser]
pub fn parse_expr_operation(i: Span) -> Res<Expr> {
    context("expr_operation", alt((parse_expr_parens, parse_expr_value)))(i)
}

#[tracable_parser]
pub fn parse_expr_value(i: Span) -> Res<Expr> {
    context(
        "expr_value",
        map(delimited(space0, parse_value, space0), Expr::Value),
    )(i)
}

#[tracable_parser]
pub fn parse_expr_parens(i: Span) -> Res<Expr> {
    context(
        "expr_parens",
        delimited(space0, delimited(char('('), parse_expr, char(')')), space0),
    )(i)
}

pub fn parse_expr_unop<'a>(op: &str, expr: Expr<'a>) -> Expr<'a> {
    let new_expr = |op, a| Expr::Un(Box::new(UnExpr { a, op }));

    match op {
        "+" => new_expr(UnOp::Plus, expr),
        "-" => new_expr(UnOp::Minus, expr),
        "!" => new_expr(UnOp::Not, expr),
        _ => unreachable!(),
    }
}

pub fn parse_expr_binop<'a>(tup: (&str, Expr<'a>), expr1: Expr<'a>) -> Expr<'a> {
    let new_expr = |a, b, op| Expr::Bin(Box::new(BinExpr { a, b, op }));

    let (op, expr2) = tup;
    match op {
        "." => new_expr(expr1, expr2, BinOp::Attr),
        "+" => new_expr(expr1, expr2, BinOp::Add),
        "-" => new_expr(expr1, expr2, BinOp::Sub),
        "*" => new_expr(expr1, expr2, BinOp::Mul),
        "/" => new_expr(expr1, expr2, BinOp::Div),
        "%" => new_expr(expr1, expr2, BinOp::Mod),
        "&" => new_expr(expr1, expr2, BinOp::BinAnd),
        "|" => new_expr(expr1, expr2, BinOp::BinOr),
        "^" => new_expr(expr1, expr2, BinOp::BinXor),
        "&&" | "and" => new_expr(expr1, expr2, BinOp::And),
        "||" | "or" => new_expr(expr1, expr2, BinOp::Or),
        "==" => new_expr(expr1, expr2, BinOp::Eq),
        "!=" => new_expr(expr1, expr2, BinOp::Neq),
        ">" => new_expr(expr1, expr2, BinOp::Gt),
        "<" => new_expr(expr1, expr2, BinOp::Lt),
        ">=" => new_expr(expr1, expr2, BinOp::Gte),
        "<=" => new_expr(expr1, expr2, BinOp::Lte),
        "in" => new_expr(expr1, expr2, BinOp::In),
        "is" => new_expr(expr1, expr2, BinOp::Is),
        "as" => new_expr(expr1, expr2, BinOp::As),
        _ => unreachable!(),
    }
}

#[tracable_parser]
pub fn parse_value(i: Span) -> Res<Value> {
    let parse = alt((
        map(parse_null, Value::Null),
        map(parse_boolean, Value::Boolean),
        map(parse_array, Value::Array),
        map(parse_object, Value::Object),
        map(parse_node_path, Value::NodePath),
        map(parse_function_call, Value::FunctionCall),
        map(parse_ident, Value::Ident),
        map(parse_string, Value::String),
        map(parse_float, Value::Float),
        map(parse_int, Value::Int),
    ));

    context("value", parse)(i)
}
