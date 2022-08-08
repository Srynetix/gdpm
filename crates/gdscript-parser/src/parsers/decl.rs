use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, space0, space1},
    combinator::{map, opt, value},
    error::context,
    multi::{separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
};
use nom_tracable::tracable_parser;

use crate::ast::{
    ClassDecl, ClassNameDecl, ConstDecl, Decl, EnumDecl, EnumVariant, ExtendsDecl, FunctionArg,
    FunctionDecl, FunctionModifier, SignalDecl, VarDecl, VarModifier,
};
use crate::types::{Res, Span};

use super::{
    base::{ms0noc, ws, wslnoc},
    expr::{parse_dotted_ident, parse_expr, parse_ident, parse_string, parse_value},
    parse_indented_block,
};

#[tracable_parser]
pub fn parse_var_modifier(i: Span) -> Res<Option<VarModifier>> {
    let onready = value(VarModifier::OnReady, tag("onready"));
    let export = value(VarModifier::Export, tag("export"));
    let parse = opt(alt((onready, export)));

    context("var_modifier", parse)(i)
}

#[tracable_parser]
pub fn parse_var_decl(i: Span) -> Res<VarDecl> {
    let parse_assign_type = pair(
        opt(preceded(ws(char(':')), parse_dotted_ident)),
        opt(preceded(ws(char('=')), parse_expr)),
    );
    let parse_assign_infer = preceded(ws(tag(":=")), parse_expr);
    let parse_assign = map(
        opt(alt((
            map(parse_assign_infer, |e| (true, None, Some(e))),
            map(parse_assign_type, |(typ, exp)| (false, typ, exp)),
        ))),
        |assign| assign.unwrap_or((false, None, None)),
    );
    let parse_setget = map(
        opt(preceded(
            ws(tag("setget")),
            alt((
                map(
                    pair(parse_ident, preceded(ws(char(',')), parse_ident)),
                    |(i1, i2)| (Some(i1.0), Some(i2.0)),
                ),
                map(preceded(ws(char(',')), parse_ident), |ident| {
                    (None, Some(ident.0))
                }),
                map(parse_ident, |ident| (Some(ident.0), None)),
            )),
        )),
        |v| v.unwrap_or((None, None)),
    );

    let parse = map(
        tuple((
            parse_var_modifier,
            ws(tag("var")),
            parse_ident,
            parse_assign,
            parse_setget,
        )),
        |(modifier, _, name, (infer, typ, exp), (setf, getf))| VarDecl {
            modifier,
            name: name.0,
            infer,
            r#type: typ,
            value: exp,
            set_func: setf,
            get_func: getf,
        },
    );

    context("var_decl", parse)(i)
}

#[tracable_parser]
pub fn parse_const_decl(i: Span) -> Res<ConstDecl> {
    let parse_assign_type = pair(
        opt(preceded(ws(char(':')), parse_dotted_ident)),
        preceded(ws(char('=')), parse_expr),
    );
    let parse_assign_infer = preceded(ws(tag(":=")), parse_expr);
    let parse_assign = alt((
        map(parse_assign_infer, |e| (true, None, e)),
        map(parse_assign_type, |(typ, exp)| (false, typ, exp)),
    ));

    let parse = map(
        preceded(ws(tag("const")), pair(parse_ident, parse_assign)),
        |(name, (infer, typ, exp))| ConstDecl {
            name: name.0,
            infer,
            r#type: typ,
            value: exp,
        },
    );

    context("const_decl", parse)(i)
}

#[tracable_parser]
pub fn parse_extends_decl(i: Span) -> Res<ExtendsDecl> {
    let parse = map(
        preceded(
            pair(tag("extends"), space1),
            alt((map(parse_string, |x| x.0), map(parse_ident, |x| x.0))),
        ),
        ExtendsDecl,
    );
    context("extends_decl", parse)(i)
}

#[tracable_parser]
pub fn parse_classname_decl(i: Span) -> Res<ClassNameDecl> {
    let parse = map(
        preceded(pair(tag("class_name"), space1), map(parse_ident, |x| x.0)),
        ClassNameDecl,
    );
    context("classname_decl", parse)(i)
}

