extends KinematicBody2D
class_name Player

signal exit()
signal dead()

var bullet_target: Node
var detect_input := true

var _gravity := Vector2(0, 25)
var _max_velocity := 300.0
var _jump_speed = 550.0
var _movement_speed := 25.0
var _friction_value := 0.85
var _max_jumps := 2
var _acceleration := Vector2.ZERO
var _velocity := Vector2.ZERO
var _current_jumps := 0
var _is_on_ice := false
var _dead := false

onready var gun := $Gun as Node2D
onready var muzzle := $Gun/Muzzle as Position2D
onready var gun_sprite := $Gun/Sprite as Sprite
onready var sprite := $Sprite as Sprite
onready var area_detector := $AreaDetector as Area2D
onready var area_detector_collision_shape := $AreaDetector/CollisionShape2D as CollisionShape2D
onready var jump_fx := $JumpFX as AudioStreamPlayer
onready var animation_player := $AnimationPlayer as AnimationPlayer
onready var jump_particles := $JumpParticles as CPUParticles2D
onready var collision_shape := $CollisionShape2D as CollisionShape2D

var _last_touch_index = -1
var _gun_angle = 0

func _ready() -> void:
    area_detector.connect("area_entered", self, "_on_area_detector_area_entered")
    area_detector.connect("body_entered", self, "_on_area_detector_body_entered")

func _input(event: InputEvent):
    if !SxOS.is_mobile():
        if event is InputEventMouseMotion:
            var motion_event := make_input_local(event) as InputEventMouseMotion
            _gun_angle = motion_event.position.angle()

    else:
        if event is InputEventScreenDrag:
            var drag_event := make_input_local(event) as InputEventScreenDrag
            if drag_event.index == _last_touch_index:
                _gun_angle = drag_event.position.angle()

        elif event is InputEventScreenTouch:
            var touch_event := make_input_local(event) as InputEventScreenTouch
            if touch_event.index == _last_touch_index && !touch_event.pressed:
                _last_touch_index = -1
                _gun_angle = touch_event.position.angle()
                Input.action_press("fire")
            elif _last_touch_index == -1 && touch_event.pressed:
                _last_touch_index = touch_event.index
                _gun_angle = touch_event.position.angle()

func _process(_delta: float) -> void:
    if detect_input:
        # Move gun with mouse
        gun.rotation = _gun_angle

        # Flip sprite depending on rotation
        if gun.rotation_degrees > -90 && gun.rotation_degrees < 90:
            sprite.flip_h = false
            gun.scale = Vector2(1, 1)
        else:
            sprite.flip_h = true
            gun.scale = Vector2(1, -1)

        # Fire!
        if Input.is_action_just_pressed("fire"):
            var bullet := GameLoadCache.instantiate_scene("Bullet") as Bullet
            var trajectory = Vector2.RIGHT.rotated(gun.rotation)
            var bullet_initial_velocity = trajectory * 200
            bullet.position = muzzle.global_position
            bullet.initial_velocity = bullet_initial_velocity

            if bullet_target != null:
                bullet_target.add_child(bullet)
            else:
                get_parent().add_child(bullet)

func _physics_process(_delta) -> void:
    _acceleration = Vector2.ZERO
    _apply_input()
    _apply_gravity()

    _velocity += _acceleration
    _clamp_velocity()

    _velocity = move_and_slide(_velocity, Vector2.UP, true)

    _is_on_ice = false
    for idx in range(get_slide_count()):
        var collision = get_slide_collision(idx)
        var collider = collision.collider
        if collider is TileMap:
            var tilemap := collider as TileMap
            var coord = tilemap.world_to_map(collision.position - collision.normal) / tilemap.scale
            var tile_id = tilemap.get_cellv(coord)
            if tile_id != -1:
                var name = tilemap.tile_set.tile_get_name(tile_id)
                if name == "ice":
                    _is_on_ice = true

    if is_on_floor():
        _current_jumps = 0

func kill() -> void:
    if !_dead:
        _dead = true
        set_physics_process(false)
        emit_signal("dead")

func exit() -> void:
    collision_shape.set_deferred("disabled", true)
    area_detector_collision_shape.set_deferred("disabled", true)
    animation_player.play("fade")
    set_physics_process(false)
    emit_signal("exit")

func _apply_input() -> void:
    var side_direction = 0

    if detect_input:
        if Input.is_action_pressed("move_left"):
            side_direction -= 1
        if Input.is_action_pressed("move_right"):
            side_direction += 1

        if Input.is_action_just_pressed("jump"):
            # Jump mid-air
            if !is_on_floor():
                if _current_jumps == 0:
                    # Force one jump
                    _current_jumps += 1

            if _current_jumps < _max_jumps:
                _acceleration += Vector2.UP * _jump_speed + (Vector2.UP * _velocity.y)
                _current_jumps += 1

                jump_fx.pitch_scale = rand_range(0.95, 1.05)
                jump_fx.play()

                jump_particles.restart()
                jump_particles.emitting = true
        elif Input.is_action_just_released("jump"):
            if _velocity.y < 0:
                _acceleration += Vector2.UP * _velocity.y / 2

    if side_direction == 0:
        if !_is_on_ice:
            _apply_side_friction()
        animation_player.play("idle")
    else:
        _acceleration += Vector2.RIGHT * side_direction * _movement_speed
        animation_player.play("run")

func _apply_side_friction() -> void:
    _acceleration += Vector2(lerp(-_velocity.x, 0, 1 - _friction_value), 0)

func _apply_gravity() -> void:
    _acceleration += _gravity

func _clamp_velocity() -> void:
    _velocity.x = clamp(_velocity.x, -_max_velocity, _max_velocity)

func _on_area_detector_area_entered(area: Area2D) -> void:
    if area is ExitDoor:
        var door := area as ExitDoor
        if door.is_exit && door.opened:
            exit()

    elif area is Spikes:
        kill()

    elif area is PushButton:
        var btn := area as PushButton
        btn.press()
    pass

func _on_area_detector_body_entered(body: PhysicsBody2D) -> void:
    if body is Bullet:
        var bullet := body as Bullet
        if bullet.hurt_player:
            bullet.destroy()
            kill()
