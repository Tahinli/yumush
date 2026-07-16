use gpui::{AppContext, Context, Entity, IntoElement, ParentElement, Styled, Window, div};
use gpui_component::{
    WindowExt,
    button::Button,
    input::{Input, InputEvent, InputState},
    notification::Notification,
};

use crate::gui::{Route, Yumush};

pub struct Register {
    username_input: Entity<InputState>,
}

impl Register {
    pub fn new(window: &mut Window, cx: &mut Context<Yumush>) -> Self {
        let username_input = cx.new(|cx| {
            let mut input_state = InputState::new(window, cx);
            input_state.set_placeholder("Username...", window, cx);
            input_state.focus(window, cx);

            input_state
        });

        cx.subscribe_in(&username_input, window, |this, input, event, window, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                register_action(this, input, window, cx);
            }
        })
        .detach();

        Self { username_input }
    }

    pub fn focus(&self, window: &mut Window, cx: &mut Context<'_, Yumush>) {
        self.username_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
    }
}

fn register_action(
    this: &mut Yumush,
    input: &Entity<InputState>,
    window: &mut Window,
    cx: &mut Context<'_, Yumush>,
) {
    let username = input.read(cx).text().to_string();
    let network = this.network.clone();

    cx.spawn_in(window, async move |this, cx| {
        let result = network.create_user(&username).await;

        let _ = this.update_in(cx, |yumush, window, cx| match result {
            Ok(user) => {
                yumush.set_user(user);
                yumush.change_page(Route::Chat, window, cx);
            }
            Err(error_value) => {
                window.push_notification(Notification::error(error_value.to_string()), cx);
            }
        });
    })
    .detach();
}

impl Yumush {
    pub fn register_page(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let username_input = self.register.username_input.clone();
        let register_button = Button::new("register_button")
            .on_click(move |_, window, cx| {
                entity.update(cx, |this, cx| {
                    register_action(this, &username_input, window, cx);
                })
            })
            .label("Register");

        let entity = cx.entity();
        let login_page_switch_button = Button::new("login_page_switch_button")
            .on_click(move |_, window, cx| {
                entity.update(cx, |this, cx| {
                    this.change_page(Route::Login, window, cx);
                })
            })
            .label("Go to Login Page");

        div()
            .w_1_5()
            .child("Username")
            .child(Input::new(&self.register.username_input))
            .child(register_button)
            .child(login_page_switch_button)
            .into_element()
    }
}
