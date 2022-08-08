use indoc::indoc;
use nom_tracable::cumulative_histogram;
use nom_tracable::histogram;
use pretty_assertions::assert_eq;

use crate::ast::*;
use crate::parsers::*;
use crate::types::{Res, Span};

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
    let span = new_span(input);
    match func(span) {
        Ok(_) => {}
        Err(e) => match e {
            nom::Err::Error(err) => panic!("Could not parse: {}", err),
            nom::Err::Failure(err) => panic!("Could not parse: {}", err),
            _ => panic!("Could not parse: {}", e),
        },
    }
}

/// Assert parse with remaining
#[track_caller]
fn apr<'a, T: PartialEq + std::fmt::Debug>(
    mut func: impl FnMut(Span<'a>) -> Res<'a, T>,
    input: &'static str,
    value: T,
    remaining: &'static str,
) {
    let span = new_span(input);
    match func(span) {
        Ok(res) => {
            histogram();
            cumulative_histogram();

            // Validate capture
            assert_eq!(res.1, value);

            // Validate remaining
            let remaining_span = new_span(remaining);

            match (remaining_span, res.0) {
                (a, b) if *a == *b => (),
                (a, b) if *a == "" => panic!("remaining value should be empty: {b}"),
                (a, b) => panic!("remaining value should NOT be empty: {a} != {b}"),
            }
        }
        Err(e) => {
            histogram();
            cumulative_histogram();

            match e {
                nom::Err::Error(err) => panic!("Could not parse: {}", err),
                nom::Err::Failure(err) => panic!("Could not parse: {}", err),
                _ => panic!("Could not parse: {}", e),
            }
        }
    }
}

/// Assert not parse
#[track_caller]
fn anp<'a, T: std::fmt::Debug>(mut func: impl FnMut(Span<'a>) -> Res<'a, T>, input: &'static str) {
    let span = new_span(input);
    match func(span) {
        Err(_) => (),
        Ok(r) => {
            assert!(false, "Token should not match: {:?}", r);
        }
    }
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
    ap(parse_int, "0x0f", Int(15));
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
    ap(
        parse_string,
        r#""Reading JSON data from path '%s'.""#,
        GdString("Reading JSON data from path '%s'."),
    );
    ap(parse_string, r#""\n""#, GdString("\\n"));

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
    ap(parse_value, "hello", Value::ident("hello"));
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
        "[1, id , \"foo\", 3.0, 4.15]",
        Array(vec![
            Expr::value(Value::int(1)),
            Expr::value(Value::ident("id")),
            Expr::value(Value::string("foo")),
            Expr::value(Value::float(3.0)),
            Expr::value(Value::float(4.15)),
        ]),
    );
    ap(parse_array, "[]", Array(vec![]));
    ap(parse_array, "[ ]", Array(vec![]));

    ap(
        parse_array,
        indoc! { r#"[
            a,

            b
        ]"#},
        Array(vec![
            Expr::value(Value::ident("a")),
            Expr::value(Value::ident("b")),
        ]),
    );

    ap(
        parse_array,
        indoc! { r#"[
            # one
            # two
            a,

            # two
            b
            # two
        ]"#},
        Array(vec![
            Expr::value(Value::ident("a")),
            Expr::value(Value::ident("b")),
        ]),
    );

    apn(
        parse_array,
        indoc! { r#"[
            _rand_u8(), _rand_u8(), _rand_u8(), _rand_u8(),
            _rand_u8(), _rand_u8(), ((_rand_u8()) & 0x0f) | 0x40, _rand_u8(),
            ((_rand_u8()) & 0x3f) | 0x80, _rand_u8(), _rand_u8(), _rand_u8(),
            _rand_u8(), _rand_u8(), _rand_u8(), _rand_u8(),
        ]"#},
    );

    anp(parse_array, "[1,, ]");
    anp(parse_array, "[1,,");
    anp(parse_array, "1,,]");
}

#[test]
fn test_pair() {
    ap(
        parse_pair,
        "12: 'hello'",
        Pair(
            Expr::value(Value::int(12)),
            Expr::value(Value::string("hello")),
        ),
    );
    ap(
        parse_pair,
        "'a' : b",
        Pair(
            Expr::value(Value::string("a")),
            Expr::value(Value::ident("b")),
        ),
    );
}

