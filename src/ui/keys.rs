use crownui::{
    ext::{Key, KeyboardEvent, NamedKey},
    prelude::{Cx, Event},
};

use super::{
    activate::activate,
    query::edit,
    results::{fitted, selected_hit},
    state::Launcher,
    visibility,
};

pub fn down(cx: &mut Cx, launcher: Launcher, event: &mut Event<KeyboardEvent>) {
    let modifiers = event.modifiers;
    let command = modifiers.ctrl_key || modifiers.meta_key;
    match &event.key {
        Key::Named(NamedKey::Escape) => {
            if launcher.searching(cx) {
                edit(cx, launcher, String::clear);
            } else {
                visibility::set(cx, launcher, false);
            }
        }
        Key::Named(NamedKey::Enter) => {
            if let Some(hit) = selected_hit(cx, launcher) {
                activate(cx, launcher, &hit);
            }
        }
        Key::Named(NamedKey::ArrowDown) => step(cx, launcher, 1),
        Key::Named(NamedKey::ArrowUp) => step(cx, launcher, -1),
        Key::Named(NamedKey::Tab) => step(cx, launcher, if modifiers.shift_key { -1 } else { 1 }),
        Key::Named(NamedKey::Backspace) if command => edit(cx, launcher, |query| {
            let kept = query
                .trim_end()
                .rfind(char::is_whitespace)
                .map_or(0, |at| at + 1);
            query.truncate(kept);
        }),
        Key::Named(NamedKey::Backspace) => edit(cx, launcher, |query| {
            query.pop();
        }),
        Key::Character(character) if command && character.eq_ignore_ascii_case(&'u') => {
            edit(cx, launcher, String::clear);
        }
        _ => {
            let typed = event.text.clone().filter(|text| {
                !command
                    && !modifiers.alt_key
                    && !text.is_empty()
                    && text.chars().all(|character| !character.is_control())
            });
            match typed {
                Some(text) => edit(cx, launcher, |query| query.push_str(&text)),
                None => return,
            }
        }
    }
    event.prevent_default();
}

fn shown_rows(cx: &mut Cx, launcher: Launcher) -> usize {
    fitted(cx, launcher)
        .iter()
        .map(|section| section.rows.len())
        .sum()
}

fn step(cx: &mut Cx, launcher: Launcher, by: isize) {
    let rows = shown_rows(cx, launcher);
    launcher
        .selection
        .update(cx, |selection| *selection = selection.step(by, rows));
}
