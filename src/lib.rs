use std::time::Duration;

use bevy::prelude::*;

pub struct TextAnimatorPlugin;

impl Plugin for TextAnimatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, text_simple_animator_system);
    }
}

fn text_simple_animator_system(
    time: Res<Time>,
    mut query: Query<(&mut TextSimpleAnimator, &mut Text)>,
) {
    for (mut animator, mut text) in query.iter_mut() {
        match animator.state {
            TextAnimationState::Playing => {
                if animator.timer.tick(time.delta()).just_finished() {
                    text.sections[0].value = animator.text.clone();
                    animator.timer.reset();
                    animator.state = TextAnimationState::Stopped;
                }else{
                    let val = utf8_slice::slice(&animator.text, 0, (animator.timer.elapsed().as_secs_f64() * animator.speed as f64) as usize);
                    text.sections[0].value = val.to_string();
                }
            }
            TextAnimationState::Paused => {
                animator.timer.tick(time.delta());
            }
            TextAnimationState::Stopped => {
                animator.timer.reset();
            }
        }
    }
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy)]
pub enum TextAnimationState {
    Stopped,
    Paused,
    #[default]
    Playing
}

/// animates text by showing one letter at a time, like a typewriter.
/// just change the value of first section of text
#[derive(Component)]
pub struct TextSimpleAnimator {
    pub text: String,
    /// letter per second
    pub speed: f32,
    pub state: TextAnimationState,
    timer: Timer,
}

impl TextSimpleAnimator {
    fn _calc_duration(text_len: usize, speed: f32) -> Duration {
        let duration = (text_len as f64 / speed as f64) as f64;
        Duration::from_secs_f64(duration)
    }

    pub fn duration(&self) -> Duration {
        Self::_calc_duration(self.text.len(), self.speed)
    }

    /// text, speed (letter per second)
    pub fn new(text: &str, speed: f32) -> Self {
        let duration = Self::_calc_duration(text.len(), speed);
        Self {
            text: text.to_string(),
            speed,
            state: TextAnimationState::Playing,
            timer: Timer::new(duration, TimerMode::Once),
        }
    }

    pub fn with_state(mut self, state: TextAnimationState) -> Self {
        self.state = state;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self.timer = Timer::new(Self::_calc_duration(self.text.len(), speed), TimerMode::Once);
        self
    }

    pub fn play(&mut self) {
        self.state = TextAnimationState::Playing;
        self.timer.reset();
    }

    pub fn pause(&mut self) {
        self.state = TextAnimationState::Paused;
        self.timer.pause();
    }

    pub fn stop(&mut self) {
        self.state = TextAnimationState::Stopped;
        self.timer.reset();
        self.timer.pause();
    }
    
} 

impl Default for TextSimpleAnimator {
    fn default() -> Self {
        TextSimpleAnimator::new("", 3.0)
    }
}