#[test]
fn test_object() {
    ap(parse_object, "{}", Object(vec![]));
    ap(parse_object, "{ }", Object(vec![]));
    ap(
        parse_object,
        "{'a': 1}",
        Object(vec![Pair(
            Expr::value(Value::String(GdString("a"))),
            Expr::value(Value::int(1)),
        )]),
    );
    ap(
        parse_object,
        "{'a': 1, 2.0: id}",
        Object(vec![
            Pair(Expr::value(Value::string("a")), Expr::value(Value::int(1))),
            Pair(
                Expr::value(Value::float(2.0)),
                Expr::value(Value::ident("id")),
            ),
        ]),
    );
    ap(
        parse_object,
        "{1: [2, 3]\n}",
        Object(vec![Pair(
            Expr::value(Value::int(1)),
            Expr::value(Value::array(vec![
                Expr::value(Value::int(2)),
                Expr::value(Value::int(3)),
            ])),
        )]),
    );
    ap(
        parse_object,
        indoc! {r#"{
            "loggers": {}
        }"#},
        Object(vec![Pair(
            Expr::value(Value::string("loggers")),
            Expr::value(Value::object(vec![])),
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
            set_func: None,
            get_func: None,
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
            set_func: None,
            get_func: None,
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
            set_func: None,
            get_func: None,
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
            set_func: None,
            get_func: None,
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
            set_func: None,
            get_func: None,
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
            set_func: None,
            get_func: None,
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
            set_func: None,
            get_func: None,
        },
    );
    ap(
        parse_var_decl,
        "onready    var   hello  := 5+ 5 setget foo",
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
            set_func: Some("foo"),
            get_func: None,
        },
    );
    ap(
        parse_var_decl,
        "onready    var   hello  := 5+ 5 setget ,foo",
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
            set_func: None,
            get_func: Some("foo"),
        },
    );
    ap(
        parse_var_decl,
        "onready    var   hello  := 5+ 5 setget foo,bar",
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
            set_func: Some("foo"),
            get_func: Some("bar"),
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
    ap(
        |i| parse_decl(i, 0),
        "class_name Foo",
        Decl::ClassName(ClassNameDecl("Foo")),
    );

    anp(|i| parse_decl(i, 0), "foo");
}

#[test]
fn test_line() {
    ap(
        |i| parse_line(i, 0),
        "class_name Foo",
        Line::Decl(Decl::ClassName(ClassNameDecl("Foo"))),
    );
    ap(
        |i| parse_line(i, 0),
        "class_name Foo# Hey !",
        Line::Decl(Decl::ClassName(ClassNameDecl("Foo"))),
    );
    ap(
        |i| parse_line(i, 0),
        "class_name Foo;",
        Line::Decl(Decl::ClassName(ClassNameDecl("Foo"))),
    );
}

#[test]
fn test_block() {
    ap(
        |i| parse_block(i, 0),
        indoc! {r#"
            # Hey!
            class_name Foo
            extends "res://Bar.gd""#},
        Block(vec![
            Line::Comment(Comment("Hey!")),
            Line::Decl(Decl::ClassName(ClassNameDecl("Foo"))),
            Line::Decl(Decl::Extends(ExtendsDecl("res://Bar.gd"))),
        ]),
    );

    ap(
        |i| parse_block(i, 0),
        indoc! {r#"
            abcd

            abcd"#},
        Block(vec![
            Line::Expr(Expr::value(Value::ident("abcd"))),
            Line::Expr(Expr::Value(Value::ident("abcd"))),
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

    ap(
        parse_file,
        indoc! {r#"
        func _on_area_detector_body_entered(body: PhysicsBody2D) -> void:
            if body is Bullet:
                var bullet := body as Bullet
                if bullet.hurt_player:
                    bullet.destroy()
                    kill()
        "#},
        Block::new_line(Line::Decl(Decl::Function(FunctionDecl::new(
            "_on_area_detector_body_entered",
            vec![FunctionArg::new_typed("body", "PhysicsBody2D")],
            Some(DottedIdent("void")),
            Block::new_line(Line::Stmt(Stmt::If(IfStmt::new(
                Expr::bin(
                    Expr::value(Value::ident("body")),
                    BinOp::Is,
                    Expr::value(Value::ident("Bullet")),
                ),
                Block::new_line(Line::Decl(Decl::Var(
                    VarDecl::new("bullet")
                        .with_infer(true)
                        .with_value(Expr::bin(
                            Expr::value(Value::ident("body")),
                            BinOp::As,
                            Expr::value(Value::ident("Bullet")),
                        )),
                )))
                .with_line(Line::Stmt(Stmt::If(IfStmt::new(
                    Expr::bin(
                        Expr::value(Value::ident("bullet")),
                        BinOp::Attr,
                        Expr::value(Value::ident("hurt_player")),
                    ),
                    Block::new_line(Line::Expr(Expr::bin(
                        Expr::value(Value::ident("bullet")),
                        BinOp::Attr,
                        Expr::value(Value::func_call(FunctionCall::new("destroy"))),
                    )))
                    .with_line(Line::Expr(Expr::value(Value::func_call(
                        FunctionCall::new("kill"),
                    )))),
                )))),
            )))),
        )))),
    )
}

#[test]
fn test_indent() {
    // Level 0
    assert!(same_indent(new_span("ok"), 0).is_ok());
    assert!(more_indent(new_span("ok"), 0).is_err());

    // Level 1
    assert!(more_indent(new_span("  ok"), 0).is_ok());
    assert!(more_indent(new_span("  ok"), 2).is_err());
    assert!(same_indent(new_span("  ok"), 2).is_ok());

    // Level 2
    assert!(same_indent(new_span("ok"), 0).is_ok());
    assert!(more_indent(new_span("ok"), 0).is_err());
}

#[test]
fn test_expr() {
    ap(parse_expr, "123", Expr::Value(Value::Int(Int(123))));
    ap(
        parse_expr,
        "(a) + b",
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
    ap(
        parse_expr,
        "((_rand_u8()) & 0x0f)",
        Expr::bin(
            Expr::value(Value::func_call(FunctionCall::new("_rand_u8"))),
            BinOp::BinAnd,
            Expr::Value(Value::int(15)),
        ),
    );
    ap(
        parse_expr,
        r#"file_name in [".", ".."]"#,
        Expr::bin(
            Expr::value(Value::ident("file_name")),
            BinOp::In,
            Expr::value(Value::array(vec![
                Expr::value(Value::string(".")),
                Expr::value(Value::string("..")),
            ])),
        ),
    );
    ap(
        parse_expr,
        "a.b",
        Expr::bin(
            Expr::value(Value::ident("a")),
            BinOp::Attr,
            Expr::value(Value::ident("b")),
        ),
    );
    ap(
        parse_expr,
        "a.b[1].c",
        Expr::bin(
            Expr::Value(Value::ident("a")),
            BinOp::Attr,
            Expr::bin(
                Expr::bin(
                    Expr::value(Value::ident("b")),
                    BinOp::Index,
                    Expr::value(Value::int(1)),
                ),
                BinOp::Attr,
                Expr::value(Value::ident("c")),
            ),
        ),
    );
    ap(
        parse_expr,
        "a(123).abcd",
        Expr::bin(
            Expr::value(Value::func_call(
                FunctionCall::new("a").with_arg(Expr::value(Value::int(123))),
            )),
            BinOp::Attr,
            Expr::value(Value::ident("abcd")),
        ),
    );
    ap(
        parse_expr,
        "a(123)[1].abcd",
        Expr::bin(
            Expr::bin(
                Expr::value(Value::func_call(
                    FunctionCall::new("a").with_arg(Expr::value(Value::int(123))),
                )),
                BinOp::Index,
                Expr::value(Value::int(1)),
            ),
            BinOp::Attr,
            Expr::value(Value::ident("abcd")),
        ),
    );
    ap(
        parse_expr,
        "a(123)[idx].abcd",
        Expr::bin(
            Expr::bin(
                Expr::value(Value::func_call(
                    FunctionCall::new("a").with_arg(Expr::value(Value::int(123))),
                )),
                BinOp::Index,
                Expr::value(Value::ident("idx")),
            ),
            BinOp::Attr,
            Expr::value(Value::ident("abcd")),
        ),
    );
    ap(
        parse_expr,
        "abcd.efgh.ijkl()",
        Expr::bin(
            Expr::value(Value::ident("abcd")),
            BinOp::Attr,
            Expr::bin(
                Expr::value(Value::ident("efgh")),
                BinOp::Attr,
                Expr::value(Value::func_call(FunctionCall::new("ijkl"))),
            ),
        ),
    );
    ap(parse_expr, "a", Expr::value(Value::ident("a")));
    ap(
        parse_expr,
        "a()",
        Expr::value(Value::func_call(FunctionCall::new("a"))),
    );
    ap(
        parse_expr,
        "(a + b).add()",
        Expr::bin(
            Expr::bin(
                Expr::value(Value::ident("a")),
                BinOp::Add,
                Expr::value(Value::ident("b")),
            ),
            BinOp::Attr,
            Expr::value(Value::func_call(FunctionCall::new("add"))),
        ),
    );
    ap(
        parse_expr,
        "a[1][2]",
        Expr::bin(
            Expr::bin(
                Expr::value(Value::ident("a")),
                BinOp::Index,
                Expr::value(Value::int(1)),
            ),
            BinOp::Index,
            Expr::value(Value::int(2)),
        ),
    );
    ap(
        parse_expr,
        r#"logger.debug("Reading JSON data from path '%s'." % path)"#,
        Expr::bin(
            Expr::value(Value::ident("logger")),
            BinOp::Attr,
            Expr::value(Value::func_call(FunctionCall::new("debug").with_arg(
                Expr::bin(
                    Expr::value(Value::string("Reading JSON data from path '%s'.")),
                    BinOp::Mod,
                    Expr::value(Value::ident("path")),
                ),
            ))),
        ),
    );
}

#[test]
fn test_if_stmt() {
    ap(
        |i| parse_if_stmt(i, 0),
        indoc!(
            r#"
            if 123456:
                hello"#
        ),
        IfStmt {
            if_branch: Condition {
                expr: Expr::Value(Value::Int(Int(123456))),
                block: Block(vec![Line::Expr(Expr::Value(Value::ident("hello")))]),
            },
            elif_branches: vec![],
            else_branch: None,
        },
    );
    ap(
        |i| parse_if_stmt(i, 0),
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
                    Line::Expr(Expr::Value(Value::ident("hello"))),
                    Line::Expr(Expr::Value(Value::ident("hi"))),
                ]),
            },
            elif_branches: vec![],
            else_branch: None,
        },
    );
    ap(
        |i| parse_if_stmt(i, 0),
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
                    Line::Expr(Expr::Value(Value::ident("hello"))),
                    Line::Stmt(Stmt::If(IfStmt {
                        if_branch: Condition {
                            expr: Expr::Value(Value::Int(Int(123456))),
                            block: Block(vec![Line::Expr(Expr::Value(Value::ident("hi")))]),
                        },
                        elif_branches: vec![],
                        else_branch: None,
                    })),
                ]),
            },
            elif_branches: vec![],
            else_branch: None,
        },
    );
    ap(
        |i| parse_block(i, 0),
        indoc!(
            r#"
            if 123456:
                hello
            hi"#
        ),
        Block(vec![
            Line::Stmt(Stmt::If(IfStmt {
                if_branch: Condition {
                    expr: Expr::Value(Value::Int(Int(123456))),
                    block: Block(vec![Line::Expr(Expr::Value(Value::ident("hello")))]),
                },
                elif_branches: vec![],
                else_branch: None,
            })),
            Line::Expr(Expr::Value(Value::ident("hi"))),
        ]),
    );

    ap(
        |i| parse_block(i, 0),
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
            Line::Stmt(Stmt::If(IfStmt {
                if_branch: Condition {
                    expr: Expr::Value(Value::int(123456)),
                    block: Block(vec![Line::Expr(Expr::Value(Value::ident("hello")))]),
                },
                elif_branches: vec![ElifStmt(Condition {
                    expr: Expr::Value(Value::ident("a")),
                    block: Block(vec![Line::Expr(Expr::Value(Value::ident("bar")))]),
                })],
                else_branch: Some(ElseStmt(Block(vec![Line::Stmt(Stmt::If(IfStmt {
                    if_branch: Condition {
                        expr: Expr::Value(Value::ident("a")),
                        block: Block(vec![Line::Expr(Expr::Value(Value::ident("foo")))]),
                    },
                    elif_branches: vec![],
                    else_branch: Some(ElseStmt(Block(vec![Line::Expr(Expr::Value(
                        Value::ident("foo"),
                    ))]))),
                }))]))),
            })),
            Line::Expr(Expr::Value(Value::ident("hi"))),
        ]),
    );

    anp(
        |i| parse_if_stmt(i, 0),
        indoc!(
            r#"
            if 123456:
        "#
        ),
    );
    anp(
        |i| parse_if_stmt(i, 0),
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
    ap(
        |i| parse_function_decl(i, 0),
        indoc! {r#"
        func   foo  (bar  , baz  :  Node, qux = 1234 )  ->   Node:
            spam"#},
        FunctionDecl {
            modifier: None,
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
            block: Block(vec![Line::Expr(Expr::Value(Value::ident("spam")))]),
        },
    );

    ap(
        |i| parse_function_decl(i, 0),
        indoc! {r#"
        static func foo():
            pass"#},
        FunctionDecl {
            modifier: Some(FunctionModifier::Static),
            name: Ident("foo"),
            args: vec![],
            return_type: None,
            block: Block::new_line(Line::Stmt(Stmt::Pass(Pass))),
        },
    );

    ap(
        |i| parse_function_decl(i, 0),
        indoc! {r#"
        static func add_message(message: LogMessage) -> void:
            _static_data["messages"].append(message)"#},
        FunctionDecl::new(
            "add_message",
            vec![FunctionArg::new_typed("message", "LogMessage")],
            Some(DottedIdent("void")),
            Block::new_line(Line::Expr(Expr::bin(
                Expr::bin(
                    Expr::value(Value::ident("_static_data")),
                    BinOp::Index,
                    Expr::value(Value::string("messages")),
                ),
                BinOp::Attr,
                Expr::value(Value::func_call(
                    FunctionCall::new("append").with_arg(Expr::value(Value::ident("message"))),
                )),
            ))),
        )
        .with_modifier(FunctionModifier::Static),
    );
}

#[test]
fn test_while_stmt() {
    ap(
        |i| parse_while_stmt(i, 0),
        indoc! {r#"
        while true:
            pass"#},
        WhileStmt(Condition {
            expr: Expr::Value(Value::Boolean(Boolean(true))),
            block: Block(vec![Line::Stmt(Stmt::Pass(Pass))]),
        }),
    );
}

#[test]
fn test_for_stmt() {
    ap(
        |i| parse_for_stmt(i, 0),
        indoc! {r#"
        for x in array:
            pass"#},
        ForStmt(Condition::new(
            Expr::bin(
                Expr::value(Value::ident("x")),
                BinOp::In,
                Expr::Value(Value::ident("array")),
            ),
            Block(vec![Line::Stmt(Stmt::Pass(Pass))]),
        )),
    )
}

#[test]
fn test_match_stmt() {
    ap(
        |i| parse_match_stmt(i, 0),
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
                    block: Block(vec![Line::Stmt(Stmt::Pass(Pass))]),
                }),
                MatchCaseStmt(Condition {
                    expr: Expr::value(Value::ident("_")),
                    block: Block(vec![Line::Stmt(Stmt::Pass(Pass))]),
                }),
            ],
        },
    )
}

#[test]
fn test_class_decl() {
    ap(
        |i| parse_class_decl(i, 0),
        indoc! {r#"
            class Foo:
                pass"#},
        ClassDecl {
            name: Ident("Foo"),
            block: Block(vec![Line::Stmt(Stmt::Pass(Pass))]),
        },
    );

    ap(
        |i| parse_class_decl(i, 0),
        indoc! {r#"
        class _LogData:
            const _static_data := {"messages": [], "cursor": 0}

            static func get_messages() -> Array:
                return _static_data["messages"]

            static func add_message(message: LogMessage) -> void:
                _static_data["messages"].append(message)"#},
        ClassDecl {
            name: Ident("_LogData"),
            block: Block::new_line(Line::Decl(Decl::Const(
                ConstDecl::new(
                    "_static_data",
                    Expr::value(Value::object(vec![
                        Pair(
                            Expr::value(Value::string("messages")),
                            Expr::value(Value::array(vec![])),
                        ),
                        Pair(
                            Expr::value(Value::string("cursor")),
                            Expr::value(Value::int(0)),
                        ),
                    ])),
                )
                .with_infer(true),
            )))
            .with_line(Line::Decl(Decl::Function(
                FunctionDecl::new(
                    "get_messages",
                    vec![],
                    Some(DottedIdent("Array")),
                    Block::new_line(Line::Stmt(Stmt::Return(ReturnStmt(Expr::bin(
                        Expr::value(Value::ident("_static_data")),
                        BinOp::Index,
                        Expr::value(Value::string("messages")),
                    ))))),
                )
                .with_modifier(FunctionModifier::Static),
            )))
            .with_line(Line::Decl(Decl::Function(
                FunctionDecl::new(
                    "add_message",
                    vec![FunctionArg::new_typed("message", "LogMessage")],
                    Some(DottedIdent("void")),
                    Block::new_line(Line::Expr(Expr::bin(
                        Expr::bin(
                            Expr::value(Value::ident("_static_data")),
                            BinOp::Index,
                            Expr::value(Value::string("messages")),
                        ),
                        BinOp::Attr,
                        Expr::value(Value::func_call(
                            FunctionCall::new("append")
                                .with_arg(Expr::value(Value::ident("message"))),
                        )),
                    ))),
                )
                .with_modifier(FunctionModifier::Static),
            ))),
        },
    );
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
    ap(parse_function_call, "foo()", FunctionCall::new("foo"));
    ap(
        parse_function_call,
        "foo(a)",
        FunctionCall::new("foo").with_args(vec![Expr::value(Value::ident("a"))]),
    );
    ap(
        parse_function_call,
        "foo(a, 1234)",
        FunctionCall::new("foo").with_args(vec![
            Expr::value(Value::ident("a")),
            Expr::value(Value::int(1234)),
        ]),
    );
}

#[test]
fn test_assign_stmt() {
    ap(
        parse_assign_stmt,
        "a = 5",
        AssignStmt {
            source: Expr::value(Value::ident("a")),
            op: AssignOp::Assign,
            value: Expr::value(Value::int(5)),
        },
    );
    ap(
        parse_assign_stmt,
        "a.b.c += 5 * 2",
        AssignStmt {
            source: Expr::bin(
                Expr::value(Value::ident("a")),
                BinOp::Attr,
                Expr::bin(
                    Expr::value(Value::ident("b")),
                    BinOp::Attr,
                    Expr::value(Value::ident("c")),
                ),
            ),
            op: AssignOp::AssignAdd,
            value: Expr::bin(
                Expr::value(Value::int(5)),
                BinOp::Mul,
                Expr::value(Value::int(2)),
            ),
        },
    );
    ap(
        parse_assign_stmt,
        r#"_scene_name.text = str(entry_idx) + " - " + entry_name + "\n" + str(entry_idx) + "/" + str(len(_known_scenes))"#,
        AssignStmt {
            source: Expr::bin(
                Expr::value(Value::ident("_scene_name")),
                BinOp::Attr,
                Expr::value(Value::ident("text")),
            ),
            op: AssignOp::Assign,
            value: Expr::bin(
                Expr::value(Value::func_call(
                    FunctionCall::new("str").with_arg(Expr::value(Value::ident("entry_idx"))),
                )),
                BinOp::Add,
                Expr::bin(
                    Expr::value(Value::string(" - ")),
                    BinOp::Add,
                    Expr::bin(
                        Expr::value(Value::ident("entry_name")),
                        BinOp::Add,
                        Expr::bin(
                            Expr::value(Value::string("\\n")),
                            BinOp::Add,
                            Expr::bin(
                                Expr::value(Value::func_call(
                                    FunctionCall::new("str")
                                        .with_arg(Expr::value(Value::ident("entry_idx"))),
                                )),
                                BinOp::Add,
                                Expr::bin(
                                    Expr::value(Value::string("/")),
                                    BinOp::Add,
                                    Expr::value(Value::func_call(
                                        FunctionCall::new("str").with_arg(Expr::value(
                                            Value::func_call(FunctionCall::new("len").with_arg(
                                                Expr::value(Value::ident("_known_scenes")),
                                            )),
                                        )),
                                    )),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        },
    )
}

#[test]
fn test_enum_decl() {
    ap(
        parse_enum_decl,
        indoc! {r#"
            enum Test {
                A,
                B,
                C = 3
            }"#},
        EnumDecl::new(
            "Test",
            vec![
                EnumVariant::new("A"),
                EnumVariant::new("B"),
                EnumVariant::new("C").with_value(Value::int(3)),
            ],
        ),
    );
}

#[test]
fn test_const_decl() {
    ap(
        parse_const_decl,
        "const FOO = 5",
        ConstDecl::new("FOO", Expr::value(Value::int(5))),
    );
    ap(
        parse_const_decl,
        "const FOO := 5",
        ConstDecl::new("FOO", Expr::value(Value::int(5))).with_infer(true),
    );
    ap(
        parse_const_decl,
        "const FOO: int = 5",
        ConstDecl::new("FOO", Expr::value(Value::int(5))).with_type("int"),
    );

    ap(
        parse_const_decl,
        indoc! {r#"
            const _static_data := {
                "loggers": {},
            }"#},
        ConstDecl::new(
            "_static_data",
            Expr::value(Value::object(vec![Pair(
                Expr::value(Value::string("loggers")),
                Expr::value(Value::object(vec![])),
            )])),
        )
        .with_infer(true),
    );

    anp(parse_const_decl, "const FOO");
}

#[test]
fn test_return_stmt() {
    ap(
        parse_return_stmt,
        "return null",
        ReturnStmt(Expr::value(Value::Null(Null))),
    );
    ap(
        parse_return_stmt,
        indoc! {r#"
            return Color(
                255,
                255,
                255,
                0
            )"#},
        ReturnStmt(Expr::Value(Value::func_call(
            FunctionCall::new("Color")
                .with_arg(Expr::value(Value::int(255)))
                .with_arg(Expr::value(Value::int(255)))
                .with_arg(Expr::value(Value::int(255)))
                .with_arg(Expr::value(Value::int(0))),
        ))),
    );
    ap(
        parse_return_stmt,
        indoc! {r#"
            return "[{time}] [{level_str}] [{name}] {args}".format({
                "time": "%0.3f" % time,
                "level_str": level_str,
                "name": name,
                "args": _format_args(message, args)
            })"#},
        ReturnStmt(Expr::bin(
            Expr::value(Value::string("[{time}] [{level_str}] [{name}] {args}")),
            BinOp::Attr,
            Expr::value(Value::func_call(FunctionCall::new("format").with_arg(
                Expr::value(Value::object(vec![
                        Pair(
                            Expr::value(Value::string("time")),
                            Expr::bin(
                                Expr::value(Value::string("%0.3f")),
                                BinOp::Mod,
                                Expr::value(Value::ident("time")),
                            ),
                        ),
                        Pair(
                            Expr::value(Value::string("level_str")),
                            Expr::value(Value::ident("level_str")),
                        ),
                        Pair(Expr::value(Value::string("name")), Expr::value(Value::ident("name"))),
                        Pair(
                            Expr::value(Value::string("args")),
                            Expr::value(Value::func_call(
                                    FunctionCall::new("_format_args")
                                        .with_arg(Expr::value(Value::ident("message")))
                                        .with_arg(Expr::value(Value::ident("args"))),
                            )),
                        ),
                    ])),
            ))),
        )),
    );
}

#[test]
fn test_sample() {
    apn(parse_file, include_str!("tests/simple.gd"));
    apn(parse_file, include_str!("tests/simple2.gd"));
    apn(parse_file, include_str!("tests/Player.gd"));
}
