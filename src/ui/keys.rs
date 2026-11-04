use crownui::{
    ext::{Key, KeyboardEvent, NamedKey},
    prelude::{Cx, Event},
};

use super::{
    activate::activate, grid::arrangement, query::edit, results::fitted, state::Launcher,
    visibility,
};
use crate::search::Hit;

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
            if let Some(hit) = selected(cx, launcher) {
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

fn shown_hits(cx: &mut Cx, launcher: Launcher) -> Vec<Hit> {
    let frame = arrangement(cx, launcher).results;
    fitted(cx, launcher, frame.size.height)
        .into_iter()
        .flat_map(|section| section.rows.into_iter().map(|row| row.hit))
        .collect()
}

fn selected(cx: &mut Cx, launcher: Launcher) -> Option<Hit> {
    let index = launcher.selection.get(cx).0;
    shown_hits(cx, launcher).into_iter().nth(index)
}

fn step(cx: &mut Cx, launcher: Launcher, by: isize) {
    let rows = shown_hits(cx, launcher).len();
    launcher
        .selection
        .update(cx, |selection| *selection = selection.step(by, rows));
}
