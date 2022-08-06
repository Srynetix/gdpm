use indoc::indoc;
use nom_locate::LocatedSpan;
use pretty_assertions::assert_eq;

use crate::ast::*;
use crate::parsers::*;
use crate::{
    debug::init_tracing,
    types::{Res, Span},
};

/// Assert parse
#[track_caller]
fn ap<'a, T: PartialEq + std::fmt::Debug>(
    func: impl FnMut(Span<'a>) -> Res<'a, T>,
    input: &'static str,
    value: T,
) {
    apr(func, input, value, "")
}

/// Assert parse no-check
#[track_caller]
fn apn<'a, T: PartialEq + std::fmt::Debug>(
    mut func: impl FnMut(Span<'a>) -> Res<'a, T>,
    input: &'static str,
) {
    match func(input.into()) {
        Ok(_) => {}
        Err(e) => match e {
            nom::Err::Error(err) => panic!("Could not parse: {}", err),
            nom::Err::Failure(err) => panic!("Could not parse: {}", err),
            _ => panic!("Could not parse: {}", e),
        },
    }
}
/// Assert parse with indent
#[track_caller]
fn api<'a, T: PartialEq + std::fmt::Debug>(
    mut func: impl FnMut(Span<'a>, usize) -> Res<'a, T>,
    input: &'static str,
    value: T,
) {
    let f = move |i| func(i, 0);
    apr(f, input, value, "")
}

/// Assert parse with remaining
#[track_caller]
fn apr<'a, T: PartialEq + std::fmt::Debug>(
    mut func: impl FnMut(Span<'a>) -> Res<'a, T>,
    input: &'static str,
    value: T,
    remaining: &'static str,
) {
    match func(input.into()) {
        Ok(res) => {
            // Validate capture
            assert_eq!(res.1, value);

            // Validate remaining
            let remaining_span = LocatedSpan::new(remaining);

            match (remaining_span, res.0) {
                (a, b) if *a == *b => (),
                (a, b) if *a == "" => panic!("remaining value should be empty: {b}"),
                (a, b) => panic!("remaining value should NOT be empty: {a} != {b}"),
            }
        }
        Err(e) => match e {
            nom::Err::Error(err) => panic!("Could not parse: {}", err),
            nom::Err::Failure(err) => panic!("Could not parse: {}", err),
            _ => panic!("Could not parse: {}", e),
        },
    }
}

/// Assert not parse
#[track_caller]
fn anp<'a, T: std::fmt::Debug>(mut func: impl FnMut(Span<'a>) -> Res<'a, T>, input: &'static str) {
    match func(input.into()) {
        Err(_) => (),
        Ok(r) => {
            assert!(false, "Token should not match: {:?}", r);
        }
    }
}

/// Assert not parse widh indent
#[track_caller]
fn anpi<'a, T: std::fmt::Debug>(
    mut func: impl FnMut(Span<'a>, usize) -> Res<'a, T>,
    input: &'static str,
) {
    let mut f = move |i| func(i, 0);

    match f(input.into()) {
        Err(_) => (),
        Ok(r) => {
            assert!(false, "Token should not match: {:?}", r);
        }
    }
}

fn setup() {
    init_tracing().unwrap();
}

#[test]
fn test_ident() {
    ap(parse_ident, "abcd", Ident("abcd"));
    ap(parse_ident, "_abcd", Ident("_abcd"));
    ap(parse_ident, "_0abcd", Ident("_0abcd"));
    apr(parse_ident, "abcd+", Ident("abcd"), "+");

    anp(parse_ident, "0abcd");
}

#[test]
fn test_boolean() {
    ap(parse_boolean, "true", Boolean(true));
    ap(parse_boolean, "false", Boolean(false));
    ap(parse_boolean, "True", Boolean(true));
    ap(parse_boolean, "False", Boolean(false));
    apr(parse_boolean, "true0", Boolean(true), "0");

    anp(parse_boolean, "FOO");
    anp(parse_boolean, "TRUE");
    anp(parse_boolean, "FALSE");
}

#[test]
fn test_int() {
    ap(parse_int, "1234", Int(1234));
    ap(parse_int, "01234", Int(1234));
    ap(parse_int, "0", Int(0));
    apr(parse_int, "0foo", Int(0), "foo");

    anp(parse_int, "-1234");
    anp(parse_int, "foo");
    anp(parse_int, "foo0");
}

