use std::{any::Any, time::Duration};

use bevy::{prelude::*, utils::tracing::event};

pub struct TextAnimatorPlugin;

trait TextComponent {
    fn set_content(&mut self, content: String);
    fn get_content(&self) -> &str;
    fn is_empty(&self) -> bool;
}

impl TextComponent for Text2d {
    fn set_content(&mut self, content: String) {
        self.0 = content;
    }
    
    fn get_content(&self) -> &str {
        &self.0
    }
    
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl TextComponent for Text {
    fn set_content(&mut self, content: String) {
        self.0 = content;
    }
    
    fn get_content(&self) -> &str {
        &self.0
    }
    
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Plugin for TextAnimatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TextAnimationFinished>();
        app.add_systems(Update, text_animator_system::<Text2d>);
        app.add_systems(Update, text_animator_system::<Text>);
    }
}

fn text_animator_system<T: Component + TextComponent>(
    time: Res<Time>,
    mut query: Query<(&mut TextSimpleAnimator, &mut T, Entity)>,
    mut events: EventWriter<TextAnimationFinished>,
) {
    for (mut animator, mut text, entity) in query.iter_mut() {
        match animator.state {
            TextAnimationState::Playing => {
                if animator.timer.tick(time.delta()).just_finished() {
                    text.set_content(animator.text.clone());
                    animator.timer.reset();
                    animator.state = TextAnimationState::Stopped;

                    if animator.secs_wait_until_finish > 0.0 {
                        animator.end_timer = Some(Timer::from_seconds(animator.secs_wait_until_finish, TimerMode::Once));
                        animator.state = TextAnimationState::Stopped;
                    } else {
                        events.send(TextAnimationFinished { entity });
                    }
                } else {
                    let val = utf8_slice::slice(&animator.text, 0, (animator.timer.elapsed().as_secs_f64() * animator.speed as f64) as usize);
                    if animator.fill_spaces {
                        let len = animator.max_text_length();
                        let v = format!("{}{}", val, animator.fill_spaces_char.repeat(len - utf8_slice::len(&val)));
                        text.set_content(v);
                    } else {
                        text.set_content(val.to_string());
                    }
                }
            },
            TextAnimationState::Waiting(wait) => {
                if !text.is_empty() {
                    if animator.fill_spaces {
                        let len = animator.max_text_length();
                        let val = animator.fill_spaces_char.repeat(len);
                        text.set_content(val);
                    } else {
                        text.set_content("".to_string());
                    }
                }
                if wait <= 0.0 {
                    animator.state = TextAnimationState::Playing;
                } else {
                    let t = wait - time.delta_secs();
                    if t <= 0.0 {
                        animator.state = TextAnimationState::Playing;
                    } else {
                        animator.state = TextAnimationState::Waiting(t);
                    }
                }
            }
            TextAnimationState::Paused => {
                // animator.timer.tick(time.delta());
            }
            TextAnimationState::Stopped => {
                if let Some(ref mut timer) = animator.end_timer {
                    if timer.tick(time.delta()).just_finished() {
                        events.send(TextAnimationFinished { entity });
                        animator.end_timer = None;
                    }
                }
            }
        }
    }
}

#[derive(Event)]
pub struct TextAnimationFinished {
    pub entity: Entity,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum TextAnimationState {
    /// waiting for x seconds before playing
    Waiting(f32),
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
    /// wait time until finish event, from text ended
    pub secs_wait_until_finish: f32,
    /// fill spaces with empty string, to fill text length
    pub fill_spaces: bool,
    pub fill_spaces_char: String,
    timer: Timer,
    end_timer: Option<Timer>,
}

impl TextSimpleAnimator {
    fn _calc_duration(text_len: usize, speed: f32) -> Duration {
        let duration = (text_len as f64 / speed as f64) as f64;
        Duration::from_secs_f64(duration)
    }

    pub fn max_text_length(&self) -> usize {
        utf8_slice::len(&self.text)
    }

    // /// to suppress TextAnimationFinished event, after text ended.
    // /// for example, when skipping message.
    // pub fn clear_end_timer(&mut self) {
    //     self.end_timer = None;
    // }

    pub fn duration(&self) -> Duration {
        Self::_calc_duration(self.max_text_length(), self.speed)
    }

    /// text, speed (letter per second)
    pub fn new(text: &str, speed: f32) -> Self {
        let duration = Self::_calc_duration(utf8_slice::len(text), speed);
        Self {
            text: text.to_string(),
            speed,
            state: TextAnimationState::Playing,
            secs_wait_until_finish: 0.0,
            fill_spaces: false,
            fill_spaces_char: " ".to_string(),
            timer: Timer::new(duration, TimerMode::Once),
            end_timer: None,
        }
    }

    pub fn with_state(mut self, state: TextAnimationState) -> Self {
        self.state = state;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self.reset_timer();
        self
    }

    pub fn with_fill_spaces(mut self, fill: bool) -> Self {
        self.fill_spaces = fill;
        self
    }

    pub fn with_fill_spaces_char(mut self, fill_char: &str) -> Self {
        self.fill_spaces_char = fill_char.to_string();
        self
    }

    /// set wait time until finish event, from text ended
    pub fn with_wait_until_finish(mut self, secs: f32) -> Self {
        self.secs_wait_until_finish = secs;
        self
    }

    pub fn with_wait_before(mut self, secs: f32) -> Self {
        self.state = TextAnimationState::Waiting(secs);
        self
    }
    
    fn reset_timer(&mut self) {
        self.timer = Timer::new(Self::_calc_duration(self.max_text_length(), self.speed), TimerMode::Once);
    }

    /// play with waiting for x seconds before playing
    pub fn play_with_wait_before(&mut self, secs: f32) {
        self.state = TextAnimationState::Waiting(secs);
        self.reset_timer();
        self.end_timer = None;
    }

    pub fn play(&mut self) {
        self.state = TextAnimationState::Playing;
        self.reset_timer();
        self.end_timer = None;
    }

    pub fn pause(&mut self) {
        self.state = TextAnimationState::Paused;
        self.timer.pause();
        self.end_timer = None;
    }

    pub fn resume(&mut self) {
        self.state = TextAnimationState::Playing;
        self.timer.unpause();
        self.end_timer = None;
    }

    pub fn unpause(&mut self) {
        self.resume();
    }

    pub fn stop(&mut self) {
        self.state = TextAnimationState::Stopped;
        self.end_timer = None;
        self.reset_timer();
        self.timer.pause();
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.state, TextAnimationState::Playing)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.state, TextAnimationState::Paused)
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self.state, TextAnimationState::Stopped)
    }

    pub fn is_waiting(&self) -> bool {
        matches!(self.state, TextAnimationState::Waiting(_))
    }
} 

impl Default for TextSimpleAnimator {
    fn default() -> Self {
        TextSimpleAnimator::new("", 3.0)
    }
}