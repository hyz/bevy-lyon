use bevy::app::AppExit; // For MacOS CMD+W to quit keybind
use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::window::*;
use bevy_prototype_debug_lines::DebugLines;
use bevy_prototype_debug_lines::DebugLinesPlugin;

use bevy_prototype_lyon as lyon;
use bevy_prototype_lyon::prelude::*; // Draw circles with ease
use std::env; // Detect OS for OS specific keybinds

const TIME_STEP: f32 = 1.0 / 120.0;

fn main() {
    App::new()
        .add_startup_system(setup_camera)
        // .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Tiny Tank (bevy edition)".to_string(),
            width: 800.,
            height: 600.,
            vsync: true,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.3, 0.35, 0.51)))
        .add_plugins(DefaultPlugins)
        .add_plugin(lyon::plugin::ShapePlugin)
        .add_plugin(DebugLinesPlugin::default()) // with_depth_test(true)
        .add_startup_system(debuglines_sys)
        .add_startup_system(create_player)
        .add_system(toggle_fullscreen_f11)
        .add_system(mouse_button_input)
        .add_system(kill_bullets)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(update_bullets)
                .with_system(movement),
        )
        .run();
}

fn debuglines_sys(mut lines: ResMut<DebugLines>) {
    lines.line_colored(
        Vec3::new(-400.0, 0.0, 0.0),
        Vec3::new(400.0, 0.0, 0.0),
        9.9,
        Color::GREEN,
    );
    lines.line_gradient(
        Vec3::new(-100.0, 100.0, 0.0),
        Vec3::new(100.0, -100.0, 0.0),
        6.8,
        Color::WHITE,
        Color::PINK,
    );
    lines.line_gradient(
        Vec3::new(-100.0, -100.0, 0.0),
        Vec3::new(100.0, 100.0, 0.0),
        4.3,
        Color::MIDNIGHT_BLUE,
        Color::YELLOW_GREEN,
    );
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    println!("{}", env::consts::OS); // Prints the current OS.
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Turret;

fn create_player(mut commands: Commands) {
    commands
        .spawn_bundle(lyon::geometry::GeometryBuilder::build_as(
            &lyon::shapes::RegularPolygon {
                sides: 30,
                feature: lyon::shapes::RegularPolygonFeature::Radius(20.0), // Define circle
                ..lyon::shapes::RegularPolygon::default()
            },
            lyon::draw::DrawMode::Outlined {
                fill_mode: lyon::draw::FillMode::color(Color::rgb(0.35, 0.6, 0.99)),
                outline_mode: lyon::draw::StrokeMode::new(Color::BLACK, 4.0),
            },
            Transform {
                translation: Vec3::new(0.0, 0.0, 1.0),
                ..Default::default()
            },
        ))
        .insert(Player)
        .insert(Velocity(Vec2::ZERO))
        .with_children(|parent| {
            // Add turret to player
            parent
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::BLACK,
                        ..Default::default()
                    },
                    transform: Transform {
                        scale: Vec3::new(16.0, 16.0, 0.),
                        translation: Vec3::new(24.0, 0.0, 0.5),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Turret);
        });
}

fn movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    for (mut transform, mut velocity) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            velocity.0.x -= 0.37;
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            velocity.0.x += 0.37;
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            velocity.0.y -= 0.37;
        }
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            velocity.0.y += 0.37;
        }

        velocity.0 *= 0.9;

        transform.translation += velocity.0.extend(0.);
    }
}

fn toggle_fullscreen_f11(
    keyboard_input: Res<Input<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();

    if env::consts::OS == "macos" {
        if keyboard_input.pressed(KeyCode::LWin) && keyboard_input.just_pressed(KeyCode::W) {
            exit.send(AppExit);
            window.set_mode(WindowMode::Windowed);
        }
        if keyboard_input.pressed(KeyCode::LWin)
            && keyboard_input.pressed(KeyCode::LControl)
            && keyboard_input.just_pressed(KeyCode::F)
        {
            println!("{:?}", window.mode());
            if window.mode() == WindowMode::Windowed {
                window.set_mode(WindowMode::BorderlessFullscreen);
            } else if window.mode() == WindowMode::BorderlessFullscreen {
                window.set_mode(WindowMode::Windowed);
            }
        }
    }
    if env::consts::OS == "windows" {
        if keyboard_input.just_pressed(KeyCode::F11) {
            if window.mode() == WindowMode::Windowed {
                window.set_mode(WindowMode::BorderlessFullscreen);
            } else if window.mode() == WindowMode::BorderlessFullscreen {
                window.set_mode(WindowMode::Windowed);
            }
        }
    }
}

fn debugcross_xy(lines: &mut ResMut<DebugLines>, p: Vec2) {
    let a = p + Vec2::new(-15., -15.);
    let b = p + Vec2::new(15., 15.);
    lines.line_colored(a.extend(0.), b.extend(0.), 0.9, Color::RED);
    let a = p + Vec2::new(15., -15.);
    let b = p + Vec2::new(-15., 15.);
    lines.line_colored(a.extend(0.), b.extend(0.), 1.2, Color::RED);
}

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Direction(Vec3);

fn mouse_button_input(
    // Shoot bullets and rotate turret to point at mouse
    buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut lines: ResMut<DebugLines>,
    mut commands: Commands,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let window = windows.get_primary().unwrap();
    if let Some(cursor_pos_) = window.cursor_position() {
        for mut transform in query.iter_mut() {
            let cursor_pos = cursor_pos_ + Vec2::new(window.width(), window.height()) / -2.;
            let diff = cursor_pos.extend(0.) - transform.translation;
            let angle = diff.y.atan2(diff.x); // Add/sub FRAC_PI here optionally

            transform.rotation = Quat::from_rotation_z(angle);

            if buttons.just_pressed(MouseButton::Left) {
                debugcross_xy(&mut lines, cursor_pos);
                println!(
                    "cursor:{} translation:{}",
                    cursor_pos_, transform.translation
                );
                commands
                    .spawn_bundle(GeometryBuilder::build_as(
                        &shapes::RegularPolygon {
                            sides: 30,
                            feature: shapes::RegularPolygonFeature::Radius(6.0),
                            ..shapes::RegularPolygon::default()
                        },
                        DrawMode::Fill(FillMode::color(Color::BLACK)),
                        Transform {
                            translation: transform.translation, //Vec3::new(trans.translation.x, trans.translation.y, 0.0),
                            ..Default::default()
                        },
                    ))
                    .insert(Bullet)
                    .insert(Direction(diff.normalize()));
            }
        }
    }
}

fn update_bullets(mut query: Query<(&mut Transform, &Direction), With<Bullet>>) {
    for (mut transform, direction) in query.iter_mut() {
        transform.translation += direction.0 * 10.;
    }
}

fn kill_bullets(
    mut commands: Commands,
    mut query: Query<((&mut Transform, Entity), With<Bullet>)>,
    windows: Res<Windows>,
) {
    let window = windows.get_primary().unwrap();

    for ((transform, bullet_entity), _bullet) in query.iter_mut() {
        if transform.translation.x.abs() > window.width() / 2.
            || transform.translation.y.abs() > window.height() / 2.
        {
            commands.entity(bullet_entity).despawn();
        }
    }
}
