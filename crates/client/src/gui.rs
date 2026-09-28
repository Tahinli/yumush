use common::user::User;
use gpui_kit::{
    Context, IntoElement, ParentElement, Render, Styled, Window,
    component::{ActiveTheme, Root, WindowExt, button::Button, notification::Notification},
    div, rgb,
};
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
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.spawn_in(window, async move |this, cx| {
            while let Some(event) = network_event_receiver.recv().await {
                let updated = this.update_in(cx, |yumush, window, cx| {
                    yumush.handle_network_event(event, window, cx);
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

    fn get_user(&self) -> Option<User> {
        self.user.clone()
    }

    fn reset_user(&mut self) {
        self.user = None;
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn is_logged_in(&self) -> bool {
        self.user.is_some()
    }

    fn get_username(&self) -> Option<&str> {
        self.user.as_ref().map(|user| user.get_username())
    }

    fn logout(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(user) = self.get_user() else {
            return;
        };
        let network = self.network.clone();

        self.reset_user();
        self.chat.reset();
        self.change_page(Route::Login, window, cx);

        cx.spawn_in(window, async move |this, cx| {
            if let Err(error_value) = network.deauthenticate(user.get_username()).await {
                let _ = this.update_in(cx, |_, window, cx| {
                    window.push_notification(Notification::error(error_value.to_string()), cx);
                });
            }
        })
        .detach();
    }

    fn change_page(&mut self, new_route: Route, window: &mut Window, cx: &mut Context<Self>) {
        self.current_route = new_route;
        cx.notify();

        match self.current_route {
            Route::Register => self.register.focus(window, cx),
            Route::Login => self.login.focus(window, cx),
            Route::Chat => self.chat.focus(window, cx),
        }
    }

    fn handle_network_event(
        &mut self,
        event: NetworkEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            NetworkEvent::Connected(user) => {
                self.connected = true;
                match user {
                    Some(user) => self.set_user(user),
                    None if self.is_logged_in() => {
                        self.reset_user();
                        self.change_page(Route::Login, window, cx);
                    }
                    None => {}
                }
            }
            NetworkEvent::ConnectionFailed(_) | NetworkEvent::Disconnected(_) => {
                self.connected = false;
            }
            NetworkEvent::MessageReceived(message) => self.chat.push_message(message),
        }

        cx.notify();
    }
}

impl Render for Yumush {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                    .text_color(cx.theme().foreground)
                    .child(match self.current_route {
                        Route::Register => self.register_page(cx).into_any_element(),
                        Route::Login => self.login_page(cx).into_any_element(),
                        Route::Chat => self.chat_page(cx).into_any_element(),
                    }),
            )
            .child(Root::read(window, cx).notification.clone())
            .child(
                div()
                    .absolute()
                    .top_2()
                    .right_2()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_color(if self.connected {
                                rgb(0x22c55e)
                            } else {
                                rgb(0xef4444)
                            })
                            .child(format!(
                                "{:?} is {}",
                                self.get_username(),
                                if self.connected { "online" } else { "offline" }
                            )),
                    )
                    .children(self.is_logged_in().then(|| {
                        let entity = cx.entity();

                        Button::new("logout_button").label("Logout").on_click(
                            move |_, window, cx| {
                                entity.update(cx, |yumush, cx| yumush.logout(window, cx));
                            },
                        )
                    })),
            )
    }
}