#[test]
fn test_null() {
    ap(parse_null, "null", Null);
    apr(parse_null, "null0", Null, "0");

    anp(parse_null, "NULL");
    anp(parse_null, "FOO");
    anp(parse_null, "0");
}

#[test]
fn test_float() {
    ap(parse_float, "1.05", Float(1.05));
    apr(parse_float, "1.0.0", Float(1.0), ".0");
    apr(parse_float, "1.0foo", Float(1.0), "foo");

    anp(parse_float, "1");
    anp(parse_float, "foo");
}

#[test]
fn test_string() {
    ap(parse_string, r#""Hello""#, GdString("Hello"));
    ap(parse_string, r#"'Hello'"#, GdString("Hello"));
    ap(
        parse_string,
        r#"'Hello/123 4, 5'"#,
        GdString("Hello/123 4, 5"),
    );
    ap(
        parse_string,
        r#""Hello/123 4, 5""#,
        GdString("Hello/123 4, 5"),
    );

    anp(parse_string, "Hello");
}

#[test]
fn test_node_path() {
    ap(parse_node_path, "$My", NodePath("$My"));
    ap(parse_node_path, "$My/Path", NodePath("$My/Path"));
    ap(
        parse_node_path,
        "$My/Other/Path",
        NodePath("$My/Other/Path"),
    );
    ap(
        parse_node_path,
        r#"$"My/Other/Path With Space""#,
        NodePath(r#"$"My/Other/Path With Space""#),
    );

    anp(parse_node_path, "My/Path");
    anp(parse_node_path, "'My/Path");
}

#[test]
fn test_value() {
    ap(parse_value, "hello", Value::Ident(Ident("hello")));
    ap(parse_value, "'hello'", Value::String(GdString("hello")));
    ap(parse_value, "1234", Value::Int(Int(1234)));
    ap(parse_value, "1.0", Value::Float(Float(1.0)));
    ap(parse_value, "[]", Value::Array(Array(vec![])));
    ap(parse_value, "{}", Value::Object(Object(vec![])));

    anp(parse_value, "}1a1a1");
}

#[test]
fn test_array() {
    ap(
        parse_array,
        "[1, id , \"ha\", 3.0, 4.15]",
        Array(vec![
            Value::Int(Int(1)),
            Value::Ident(Ident("id")),
            Value::String(GdString("ha")),
            Value::Float(Float(3.0)),
            Value::Float(Float(4.15)),
        ]),
    );
    ap(parse_array, "[]", Array(vec![]));
    ap(parse_array, "[ ]", Array(vec![]));

    anp(parse_array, "[1,, ]");
    anp(parse_array, "[1,,");
    anp(parse_array, "1,,]");
}

#[test]
fn test_pair() {
    ap(
        parse_pair,
        "12: 'hello'",
        Pair(Value::Int(Int(12)), Value::String(GdString("hello"))),
    );
    ap(
        parse_pair,
        "'a' : b",
        Pair(Value::String(GdString("a")), Value::Ident(Ident("b"))),
    );
}

#[test]
fn test_object() {
    ap(parse_object, "{}", Object(vec![]));
    ap(parse_object, "{ }", Object(vec![]));
    ap(
        parse_object,
        "{'a': 1}",
        Object(vec![Pair(Value::String(GdString("a")), Value::Int(Int(1)))]),
    );
    ap(
        parse_object,
        "{'a': 1, 2.0: id}",
        Object(vec![
            Pair(Value::String(GdString("a")), Value::Int(Int(1))),
            Pair(Value::Float(Float(2.0)), Value::Ident(Ident("id"))),
        ]),
    );
    ap(
        parse_object,
        "{1: [2, 3]\n}",
        Object(vec![Pair(
            Value::Int(Int(1)),
            Value::Array(Array(vec![Value::Int(Int(2)), Value::Int(Int(3))])),
        )]),
    );

    anp(parse_object, "{a,,}");
    anp(parse_object, "{a,}");
}

#[test]
fn test_comment() {
    ap(parse_comment, "# hello", Comment("hello"));
    ap(parse_comment, "# hello   ", Comment("hello"));
    ap(parse_comment, "#hello   ", Comment("hello"));
    ap(parse_comment, "#hello   1234", Comment("hello   1234"));
}

#[test]
fn test_var_decl() {
    ap(
        parse_var_decl,
        "var hello",
        VarDecl {
            modifier: None,
            name: "hello",
            r#type: None,
            infer: false,
            value: None,
        },
    );
    ap(
        parse_var_decl,
        "onready var hello",
        VarDecl {
            modifier: Some(VarModifier::OnReady),
            name: "hello",
            r#type: None,
            infer: false,
            value: None,
        },
    );
    ap(
        parse_var_decl,
        "onready var hello: Node",
        VarDecl {
            modifier: Some(VarModifier::OnReady),
            name: "hello",
            r#type: Some(DottedIdent("Node")),
            infer: false,
            value: None,
        },
    );
    ap(
        parse_var_decl,
        "onready    var   hello   :   Node",
        VarDecl {
            modifier: Some(VarModifier::OnReady),
            name: "hello",
            r#type: Some(DottedIdent("Node")),
            infer: false,
            value: None,
        },
    );
    ap(
        parse_var_decl,
        "var   hello  =   5.0",
        VarDecl {
            modifier: None,
            name: "hello",
            r#type: None,
            infer: false,
            value: Some(Expr::Value(Value::float(5.0))),
        },
    );
    ap(
        parse_var_decl,
        "onready    var   hello   :   Node = 5",
        VarDecl {
            modifier: Some(VarModifier::OnReady),
            name: "hello",
            r#type: Some(DottedIdent("Node")),
            infer: false,
            value: Some(Expr::Value(Value::int(5))),
        },
    );

    ap(
        parse_var_decl,
        "onready    var   hello  := 5+ 5",
        VarDecl {
            modifier: Some(VarModifier::OnReady),
            name: "hello",
            r#type: None,
            infer: true,
            value: Some(Expr::bin(
                Expr::value(Value::int(5)),
                BinOp::Add,
                Expr::value(Value::int(5)),
            )),
        },
    );

    anp(parse_var_decl, "var");
    anp(parse_var_decl, "foo   var");
}

#[test]
fn test_extends_decl() {
    ap(parse_extends_decl, "extends Node", ExtendsDecl("Node"));
    ap(
        parse_extends_decl,
        "extends \"res://sample.gd\"",
        ExtendsDecl("res://sample.gd"),
    );

    anp(parse_extends_decl, "extendsNode");
    anp(parse_extends_decl, "extends");
    anp(parse_extends_decl, "extends 12");
}

#[test]
fn test_classname_decl() {
    ap(parse_classname_decl, "class_name Foo", ClassNameDecl("Foo"));
    ap(
        parse_classname_decl,
        "class_name    Foo123",
        ClassNameDecl("Foo123"),
    );
    apr(
        parse_classname_decl,
        "class_name Foo Bar",
        ClassNameDecl("Foo"),
        " Bar",
    );

    anp(parse_classname_decl, "class_nameFoo");
}

#[test]
fn test_decl() {
    api(
        parse_decl,
        "class_name Foo",
        Decl::ClassName(ClassNameDecl("Foo")),
    );

    anpi(parse_decl, "foo");
}

#[test]
fn test_line_fragment() {
    api(
        parse_line_fragment,
        "class_name Foo",
        LineFragment::Decl(Decl::ClassName(ClassNameDecl("Foo"))),
    );
}

#[test]
fn test_line() {
    api(
        parse_line,
        "class_name Foo",
        Line(vec![LineFragment::Decl(Decl::ClassName(ClassNameDecl(
            "Foo",
        )))]),
    );
    api(
        parse_line,
        "class_name Foo# Hey !",
        Line(vec![
            LineFragment::Decl(Decl::ClassName(ClassNameDecl("Foo"))),
            LineFragment::Comment(Comment("Hey !")),
        ]),
    );
}

#[test]
fn test_block() {
    api(
        parse_block,
        indoc! {r#"
            # Hey!
            class_name Foo
            extends "res://Bar.gd""#},
        Block(vec![
            Line(vec![LineFragment::Comment(Comment("Hey!"))]),
            Line(vec![LineFragment::Decl(Decl::ClassName(ClassNameDecl(
                "Foo",
            )))]),
            Line(vec![LineFragment::Decl(Decl::Extends(ExtendsDecl(
                "res://Bar.gd",
            )))]),
        ]),
    );

    api(
        parse_block,
        indoc! {r#"
            abcd

            abcd"#},
        Block(vec![
            Line(vec![LineFragment::Expr(Expr::Value(Value::Ident(Ident(
                "abcd",
            ))))]),
            Line(vec![LineFragment::Expr(Expr::Value(Value::Ident(Ident(
                "abcd",
            ))))]),
        ]),
    );
}

#[test]
fn test_file() {
    anp(
        parse_file,
        indoc! {r#"
            class_name Foo
                class_name Foo
        "#},
    );
}

#[test]
fn test_indent() {
    // Level 0
    assert!(same_indent(LocatedSpan::new("ok"), 0).is_ok());
    assert!(more_indent(LocatedSpan::new("ok"), 0).is_err());
    assert!(less_indent(LocatedSpan::new("ok"), 0).is_err());

    // Level 1
    assert!(more_indent(LocatedSpan::new("  ok"), 0).is_ok());
    assert!(more_indent(LocatedSpan::new("  ok"), 2).is_err());
    assert!(same_indent(LocatedSpan::new("  ok"), 2).is_ok());
    assert!(less_indent(LocatedSpan::new("  ok"), 2).is_err());

    // Level 2
    assert!(less_indent(LocatedSpan::new("ok"), 2).is_ok());
    assert!(same_indent(LocatedSpan::new("ok"), 0).is_ok());
    assert!(more_indent(LocatedSpan::new("ok"), 0).is_err());
}

#[test]
fn test_expr() {
    ap(parse_expr, "123", Expr::Value(Value::Int(Int(123))));

    ap(
        parse_expr,
        "a + b",
        Expr::bin(
            Expr::value(Value::ident("a")),
            BinOp::Add,
            Expr::value(Value::ident("b")),
        ),
    );

    ap(
        parse_expr,
        "a * (b + c)",
        Expr::bin(
            Expr::value(Value::ident("a")),
            BinOp::Mul,
            Expr::bin(
                Expr::value(Value::ident("b")),
                BinOp::Add,
                Expr::value(Value::ident("c")),
            ),
        ),
    );

    ap(
        parse_expr,
        "-a * (b & 1 + c) && (5 + 10 / (2 % \"foo\"))",
        Expr::bin(
            Expr::bin(
                Expr::un(UnOp::Minus, Expr::value(Value::ident("a"))),
                BinOp::Mul,
                Expr::bin(
                    Expr::bin(
                        Expr::value(Value::ident("b")),
                        BinOp::BinAnd,
                        Expr::value(Value::int(1)),
                    ),
                    BinOp::Add,
                    Expr::value(Value::ident("c")),
                ),
            ),
            BinOp::And,
            Expr::bin(
                Expr::value(Value::int(5)),
                BinOp::Add,
                Expr::bin(
                    Expr::value(Value::int(10)),
                    BinOp::Div,
                    Expr::bin(
                        Expr::value(Value::int(2)),
                        BinOp::Mod,
                        Expr::value(Value::string("foo")),
                    ),
                ),
            ),
        ),
    );
}

#[test]
fn test_if_stmt() {
    api(
        parse_if_stmt,
        indoc!(
            r#"
            if 123456:
                hello"#
        ),
        IfStmt {
            if_branch: Condition {
                expr: Expr::Value(Value::Int(Int(123456))),
                block: Block(vec![Line(vec![LineFragment::Expr(Expr::Value(
                    Value::Ident(Ident("hello")),
                ))])]),
            },
            elif_branches: vec![],
            else_branch: None,
        },
    );
    api(
        parse_if_stmt,
        indoc!(
            r#"
            if 123456:
                hello
                hi"#
        ),
        IfStmt {
            if_branch: Condition {
                expr: Expr::Value(Value::Int(Int(123456))),
                block: Block(vec![
                    Line(vec![LineFragment::Expr(Expr::Value(Value::Ident(Ident(
                        "hello",
                    ))))]),
                    Line(vec![LineFragment::Expr(Expr::Value(Value::Ident(Ident(
                        "hi",
                    ))))]),
                ]),
            },
            elif_branches: vec![],
            else_branch: None,
        },
    );
    api(
        parse_if_stmt,
        indoc!(
            r#"
            if 123456:
                hello
                if 123456:
                    hi"#
        ),
        IfStmt {
            if_branch: Condition {
                expr: Expr::Value(Value::Int(Int(123456))),
                block: Block(vec![
                    Line(vec![LineFragment::Expr(Expr::Value(Value::Ident(Ident(
                        "hello",
                    ))))]),
                    Line(vec![LineFragment::Stmt(Stmt::If(IfStmt {
                        if_branch: Condition {
                            expr: Expr::Value(Value::Int(Int(123456))),
                            block: Block(vec![Line(vec![LineFragment::Expr(Expr::Value(
                                Value::Ident(Ident("hi")),
                            ))])]),
                        },
                        elif_branches: vec![],
                        else_branch: None,
                    }))]),
                ]),
            },
            elif_branches: vec![],
            else_branch: None,
        },
    );
    api(
        parse_block,
        indoc!(
            r#"
            if 123456:
                hello
            hi"#
        ),
        Block(vec![
            Line(vec![LineFragment::Stmt(Stmt::If(IfStmt {
                if_branch: Condition {
                    expr: Expr::Value(Value::Int(Int(123456))),
                    block: Block(vec![Line(vec![LineFragment::Expr(Expr::Value(
                        Value::Ident(Ident("hello")),
                    ))])]),
                },
                elif_branches: vec![],
                else_branch: None,
            }))]),
            Line(vec![LineFragment::Expr(Expr::Value(Value::Ident(Ident(
                "hi",
            ))))]),
        ]),
    );

    api(
        parse_block,
        indoc!(
            r#"
            if 123456:
                hello
            elif a:
                bar
            else:
                if a:
                    foo
                else:
                    foo
            hi"#
        ),
        Block(vec![
            Line(vec![LineFragment::Stmt(Stmt::If(IfStmt {
                if_branch: Condition {
                    expr: Expr::Value(Value::Int(Int(123456))),
                    block: Block(vec![Line(vec![LineFragment::Expr(Expr::Value(
                        Value::Ident(Ident("hello")),
                    ))])]),
                },
                elif_branches: vec![ElifStmt(Condition {
                    expr: Expr::Value(Value::Ident(Ident("a"))),
                    block: Block(vec![Line(vec![LineFragment::Expr(Expr::Value(
                        Value::Ident(Ident("bar")),
                    ))])]),
                })],
                else_branch: Some(ElseStmt(Block(vec![Line(vec![LineFragment::Stmt(
                    Stmt::If(IfStmt {
                        if_branch: Condition {
                            expr: Expr::Value(Value::Ident(Ident("a"))),
                            block: Block(vec![Line(vec![LineFragment::Expr(Expr::Value(
                                Value::Ident(Ident("foo")),
                            ))])]),
                        },
                        elif_branches: vec![],
                        else_branch: Some(ElseStmt(Block(vec![Line(vec![LineFragment::Expr(
                            Expr::Value(Value::Ident(Ident("foo"))),
                        )])]))),
                    }),
                )])]))),
            }))]),
            Line(vec![LineFragment::Expr(Expr::Value(Value::Ident(Ident(
                "hi",
            ))))]),
        ]),
    );

    anpi(
        parse_if_stmt,
        indoc!(
            r#"
            if 123456:
        "#
        ),
    );
    anpi(
        parse_if_stmt,
        indoc!(
            r#"
            if 123456:
            hello
        "#
        ),
    );
}

#[test]
fn test_dotted_ident() {
    ap(parse_dotted_ident, "abcd", DottedIdent("abcd"));
    ap(parse_dotted_ident, "abcd.abcd", DottedIdent("abcd.abcd"));
    ap(
        parse_dotted_ident,
        "abcd.abcd.abcd",
        DottedIdent("abcd.abcd.abcd"),
    );
}

#[test]
fn test_function_decl() {
    api(
        parse_function_decl,
        indoc! {r#"
        func   foo  (bar  , baz  :  Node, qux = 1234 )  ->   Node:
            spam"#},
        FunctionDecl {
            name: Ident("foo"),
            args: vec![
                FunctionArg {
                    name: Ident("bar"),
                    r#type: None,
                    default: None,
                },
                FunctionArg {
                    name: Ident("baz"),
                    r#type: Some(DottedIdent("Node")),
                    default: None,
                },
                FunctionArg {
                    name: Ident("qux"),
                    r#type: None,
                    default: Some(Expr::Value(Value::Int(Int(1234)))),
                },
            ],
            return_type: Some(DottedIdent("Node")),
            block: Block(vec![Line(vec![LineFragment::Expr(Expr::Value(
                Value::Ident(Ident("spam")),
            ))])]),
        },
    );
}

#[test]
fn test_while_stmt() {
    api(
        parse_while_stmt,
        indoc! {r#"
        while true:
            pass"#},
        WhileStmt(Condition {
            expr: Expr::Value(Value::Boolean(Boolean(true))),
            block: Block(vec![Line(vec![LineFragment::Stmt(Stmt::Pass(Pass))])]),
        }),
    );
}

#[test]
fn test_for_stmt() {
    api(
        parse_for_stmt,
        indoc! {r#"
        for x in array:
            pass"#},
        ForStmt {
            expr: Expr::Value(Value::Ident(Ident("x"))),
            in_expr: Expr::Value(Value::Ident(Ident("array"))),
            block: Block(vec![Line(vec![LineFragment::Stmt(Stmt::Pass(Pass))])]),
        },
    )
}

#[test]
fn test_match_stmt() {
    api(
        parse_match_stmt,
        indoc! {r#"
            match x:
                foo:

                    pass

                _:
                    pass"#},
        MatchStmt {
            expr: Expr::value(Value::ident("x")),
            cases: vec![
                MatchCaseStmt(Condition {
                    expr: Expr::value(Value::ident("foo")),
                    block: Block(vec![Line(vec![LineFragment::Stmt(Stmt::Pass(Pass))])]),
                }),
                MatchCaseStmt(Condition {
                    expr: Expr::value(Value::ident("_")),
                    block: Block(vec![Line(vec![LineFragment::Stmt(Stmt::Pass(Pass))])]),
                }),
            ],
        },
    )
}

#[test]
fn test_class_decl() {
    api(
        parse_class_decl,
        indoc! {r#"
            class Foo:
                pass"#},
        ClassDecl {
            name: Ident("Foo"),
            block: Block(vec![Line(vec![LineFragment::Stmt(Stmt::Pass(Pass))])]),
        },
    )
}

#[test]
fn test_signal_decl() {
    ap(
        parse_signal_decl,
        "signal foo",
        SignalDecl {
            name: Ident("foo"),
            args: vec![],
        },
    );

    ap(
        parse_signal_decl,
        "signal foo()",
        SignalDecl {
            name: Ident("foo"),
            args: vec![],
        },
    );

    ap(
        parse_signal_decl,
        "signal foo(a)",
        SignalDecl {
            name: Ident("foo"),
            args: vec![Ident("a")],
        },
    );

    ap(
        parse_signal_decl,
        "signal foo(a, b)",
        SignalDecl {
            name: Ident("foo"),
            args: vec![Ident("a"), Ident("b")],
        },
    );
}

#[test]
fn test_function_call() {
    ap(
        parse_function_call,
        "foo()",
        FunctionCall::new("foo", vec![]),
    );
    ap(
        parse_function_call,
        "foo(a)",
        FunctionCall::new("foo", vec![Expr::value(Value::ident("a"))]),
    );
    ap(
        parse_function_call,
        "foo(a, 1234)",
        FunctionCall::new(
            "foo",
            vec![
                Expr::value(Value::ident("a")),
                Expr::value(Value::int(1234)),
            ],
        ),
    );
}

#[test]
fn test_attr_expr() {
    ap(
        parse_attr_expr,
        "a.b",
        AttrExpr(vec![AttrNode::Name("a"), AttrNode::Name("b")]),
    );
    ap(
        parse_attr_expr,
        "a.b[1].c",
        AttrExpr(vec![
            AttrNode::Name("a"),
            AttrNode::Name("b"),
            AttrNode::Index(1),
            AttrNode::Name("c"),
        ]),
    );
    ap(
        parse_attr_expr,
        "a(123).abcd",
        AttrExpr(vec![
            AttrNode::FuncCall(FunctionCall::new("a", vec![Expr::value(Value::int(123))])),
            AttrNode::Name("abcd"),
        ]),
    );
    ap(
        parse_attr_expr,
        "a(123)[1].abcd",
        AttrExpr(vec![
            AttrNode::FuncCall(FunctionCall::new("a", vec![Expr::value(Value::int(123))])),
            AttrNode::Index(1),
            AttrNode::Name("abcd"),
        ]),
    );
    ap(
        parse_attr_expr,
        "abcd.efgh.ijkl()",
        AttrExpr(vec![
            AttrNode::Name("abcd"),
            AttrNode::Name("efgh"),
            AttrNode::FuncCall(FunctionCall::new("ijkl", vec![])),
        ]),
    );

    anp(parse_attr_expr, "a");
    anp(parse_attr_expr, "a()");
}

// #[test]
// fn test_sample() {
//     apn(
//         parse_file,
//         include_str!("samples/Player.gd")
//     );
// }
