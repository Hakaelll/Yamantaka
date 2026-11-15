use eframe::egui::{self, Event, MouseWheelUnit};
use std::{cell::RefCell, collections::VecDeque};
use winit::{event_loop::EventLoopBuilder, platform::windows::EventLoopBuilderExtWindows};

thread_local! {
    static WHEELS: RefCell<VecDeque<(f32, bool)>> = const { RefCell::new(VecDeque::new()) };
}

pub(crate) fn install<T>(builder: &mut EventLoopBuilder<T>) {
    builder.with_msg_hook(|message| {
        // SAFETY: winit supplies a valid Win32 MSG for the duration of this hook.
        if let Some(wheel) = unsafe { yamantaka_windows::vertical_wheel_message(message) } {
            WHEELS.with_borrow_mut(|queue| {
                if queue.len() == 128 {
                    queue.pop_front();
                }
                queue.push_back(wheel);
            });
        }
        false
    });
}

pub(crate) fn preserve_modifiers(input: &mut egui::RawInput) {
    WHEELS.with_borrow_mut(|queue| {
        apply_modifiers(queue, &mut input.events);
        queue.clear();
    });
}

fn apply_modifiers(queue: &mut VecDeque<(f32, bool)>, events: &mut [Event]) {
    for event in events {
        if let Event::MouseWheel {
            unit: MouseWheelUnit::Line,
            delta,
            modifiers,
            ..
        } = event
            && delta.x == 0.0
            && queue.front().is_some_and(|(steps, _)| *steps == delta.y)
            && let Some((_, true)) = queue.pop_front()
        {
            modifiers.ctrl = true;
            modifiers.command = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touchpad_ctrl_flag_survives_native_wheel_translation() {
        let wheel = || Event::MouseWheel {
            unit: MouseWheelUnit::Line,
            delta: egui::vec2(0.0, 0.25),
            phase: egui::TouchPhase::Move,
            modifiers: egui::Modifiers::NONE,
        };
        let mut events = [wheel(), wheel()];
        apply_modifiers(
            &mut VecDeque::from([(0.25, false), (0.25, true)]),
            &mut events,
        );
        assert!(matches!(events[0], Event::MouseWheel { modifiers, .. } if !modifiers.ctrl));
        assert!(
            matches!(events[1], Event::MouseWheel { modifiers, .. } if modifiers.ctrl && modifiers.command)
        );
    }
}
