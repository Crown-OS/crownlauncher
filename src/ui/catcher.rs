//! A second, invisible surface across the whole output, under the launcher's own. It only takes
//! clicks, and only while the launcher is shown, so a click anywhere off the window closes it. It
//! never redraws while the launcher animates, which keeps the compositor's work to the window.

use crownui::prelude::{Cx, Listeners, Signal, Styled, View, bind, component, vstack};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// `shown` follows the launcher; a click sends `false` down `requests`.
pub fn catcher(shown: UnboundedReceiver<bool>, requests: UnboundedSender<bool>) -> impl View {
    component(move |cx| {
        let active = cx.signal(false);
        follow(cx, active, shown);
        let backdrop = vstack(())
            .absolute()
            .inset(0.0)
            .pointer_events_auto()
            .hidden(bind(move |runtime| !active.get(runtime)))
            .on_click(move |_, _| {
                let _ = requests.send(false);
            });
        vstack(backdrop).absolute().inset(0.0).pointer_events_none()
    })
}

fn follow(cx: &mut Cx, active: Signal<bool>, mut shown: UnboundedReceiver<bool>) {
    let next = async move {
        let latest = shown.recv().await;
        (shown, latest)
    };
    cx.spawn(next, move |cx, (shown, latest)| {
        let Some(latest) = latest else {
            return;
        };
        active.set(cx, latest);
        follow(cx, active, shown);
    });
}
