use gpui::{AppContext, Context, Entity, IntoElement, ParentElement, Styled, Window, div};
use gpui_component::{
    WindowExt,
    button::Button,
    input::{Input, InputEvent, InputState},
    notification::Notification,
};

use crate::gui::{Route, Yumush};

pub struct Login {
    username_input: Entity<InputState>,
}

impl Login {
    pub fn new(window: &mut Window, cx: &mut Context<Yumush>) -> Self {
        let username_input = cx.new(|cx| {
            let mut input_state = InputState::new(window, cx);
            input_state.set_placeholder("Username...", window, cx);
            input_state.focus(window, cx);

            input_state
        });

        cx.subscribe_in(&username_input, window, |this, input, event, window, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                login_action(this, input, window, cx);
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

fn login_action(
    this: &mut Yumush,
    input: &Entity<InputState>,
    window: &mut Window,
    cx: &mut Context<'_, Yumush>,
) {
    let username = input.read(cx).text().to_string();
    let network = this.network.clone();

    cx.spawn_in(window, async move |this, cx| {
        let result = network.authenticate(&username).await;

        let _ = this.update_in(cx, |yumush, window, cx| match result {
            Ok(user) => {
                yumush.set_user(user);
                yumush.change_page(Route::Chat, window, cx);
                yumush.time_based_repetitive_checks_after_login(window, cx);
            }
            Err(error_value) => {
                window.push_notification(Notification::error(error_value.to_string()), cx);
            }
        });
    })
    .detach();
}

impl Yumush {
    pub fn login_page(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        let username_input = self.login.username_input.clone();
        let login_button = Button::new("login_button")
            .on_click(move |_, window, cx| {
                entity.update(cx, |this, cx| {
                    login_action(this, &username_input, window, cx);
                })
            })
            .label("Login");

        let entity = cx.entity();
        let register_page_switch_button = Button::new("register_page_switch_button")
            .on_click(move |_, window, cx| {
                entity.update(cx, |this, cx| {
                    this.change_page(Route::Register, window, cx);
                })
            })
            .label("Go to Register Page");

        div()
            .w_1_5()
            .child("Username")
            .child(Input::new(&self.login.username_input))
            .child(login_button)
            .child(register_page_switch_button)
            .into_element()
    }
}
