use gpui::{IntoElement, ParentElement, Render, Styled, div, rgb};
use gpui_component::Root;

use crate::gui::{chat::Chat, login::Login};
mod chat;
mod login;

#[derive(Debug, PartialEq, Eq)]
enum Route {
    Login,
    Chat,
}

pub struct Yumush {
    current_route: Route,
    login: Login,
    chat: Chat,
}

impl Yumush {
    pub fn new(window: &mut gpui::Window, cx: &mut gpui::Context<Self>) -> Self {
        Self {
            current_route: Route::Login,
            login: Login::new(window, cx),
            chat: Chat::new(window, cx),
        }
    }

    fn change_page(
        &mut self,
        new_route: Route,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        self.current_route = new_route;

        match self.current_route {
            Route::Login => self.login.focus(window, cx),
            Route::Chat => self.chat.focus(window, cx),
        }
    }
}

impl Render for Yumush {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .size_full()
            .relative()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .items_center()
                    .size_full()
                    .bg(rgb(0x1e1e1e))
                    .text_color(rgb(0xffffff))
                    .child(match self.current_route {
                        Route::Login => self.login_page(cx).into_any_element(),
                        Route::Chat => self.chat_page().into_any_element(),
                    }),
            )
            .child(Root::read(window, cx).notification.clone())
    }
}
