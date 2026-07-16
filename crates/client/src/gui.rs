use common::user::User;
use gpui::{IntoElement, ParentElement, Render, Styled, div, rgb};
use gpui_component::Root;
use tokio::sync::mpsc;

use crate::{
    gui::{chat::Chat, login::Login, register::Register},
    network::{NetworkEvent, NetworkHandle},
};
mod chat;
mod login;
mod register;

#[derive(Debug, PartialEq, Eq)]
enum Route {
    Register,
    Login,
    Chat,
}

pub struct Yumush {
    user: Option<User>,
    current_route: Route,
    register: Register,
    login: Login,
    chat: Chat,
    network: NetworkHandle,
    connected: bool,
}

impl Yumush {
    pub fn new(
        network: NetworkHandle,
        mut network_event_receiver: mpsc::Receiver<NetworkEvent>,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> Self {
        cx.spawn(async move |this, cx| {
            while let Some(event) = network_event_receiver.recv().await {
                let updated = this.update(cx, |yumush, cx| {
                    yumush.handle_network_event(event, cx);
                });

                if updated.is_err() {
                    break;
                }
            }
        })
        .detach();

        Self {
            user: None,
            current_route: Route::Login,
            register: Register::new(window, cx),
            login: Login::new(window, cx),
            chat: Chat::new(window, cx),
            network,
            connected: false,
        }
    }

    fn set_user(&mut self, user: User) {
        self.user = Some(user);
    }

    fn reset_user(&mut self) {
        self.user = None;
    }

    fn get_username(&self) -> Option<&str> {
        self.user.as_ref().map(|user| user.get_username())
    }

    fn change_page(
        &mut self,
        new_route: Route,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) {
        self.current_route = new_route;

        match self.current_route {
            Route::Register => self.register.focus(window, cx),
            Route::Login => self.login.focus(window, cx),
            Route::Chat => self.chat.focus(window, cx),
        }
    }

    fn handle_network_event(&mut self, event: NetworkEvent, cx: &mut gpui::Context<Self>) {
        match event {
            NetworkEvent::Connected => self.connected = true,
            NetworkEvent::ConnectionFailed(_) | NetworkEvent::Disconnected(_) => {
                self.connected = false;
                self.reset_user();
            }
        }

        cx.notify();
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
                        Route::Register => self.register_page(cx).into_any_element(),
                        Route::Login => self.login_page(cx).into_any_element(),
                        Route::Chat => self.chat_page().into_any_element(),
                    }),
            )
            .child(Root::read(window, cx).notification.clone())
    }
}
