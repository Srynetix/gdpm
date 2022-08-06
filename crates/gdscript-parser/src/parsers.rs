use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::{
        complete::{
            alpha1, alphanumeric1, char, digit1, line_ending, multispace0, none_of, one_of, space0,
        },
        streaming::space1,
    },
    combinator::{all_consuming, cut, map, map_res, opt, peek, recognize, value},
    error::{context, VerboseError, VerboseErrorKind},
    multi::{many0, many0_count, many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

use crate::{
    ast::{
        Array, AttrExpr, AttrNode, BinExpr, BinOp, Block, Boolean, ClassDecl, ClassNameDecl,
        Comment, Condition, Decl, DottedIdent, Expr, ExtendsDecl, Float, ForStmt, FunctionArg,
        FunctionCall, FunctionDecl, GdString, IfStmt, Int, Line, LineFragment, MatchCaseStmt,
        MatchStmt, NodePath, Null, Object, Pair, Pass, SignalDecl, Stmt, UnExpr, UnOp, Value,
        VarDecl, VarModifier, VarType, WhileStmt,
    },
    debug::{pp_ret, pp_span},
};
use crate::{
    ast::{ElifStmt, ElseStmt, Ident},
    types::{Res, Span},
};

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
pub fn wsl<'a, F: 'a, O>(inner: F) -> impl FnMut(Span<'a>) -> Res<O>
where
    F: FnMut(Span<'a>) -> Res<O>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(Span<'a>) -> Res<O>
where
    F: FnMut(Span<'a>) -> Res<O>,
{
    delimited(space0, inner, space0)
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_ident(i: Span) -> Res<Ident> {
    pp_ret(
        "ident",
        context(
            "ident",
            map(
                recognize(pair(
                    alt((alpha1, tag("_"))),
                    many0_count(alt((alphanumeric1, tag("_")))),
                )),
                |r: Span| Ident(*r),
            ),
        )(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_empty_line(i: Span) -> Res<()> {
    pp_ret(
        "empty_line",
        context(
            "empty_line",
            terminated(value((), space0), peek(line_ending)),
        )(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_int(i: Span) -> Res<Int> {
    pp_ret(
        "int",
        context("int", map(map_res(digit1, |s: Span| s.parse::<i64>()), Int))(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_null(i: Span) -> Res<Null> {
    pp_ret("null", context("null", value(Null, tag("null")))(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_float(i: Span) -> Res<Float> {
    let parse = map_res(recognize(tuple((digit1, char('.'), digit1))), |s: Span| {
        s.parse::<f64>().map(Float)
    });
    pp_ret("float", context("float", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_boolean(i: Span) -> Res<Boolean> {
    let parse_true = value(Boolean(true), alt((tag("true"), tag("True"))));
    let parse_false = value(Boolean(false), alt((tag("false"), tag("False"))));
    pp_ret(
        "boolean",
        context("boolean", alt((parse_true, parse_false)))(i),
    )
}

pub fn parse_string_inner(i: Span) -> Res<Span> {
    escaped(none_of("\\\"'"), '\\', one_of("\"n\\'"))(i)
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_string(i: Span) -> Res<GdString> {
    pp_ret(
        "string",
        context(
            "string",
            map(
                alt((
                    preceded(char('\"'), cut(terminated(parse_string_inner, char('\"')))),
                    preceded(char('\''), cut(terminated(parse_string_inner, char('\'')))),
                )),
                |v| GdString(*v),
            ),
        )(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_node_path(i: Span) -> Res<NodePath> {
    let parse_ident_path = recognize(separated_list1(char('/'), parse_ident));
    let parse_string_path = recognize(parse_string);
    let parse = map(
        recognize(pair(char('$'), alt((parse_ident_path, parse_string_path)))),
        |v| NodePath(*v),
    );

    pp_ret("node_path", context("node_path", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_array(i: Span) -> Res<Array> {
    let inner_parse = map(separated_list0(char(','), wsl(parse_value)), Array);
    let parse = preceded(
        pair(char('['), multispace0),
        terminated(inner_parse, pair(multispace0, char(']'))),
    );

    pp_ret("array", context("array", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_object(i: Span) -> Res<Object> {
    let pairs = map(separated_list0(char(','), parse_pair), Object);
    let parse = preceded(
        pair(char('{'), multispace0),
        terminated(pairs, pair(multispace0, char('}'))),
    );

    pp_ret("object", context("object", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_value(i: Span) -> Res<Value> {
    let parse = alt((
        map(parse_null, Value::Null),
        map(parse_boolean, Value::Boolean),
        map(parse_array, Value::Array),
        map(parse_object, Value::Object),
        map(parse_node_path, Value::NodePath),
        map(parse_attr_expr, Value::AttrExpr),
        map(parse_function_call, Value::FuncCall),
        map(parse_ident, Value::Ident),
        map(parse_string, Value::String),
        map(parse_float, Value::Float),
        map(parse_int, Value::Int),
    ));
    pp_ret("value", context("value", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_pass(i: Span) -> Res<Pass> {
    pp_ret("pass", value(Pass, tag("pass"))(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_pair(i: Span) -> Res<Pair> {
    pp_ret(
        "pair",
        context(
            "pair",
            map(
                separated_pair(wsl(parse_value), char(':'), wsl(parse_value)),
                |(k, v)| Pair(k, v),
            ),
        )(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_comment(i: Span) -> Res<Comment> {
    let parse = map(preceded(char('#'), is_not("\n\r")), |s: Span| {
        Comment(s.trim())
    });
    pp_ret("comment", context("comment", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_var_modifier(i: Span) -> Res<Option<VarModifier>> {
    let onready = value(VarModifier::OnReady, tag("onready"));
    let export = value(VarModifier::Export, tag("export"));
    let parse = opt(alt((onready, export)));

    pp_ret("var_modifier", context("var_modifier", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_var_decl(i: Span) -> Res<VarDecl> {
    let parse_assign_type = pair(
        opt(preceded(ws(char(':')), parse_var_type)),
        opt(preceded(ws(char('=')), parse_expr)),
    );
    let parse_assign_infer = preceded(ws(tag(":=")), parse_expr);
    let parse_assign = alt((
        map(parse_assign_infer, |e| (true, None, Some(e))),
        map(parse_assign_type, |(typ, exp)| (false, typ, exp)),
    ));

    let parse = map(
        tuple((
            parse_var_modifier,
            ws(tag("var")),
            parse_ident,
            opt(parse_assign),
        )),
        |(modifier, _, name, assign)| match assign {
            Some((infer, typ, exp)) => VarDecl {
                modifier,
                name: name.0,
                infer,
                r#type: typ,
                value: exp,
            },
            None => VarDecl {
                modifier,
                name: name.0,
                infer: false,
                r#type: None,
                value: None,
            },
        },
    );

    pp_ret("var_decl", context("var_decl", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_extends_decl(i: Span) -> Res<ExtendsDecl> {
    let parse = map(
        preceded(
            pair(tag("extends"), space1),
            alt((map(parse_string, |x| x.0), map(parse_ident, |x| x.0))),
        ),
        ExtendsDecl,
    );
    pp_ret("extends_decl", context("extends_decl", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_classname_decl(i: Span) -> Res<ClassNameDecl> {
    let parse = map(
        preceded(pair(tag("class_name"), space1), map(parse_ident, |x| x.0)),
        ClassNameDecl,
    );
    pp_ret("classname_decl", context("classname_decl", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_signal_decl(i: Span) -> Res<SignalDecl> {
    let parse_args = separated_list0(ws(char(',')), parse_ident);
    let mut parse = map(
        preceded(
            pair(tag("signal"), space1),
            pair(
                parse_ident,
                opt(delimited(ws(char('(')), parse_args, ws(char(')')))),
            ),
        ),
        |(name, args)| SignalDecl {
            name,
            args: args.unwrap_or_default(),
        },
    );

    pp_ret("signal_decl", parse(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_decl(i: Span, indent: usize) -> Res<Decl> {
    let parse = alt((
        map(parse_var_decl, Decl::Var),
        map(parse_extends_decl, Decl::Extends),
        map(parse_classname_decl, Decl::ClassName),
        map(parse_signal_decl, Decl::Signal),
        map(|i| parse_function_decl(i, indent), Decl::Function),
    ));
    pp_ret("decl", context("decl", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_stmt(i: Span, indent: usize) -> Res<Stmt> {
    let parse = alt((
        map(|i| parse_if_stmt(i, indent), Stmt::If),
        map(|i| parse_while_stmt(i, indent), Stmt::While),
        map(|i| parse_for_stmt(i, indent), Stmt::For),
        map(|i| parse_match_stmt(i, indent), Stmt::Match),
        map(parse_pass, Stmt::Pass),
    ));
    pp_ret("stmt", context("stmt", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_line_fragment(i: Span, indent: usize) -> Res<LineFragment> {
    let parse = alt((
        map(|i| parse_stmt(i, indent), LineFragment::Stmt),
        map(|i| parse_decl(i, indent), LineFragment::Decl),
        map(parse_expr, LineFragment::Expr),
        map(parse_comment, LineFragment::Comment),
    ));

    pp_ret("line_fragment", context("line_fragment", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_line(i: Span, indent: usize) -> Res<Line> {
    let fragment = |i| parse_line_fragment(i, indent);
    let parse = map(many1(fragment), Line);
    pp_ret("line", context("line", parse)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_block(i: Span, indent: usize) -> Res<Block> {
    let parse = separated_list0(
        line_ending,
        alt((
            preceded(|i| same_indent(i, indent), |i| parse_line(i, indent)),
            map(parse_empty_line, |_| Line(vec![])),
        )),
    );

    pp_ret(
        "block",
        context(
            "block",
            map(parse, |v| {
                Block(v.into_iter().filter(|x| !x.0.is_empty()).collect())
            }),
        )(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_file(i: Span) -> Res<Block> {
    context("file", all_consuming(move |i| parse_block(i, 0)))(i)
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_function_call(i: Span) -> Res<FunctionCall> {
    let mut parse = map(
        pair(
            parse_ident,
            delimited(
                ws(char('(')),
                opt(separated_list0(ws(char(',')), parse_expr)),
                ws(char(')')),
            ),
        ),
        |(name, args)| FunctionCall::new(name.0, args.unwrap_or_default()),
    );

    pp_ret("function_call", parse(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_attr_node(i: Span) -> Res<AttrNode> {
    let mut parse = alt((
        map(preceded(char('.'), parse_function_call), AttrNode::FuncCall),
        map(preceded(char('.'), parse_ident), |i| AttrNode::Name(i.0)),
        map(delimited(char('['), parse_int, char(']')), |i| {
            AttrNode::Index(i.0 as usize)
        }),
    ));

    pp_ret("attr_node", parse(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_first_attr_node(i: Span) -> Res<AttrNode> {
    let mut parse = alt((
        map(parse_function_call, AttrNode::FuncCall),
        map(parse_ident, |i| AttrNode::Name(i.0)),
    ));

    pp_ret("first_attr_node", parse(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_attr_expr(i: Span) -> Res<AttrExpr> {
    let mut parse = map(
        pair(parse_first_attr_node, many1(parse_attr_node)),
        |(first, mut rest)| {
            rest.insert(0, first);
            AttrExpr(rest)
        },
    );
    pp_ret("attr_expr", parse(i))
}

pub fn more_indent(i: Span, indent: usize) -> Res<usize> {
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
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn same_indent(i: Span, indent: usize) -> Res<()> {
    let (s, parsed) = parse_indentation(i)?;
    pp_ret(
        "same_indent",
        if parsed == indent {
            Ok((s, ()))
        } else {
            Err(nom::Err::Error(VerboseError {
                errors: vec![(s, VerboseErrorKind::Context("not the same indentation"))],
            }))
        },
    )
}

pub fn less_indent(i: Span, indent: usize) -> Res<usize> {
    let (s, parsed) = scan_indentation(i)?;
    if parsed < indent {
        Ok((s, parsed))
    } else {
        Err(nom::Err::Error(VerboseError {
            errors: vec![(
                s,
                VerboseErrorKind::Context("should be a lesser indentation"),
            )],
        }))
    }
}

pub fn parse_indentation(i: Span) -> Res<usize> {
    let (s, spaces) = many0(char(' '))(i)?;
    let indent_level = spaces.len();
    Ok((s, indent_level))
}

pub fn scan_indentation(i: Span) -> Res<usize> {
    let (s, spaces) = peek(many0(char(' ')))(i)?;
    let indent_level = spaces.len();
    Ok((s, indent_level))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_expr(i: Span) -> Res<Expr> {
    pp_ret("expr", parse_expr_logic_expr(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_expr_logic_expr(i: Span) -> Res<Expr> {
    let (i, num1) = parse_expr_math_expr(i)?;
    let (i, exprs) = many0(tuple((
        map(
            alt((
                tag("&&"),
                tag("and"),
                tag("||"),
                tag("or"),
                tag(">="),
                tag("<="),
                tag("=="),
                tag("!="),
                tag("is"),
                tag("as"),
            )),
            |k: Span| *k,
        ),
        parse_expr_logic_expr,
    )))(i)?;

    pp_ret("expr_logic_expr", Ok((i, parse_expr_rec(num1, exprs))))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_expr_math_expr(i: Span) -> Res<Expr> {
    let (i, num1) = parse_expr_term(i)?;
    let (i, exprs) = many0(tuple((
        map(ws(alt((tag("+"), tag("-")))), |k: Span| *k),
        parse_expr_math_expr,
    )))(i)?;

    pp_ret("expr_math_expr", Ok((i, parse_expr_rec(num1, exprs))))
}

pub fn parse_expr_rec<'a>(a: Expr<'a>, rem: Vec<(&str, Expr<'a>)>) -> Expr<'a> {
    rem.into_iter()
        .fold(a, |acc, val| parse_expr_binop(val, acc))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_expr_un(i: Span) -> Res<Expr> {
    let (i, op) = opt(map(ws(alt((tag("-"), tag("+"), tag("!")))), |k: Span| *k))(i)?;
    let (i, num) = parse_expr_operation(i)?;

    pp_ret(
        "expr_un",
        match op {
            Some(op) => Ok((i, parse_expr_unop(op, num))),
            None => Ok((i, num)),
        },
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_expr_term(i: Span) -> Res<Expr> {
    let (i, num1) = parse_expr_un(i)?;
    let (i, exprs) = many0(tuple((
        map(
            ws(alt((tag("*"), tag("/"), tag("%"), tag("|"), tag("&")))),
            |k: Span| *k,
        ),
        parse_expr_term,
    )))(i)?;

    pp_ret("expr_term", Ok((i, parse_expr_rec(num1, exprs))))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_expr_operation(i: Span) -> Res<Expr> {
    pp_ret(
        "expr_operation",
        alt((parse_expr_parens, parse_expr_value))(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_expr_value(i: Span) -> Res<Expr> {
    pp_ret(
        "expr_value",
        map(delimited(space0, parse_value, space0), Expr::Value)(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_expr_parens(i: Span) -> Res<Expr> {
    pp_ret(
        "expr_parens",
        delimited(space0, delimited(char('('), parse_expr, char(')')), space0)(i),
    )
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
        "+" => new_expr(expr1, expr2, BinOp::Add),
        "-" => new_expr(expr1, expr2, BinOp::Sub),
        "*" => new_expr(expr1, expr2, BinOp::Mul),
        "/" => new_expr(expr1, expr2, BinOp::Div),
        "%" => new_expr(expr1, expr2, BinOp::Mod),
        "&" => new_expr(expr1, expr2, BinOp::BinAnd),
        "|" => new_expr(expr1, expr2, BinOp::BinOr),
        "&&" | "and" => new_expr(expr1, expr2, BinOp::And),
        "||" | "or" => new_expr(expr1, expr2, BinOp::Or),
        "==" => new_expr(expr1, expr2, BinOp::Eq),
        "!=" => new_expr(expr1, expr2, BinOp::Neq),
        ">" => new_expr(expr1, expr2, BinOp::Gt),
        ">=" => new_expr(expr1, expr2, BinOp::Gte),
        "<=" => new_expr(expr1, expr2, BinOp::Lte),
        "is" => new_expr(expr1, expr2, BinOp::Is),
        "as" => new_expr(expr1, expr2, BinOp::As),
        _ => unreachable!(),
    }
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_indented_block(i: Span, indent: usize) -> Res<Block> {
    pp_ret(
        "indented_block",
        preceded(many1(line_ending), |i| more_indent(i, indent))(i)
            .and_then(|(i, indent)| parse_block(i, indent)),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_if_stmt(i: Span, indent: usize) -> Res<IfStmt> {
    let parse = preceded(tag("if"), |i| parse_condition(i, indent));

    pp_ret(
        "if_stmt",
        context(
            "if_stmt",
            map(
                tuple((
                    parse,
                    many0(preceded(
                        pair(line_ending, |i| same_indent(i, indent)),
                        |i| parse_elif_stmt(i, indent),
                    )),
                    opt(preceded(
                        pair(line_ending, |i| same_indent(i, indent)),
                        |i| parse_else_stmt(i, indent),
                    )),
                )),
                |(ifb, elifb, elseb)| IfStmt {
                    if_branch: ifb,
                    elif_branches: elifb,
                    else_branch: elseb,
                },
            ),
        )(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_elif_stmt(i: Span, indent: usize) -> Res<ElifStmt> {
    let parse = preceded(tag("elif"), |i| parse_condition(i, indent));

    pp_ret("elif_stmt", map(parse, ElifStmt)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_else_stmt(i: Span, indent: usize) -> Res<ElseStmt> {
    let parse = preceded(terminated(tag("else"), ws(char(':'))), |i| {
        parse_indented_block(i, indent)
    });

    pp_ret("else_stmt", map(parse, ElseStmt)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_while_stmt(i: Span, indent: usize) -> Res<WhileStmt> {
    let parse = preceded(tag("while"), |i| parse_condition(i, indent));

    pp_ret("while_stmt", map(parse, WhileStmt)(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_for_stmt(i: Span, indent: usize) -> Res<ForStmt> {
    let mut parse = map(
        preceded(
            ws(tag("for")),
            tuple((
                parse_expr,
                terminated(preceded(ws(tag("in")), parse_expr), ws(char(':'))),
                |i| parse_indented_block(i, indent),
            )),
        ),
        |(expr, in_expr, block)| ForStmt {
            expr,
            in_expr,
            block,
        },
    );

    pp_ret("for_stmt", parse(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_match_stmt(i: Span, indent: usize) -> Res<MatchStmt> {
    let (i, expr) = terminated(
        preceded(pair(tag("match"), space1), parse_expr),
        ws(char(':')),
    )(i)?;
    let (i, _) = line_ending(i)?;
    let (i, scanned_indent) = more_indent(i, indent)?;
    let (i, cases) = separated_list0(
        line_ending,
        preceded(
            |i| same_indent(i, scanned_indent),
            terminated(
                |i| parse_match_case_stmt(i, scanned_indent),
                opt(parse_empty_line),
            ),
        ),
    )(i)?;

    pp_ret("match_stmt", Ok((i, MatchStmt { expr, cases })))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_match_case_stmt(i: Span, indent: usize) -> Res<MatchCaseStmt> {
    pp_ret(
        "match_case_stmt",
        map(|i| parse_condition(i, indent), MatchCaseStmt)(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_condition(i: Span, indent: usize) -> Res<Condition> {
    let parse_cond = terminated(preceded(space0, parse_expr), ws(char(':')));

    pp_ret(
        "condition",
        context(
            "condition",
            map(
                pair(parse_cond, |i| parse_indented_block(i, indent)),
                |(e, b)| Condition { expr: e, block: b },
            ),
        )(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_dotted_ident(i: Span) -> Res<DottedIdent> {
    pp_ret(
        "dotted_ident",
        map(
            recognize(separated_list1(char('.'), parse_ident)),
            |x: Span| DottedIdent(*x),
        )(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_var_type(i: Span) -> Res<VarType> {
    parse_dotted_ident(i)
}

#[tracing::instrument(skip_all, fields(i = pp_span(i)))]
pub fn parse_function_arg(i: Span) -> Res<FunctionArg> {
    pp_ret(
        "function_arg",
        map(
            tuple((
                parse_ident,
                opt(preceded(ws(char(':')), parse_var_type)),
                opt(preceded(ws(char('=')), parse_expr)),
            )),
            |(ident, typ, expr)| FunctionArg {
                name: ident,
                r#type: typ,
                default: expr,
            },
        )(i),
    )
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_function_decl(i: Span, indent: usize) -> Res<FunctionDecl> {
    let parse_args = delimited(
        ws(char('(')),
        separated_list0(wsl(char(',')), parse_function_arg),
        ws(char(')')),
    );
    let parse_type = opt(preceded(ws(tag("->")), parse_var_type));
    let parse_header = preceded(
        pair(tag("func"), space1),
        terminated(tuple((parse_ident, parse_args, parse_type)), ws(char(':'))),
    );

    let mut parse = map(
        pair(parse_header, |i| parse_indented_block(i, indent)),
        |((ident, args, typ), block)| FunctionDecl {
            name: ident,
            args,
            return_type: typ,
            block,
        },
    );

    pp_ret("function_decl", parse(i))
}

#[tracing::instrument(skip_all, fields(i = pp_span(i), indent = indent))]
pub fn parse_class_decl(i: Span, indent: usize) -> Res<ClassDecl> {
    let mut parse = map(
        preceded(
            ws(tag("class")),
            pair(terminated(parse_ident, ws(char(':'))), |i| {
                parse_indented_block(i, indent)
            }),
        ),
        |(name, block)| ClassDecl { name, block },
    );

    pp_ret("class_decl", parse(i))
}
