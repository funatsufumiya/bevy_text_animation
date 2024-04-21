use bevy::prelude::*;
use bevy_text_animation::{TextAnimationFinished, TextAnimatorPlugin, TextSimpleAnimator};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TextAnimatorPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, key_handler)
        .add_systems(Update, event_handler)
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(Text2dBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font_size: 60.0,
                        color: Color::WHITE,
                        ..default()
                    },
                },
            ],
            ..default()
        },
        ..default()
    // }).insert(TextSimpleAnimator::new("Hello, World!", 8.0));
    }).insert(TextSimpleAnimator::new("Hello, World!", 8.0));
}

fn key_handler(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TextSimpleAnimator>,
) {
    for mut animator in query.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Space) {
            animator.play();
        }
    }
}

fn event_handler(
    mut events: EventReader<TextAnimationFinished>,
) {
    for event in events.read() {
        println!("Text Animation finished for entity (id: {:?})", event.entity);
    }
}