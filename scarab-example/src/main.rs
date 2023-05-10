use std::collections::HashMap;

use debug::{DebugOptions, FieldDebug};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::{ButtonState, EventSettings, Key, Window, WindowSettings};

use scarab_engine::{
    gameobject::{
        field::{Cell, CellColorView, Field, FieldColorView},
        Entity, NO_SOLIDITY, SOLID,
    },
    input::{ButtonBinding, LogicalDpad, SingleButton, VirtualDpad},
    rendering::{
        debug::StandardAndDebugView,
        registry::TextureRegistry,
        sprite::{AnimationStateMachine, SpriteAnimation},
    },
    App, Axis, Camera, GlutinWindow, HasBoxMut, LogicalSize, PhysBox, ScarabResult, Scene,
};

mod app;
mod debug;
mod entities;
mod external_serde;
mod inputs;
use app::ExampleApp;
use entities::{Enemy, EntityDebug, ExampleEntities, Player, PlayerAnimations};
use inputs::Inputs;

const MS_PER_FRAME: f64 = 1000.0 / 15.0;

fn main() -> ScarabResult<()> {
    let camera_size = [640, 360];
    let opengl = OpenGL::V3_2;
    let window: GlutinWindow = WindowSettings::new("scarab-example", camera_size)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap(); // TODO! log a more readable message before panicking
    let gl = GlGraphics::new(opengl);
    window
        .ctx
        .window()
        .set_min_inner_size(Some(LogicalSize::<u32>::from(camera_size)));

    // Manually construct the field for testing
    let cell0 = Cell::new(SOLID, PhysBox::new([0.0, 0.0, 50.0, 100.0])?);
    let cell1 = Cell::new(SOLID, PhysBox::new([50.0, 50.0, 100.0, 150.0])?);
    let cell2 = Cell::new(NO_SOLIDITY, PhysBox::new([50.0, 0.0, 590.0, 50.0])?);
    let cell3 = Cell::new(NO_SOLIDITY, PhysBox::new([150.0, 50.0, 490.0, 310.0])?);
    let cell4 = Cell::new(NO_SOLIDITY, PhysBox::new([0.0, 200.0, 150.0, 160.0])?);
    let cell5 = Cell::new(NO_SOLIDITY, PhysBox::new([0.0, 100.0, 50.0, 100.0])?);
    let cell6 = Cell::new(SOLID, PhysBox::new([640.0, 0.0, 1.0, 360.0])?);
    let cell7 = Cell::new(SOLID, PhysBox::new([0.0, 360.0, 640.0, 1.0])?);
    let cell8 = Cell::new(SOLID, PhysBox::new([0.0, -1.0, 640.0, 1.0])?);
    let cell9 = Cell::new(SOLID, PhysBox::new([-1.0, 0.0, 1.0, 360.0])?);

    let field = Field::new(vec![
        cell0, cell1, cell2, cell3, cell4, cell5, cell6, cell7, cell8, cell9,
    ])?;

    // The field view has solid cells as black and NO_SOLIDITY cells as white,
    // and other cells as grey
    let field_view = FieldColorView {
        solid_view: CellColorView {
            color: [0.0, 0.0, 0.0, 1.0],
        },
        air_view: CellColorView { color: [1.0; 4] },
        default_view: CellColorView {
            color: [0.5, 0.5, 0.5, 1.0],
        },
    };

    // Put the field and its view in the scene
    let mut scene = Scene::new(
        field,
        StandardAndDebugView::from((field_view, FieldDebug {})),
    );

    // Create a camera with a 100x100 tile view
    let cambox = PhysBox::new([0.0, 0.0, camera_size[0].into(), camera_size[1].into()])?;
    let camera = Camera::new(cambox, window.size().into());

    let texture_registry = TextureRegistry::new(
        // This ends up being the path from cwd to the assets. It has to change depending on deployment
        "scarab-example/assets".into(),
        "texture-default.png".into(),
        &[
            "RH-idle-front.png".into(),
            "RH-run-front.png".into(),
            "RH-run-front-color-dodged.png".into(),
        ],
    )?;

    let player_idle = SpriteAnimation::new(
        [54.0, 67.0].into(),
        [128.0, 128.0].into(),
        "RH-idle-front.png".into(),
        MS_PER_FRAME,
        Axis::X,
        None,
        &texture_registry,
    )?;

    let player_run = SpriteAnimation::new(
        [54.0, 67.0].into(),
        [128.0, 128.0].into(),
        "RH-run-front.png".into(),
        MS_PER_FRAME,
        Axis::X,
        None,
        &texture_registry,
    )?;
    let mut player_animation_states = HashMap::new();
    player_animation_states.insert(PlayerAnimations::Idle, player_idle);
    player_animation_states.insert(PlayerAnimations::Run, player_run);
    let player_view = AnimationStateMachine::new(PlayerAnimations::Idle, player_animation_states)?;

    let enemy_view = AnimationStateMachine::static_animation(SpriteAnimation::new(
        [56.0, 70.0].into(),
        [128.0, 128.0].into(),
        "RH-run-front-color-dodged.png".into(),
        MS_PER_FRAME,
        Axis::X,
        None,
        &texture_registry,
    )?);

    // Create the player setting its position, size and max speed
    let mut p = Entity::new()?;
    let b = p.get_box_mut();
    b.set_pos([310.0, 170.0].into());
    b.set_size([20.0, 20.0].into())?;
    p.set_max_velocity(75.0)?;
    let player = Player::new(p, 2.0, 1.0);

    // Create the enemy setting its position, size and max speed
    let mut r = Entity::new()?;
    let b = r.get_box_mut();
    b.set_pos([180.0, 230.0].into());
    b.set_size([15.0, 15.0].into())?;
    r.set_max_velocity(50.0)?;

    let enemy = Enemy { entity: r };

    // Create the second enemy setting its position, size and max speed
    let mut r = Entity::new()?;
    let b = r.get_box_mut();
    b.set_pos([180.0, 120.0].into());
    b.set_size([15.0, 15.0].into())?;
    r.set_max_velocity(50.0)?;

    let enemy2 = Enemy { entity: r };

    let entity_debug = EntityDebug {
        box_color: [0.0, 1.0, 1.0, 1.0],
        health_color: [1.0, 0.0, 0.0, 1.0],
    };

    scene.register_entity(ExampleEntities::Player((
        player,
        (player_view, entity_debug.clone()).into(),
    )))?;
    scene.register_entity(ExampleEntities::Enemy((
        enemy,
        (enemy_view.clone(), entity_debug.clone()).into(),
    )))?;
    scene.register_entity(ExampleEntities::Enemy((
        enemy2,
        (enemy_view, entity_debug).into(),
    )))?;

    // Use WASD inputs (reminder that up is negative y)
    let mut input_registry = Inputs::new();
    input_registry.bind_movement(
        LogicalDpad::from(VirtualDpad::new(
            SingleButton::Keyboard(Key::D),
            SingleButton::Keyboard(Key::S),
            SingleButton::Keyboard(Key::A),
            SingleButton::Keyboard(Key::W),
        ))
        .into(),
    );
    input_registry.bind_attack(ButtonBinding::new(
        ButtonState::Press,
        SingleButton::Mouse(piston::MouseButton::Left),
    ));

    // NOTE: All of the above code is reponsible for initializing the game state
    // the first time the app is run. After that you can comment out all of the above
    // and simply swap the two 'app' initilization statements below to allow the
    // game state to persist between runs.
    let save_name = "scarab-example.dat".to_string();
    let mut event_settings = EventSettings::new();
    event_settings.max_fps = 60;
    event_settings.ups = 60;
    let app = ExampleApp::new(
        gl,
        window,
        scene,
        camera,
        input_registry,
        DebugOptions {
            entity_collision_boxes: true,
            entity_health: true,
            field_collision_boxes: true,
            attack_cooldowns: true,
        },
        save_name,
        event_settings,
        texture_registry,
    )
    .unwrap();
    // let app = ExampleApp::<ExampleEntities, FieldColorView, Inputs>::load_from_save(
    //     OpenGL::V3_2,
    //     save_name,
    // )
    // .unwrap();

    Box::new(app).run();
    Ok(())
}
