use common::user::User;
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
    user: Option<User>,
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

        Self {
            username_input,
            user: None,
        }
    }

    pub fn get_user(&self) -> Option<User> {
        self.user.clone()
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
    let id = "A".to_string();
    let username = input.read(cx).text().to_string();

    let user = User::new(&id, &username);
    this.login.user = Some(user);
    this.change_page(Route::Chat, window, cx);
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

        div()
            .w_1_5()
            .child("Username")
            .child(Input::new(&self.login.username_input))
            .child(login_button)
            .into_element()
    }
}
