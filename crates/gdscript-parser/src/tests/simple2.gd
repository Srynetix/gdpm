# Hello !

extends Node
class_name pouet

enum Test {
    A,
    B,
    C = 3
}

enum Line { A, B, C }

const FOO = 4

signal toto
signal toto(a)
signal toto(a, b)

var toto
var toto = 1
var toto: int = 1
var toto := 1
var toto := 1.0
var toto := "hello"
var toto := "hello" as String
var toto := 1 + 2
var toto := (1 + 2) + 3
var toto := ((1 + 2) + 3)

func _ready() -> void:
    if toto:
        print("ok!")
    else:
        pass

    while toto:
        if cond == 5:
            if cond:
                pass
            elif cond:
                pass
            else:
                pass
        elif cond:
            pass

func _yield_test() -> void:
    yield(get_tree(), "ready")