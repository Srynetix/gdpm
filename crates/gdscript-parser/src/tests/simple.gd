# Hello
extends Node
class_name Simple

var simple
var simple_with_value = 1 + simple
var simple_with_type: float
var simple_with_null: Node = null
var simple_with_infer := 5.0
var simple_with_infer := (1 + 2) + 3
# Hello

onready var _simple_node = $Simple
onready var _simple_node_2 = $Simple/Child
onready var _simple_node_type: Node = $Simple/Child2
onready var _simple_node_infer := $Simple/Child3 as Node
# Hello

# Hello
func _ready() -> void:
    # Hello
    pass

func _ready_no_type():
    pass
    # Hello

func _process(delta: float) -> void:
    var inner = 1
    # Hello
    var inner2 = 2

func _process_no_type(delta):
    1 + 1
    var inner = 1
    var inner2 = 2
    _static()
    var toto = _static(1, 2)
    toto.tutu(1).hello().hello
    var toto = toto.tutu
    var toto = toto.tutu.tata
    var a = toto.tutu(5) + 1
    _static().pouet
    _static().pouet = 5
    toto = 5
    toto += 5
    toto /= 5
    toto %= 5
    toto *= 5 + 1 + _static()

static func _static() -> void:
    pass