#[tracable_parser]
pub fn parse_signal_decl(i: Span) -> Res<SignalDecl> {
    let parse_args = separated_list0(ws(char(',')), parse_ident);
    let parse = map(
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

    context("signal_decl", parse)(i)
}

#[tracable_parser]
pub fn parse_decl(i: Span, indent: usize) -> Res<Decl> {
    let parse = alt((
        map(parse_classname_decl, Decl::ClassName),
        map(parse_extends_decl, Decl::Extends),
        map(parse_signal_decl, Decl::Signal),
        map(parse_enum_decl, Decl::Enum),
        map(|i| parse_class_decl(i, indent), Decl::Class),
        map(|i| parse_function_decl(i, indent), Decl::Function),
        map(parse_const_decl, Decl::Const),
        map(parse_var_decl, Decl::Var),
    ));

    context("decl", parse)(i)
}

#[tracable_parser]
pub fn parse_enum_variant(i: Span) -> Res<EnumVariant> {
    let parse = map(
        pair(ws(parse_ident), opt(preceded(ws(char('=')), parse_value))),
        |(ident, value)| EnumVariant {
            name: ident.0,
            value,
        },
    );

    context("enum_variant", parse)(i)
}

#[tracable_parser]
pub fn parse_enum_decl(i: Span) -> Res<EnumDecl> {
    let parse = map(
        preceded(
            pair(tag("enum"), space1),
            pair(
                parse_ident,
                delimited(
                    preceded(space0, pair(char('{'), ms0noc)),
                    separated_list1(wslnoc(char(',')), parse_enum_variant),
                    pair(ms0noc, char('}')),
                ),
            ),
        ),
        |(name, variants)| EnumDecl {
            name: name.0,
            variants,
        },
    );

    context("enum_decl", parse)(i)
}

#[tracable_parser]
pub fn parse_function_arg(i: Span) -> Res<FunctionArg> {
    context(
        "function_arg",
        map(
            tuple((
                parse_ident,
                opt(preceded(ws(char(':')), parse_dotted_ident)),
                opt(preceded(ws(char('=')), parse_expr)),
            )),
            |(ident, typ, expr)| FunctionArg {
                name: ident,
                r#type: typ,
                default: expr,
            },
        ),
    )(i)
}

#[tracable_parser]
pub fn parse_function_modifier(i: Span) -> Res<FunctionModifier> {
    let parse = alt((
        value(FunctionModifier::Static, tag("static")),
        value(FunctionModifier::RemoteSync, tag("remotesync")),
        value(FunctionModifier::MasterSync, tag("mastersync")),
        value(FunctionModifier::PuppetSync, tag("puppetsync")),
        value(FunctionModifier::Remote, tag("remote")),
        value(FunctionModifier::Master, tag("master")),
        value(FunctionModifier::Puppet, tag("puppet")),
    ));

    context("function_modifier", parse)(i)
}

#[tracable_parser]
pub fn parse_function_decl(i: Span, indent: usize) -> Res<FunctionDecl> {
    let parse_args = delimited(
        preceded(space0, terminated(char('('), ms0noc)),
        separated_list0(wslnoc(char(',')), parse_function_arg),
        preceded(ms0noc, terminated(char(')'), space0)),
    );
    let parse_type = opt(preceded(ws(tag("->")), parse_dotted_ident));
    let parse_header = pair(
        opt(terminated(parse_function_modifier, space1)),
        preceded(
            pair(tag("func"), space1),
            terminated(tuple((parse_ident, parse_args, parse_type)), ws(char(':'))),
        ),
    );

    let parse = map(
        pair(parse_header, |i| parse_indented_block(i, indent)),
        |((modifier, (ident, args, typ)), block)| FunctionDecl {
            modifier,
            name: ident,
            args,
            return_type: typ,
            block,
        },
    );

    context("function_decl", parse)(i)
}

#[tracable_parser]
pub fn parse_class_decl(i: Span, indent: usize) -> Res<ClassDecl> {
    let parse = map(
        preceded(
            ws(tag("class")),
            pair(terminated(parse_ident, ws(char(':'))), |i| {
                parse_indented_block(i, indent)
            }),
        ),
        |(name, block)| ClassDecl { name, block },
    );

    context("class_decl", parse)(i)
}
