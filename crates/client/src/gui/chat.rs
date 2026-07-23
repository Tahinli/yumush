use common::{community::Community, message::Message, user::User, validate::validate_message_body};
use gpui::{
    AppContext, Context, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, rgb,
};
use gpui_component::{
    WindowExt,
    input::{Input, InputEvent, InputState},
    notification::Notification,
};

use crate::gui::Yumush;

pub struct Chat {
    communities: Vec<Community>,
    selected_community: Option<String>,
    users: Vec<User>,
    chat_messages: Vec<Message>,
    message_input: Entity<InputState>,
}

impl Chat {
    pub fn new(window: &mut Window, cx: &mut Context<Yumush>) -> Self {
        let message_input = cx.new(|cx| {
            let mut input_state = InputState::new(window, cx);
            input_state.set_placeholder("", window, cx);

            input_state
        });

        cx.subscribe_in(&message_input, window, |this, input, event, window, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                let message_body = input.read(cx).text().to_string();

                if let Err(error_value) = validate_message_body(&message_body) {
                    window.push_notification(Notification::error(error_value.to_string()), cx);

                    return;
                }

                let Some(user) = this.get_user() else {
                    return;
                };

                let network = this.network.clone();
                let Some(community_id) = this.chat.selected_community.clone() else {
                    window.push_notification(Notification::error("No Community Selected"), cx);
                    return;
                };

                cx.spawn_in(window, async move |this, cx| {
                    if let Err(error_value) = network
                        .create_message(user.get_user_id(), &community_id, &message_body)
                        .await
                    {
                        let _ = this.update_in(cx, |_, window, cx| {
                            window.push_notification(
                                Notification::error(error_value.to_string()),
                                cx,
                            );
                        });
                    }
                })
                .detach();

                input.update(cx, |input_state, cx| {
                    input_state.set_value("", window, cx);
                });
            }
        })
        .detach();

        Self {
            communities: vec![],
            selected_community: None,
            users: vec![],
            chat_messages: vec![],
            message_input,
        }
    }

    pub fn focus(&self, window: &mut Window, cx: &mut Context<'_, Yumush>) {
        self.message_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
    }

    pub fn push_message(&mut self, message: Message) {
        self.chat_messages.push(message);
    }

    pub fn update_user_list(&mut self, users: Vec<User>) {
        self.users = users;
    }

    pub fn update_community_list(&mut self, communities: Vec<Community>) {
        if self.selected_community.is_none() {
            self.selected_community = communities
                .first()
                .map(|community| community.get_community_id().to_string());
        }

        self.communities = communities;
    }
}

impl Yumush {
    pub fn chat_page(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let entity = cx.entity();
        div()
            .flex()
            .flex_row()
            .size_full()
            .child(
                div()
                    .id("community_list")
                    .h_full()
                    .flex()
                    .flex_col()
                    .flex_shrink_0()
                    .w_1_5()
                    .border_r_1()
                    .border_color(rgb(0x2f3136))
                    .bg(rgb(0x2b2d31))
                    .p_2()
                    .gap_1()
                    .child(
                        div()
                            .text_color(rgb(0x949ba4))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Communities"),
                    )
                    .children(self.chat.communities.iter().enumerate().map(
                        |(index, community)| {
                            let community_id = community.get_community_id().to_string();
                            let selected =
                                self.chat.selected_community.as_deref() == Some(&community_id);
                            let entity = entity.clone();

                            div()
                                .id(("community", index))
                                .p_1()
                                .cursor_pointer()
                                .bg(if selected {
                                    rgb(0x404249)
                                } else {
                                    rgb(0x2b2d31)
                                })
                                .child(community.get_community_name().to_string())
                                .on_click(move |_, window, cx| {
                                    entity.update(cx, |yumush, cx| {
                                        yumush.chat.selected_community =
                                            Some(community_id.to_owned());
                                        yumush.chat.update_user_list(vec![]);
                                        yumush.get_chat_users(window, cx);
                                        cx.notify();
                                    })
                                })
                        },
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .id("chat_messages")
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .overflow_y_scroll()
                            .p_2()
                            .gap_1()
                            .children(
                                self.chat
                                    .chat_messages
                                    .iter()
                                    .filter(|message| {
                                        Some(message.get_community_id())
                                            == self.chat.selected_community.as_deref()
                                    })
                                    .map(|message| render_message(message, &self.chat.users)),
                            ),
                    )
                    .child(
                        div()
                            .p_2()
                            .border_t_1()
                            .border_color(rgb(0x2f3136))
                            .child(Input::new(&self.chat.message_input)),
                    ),
            )
            .child(
                div()
                    .id("user_list")
                    .h_full()
                    .flex()
                    .flex_col()
                    .flex_shrink_0()
                    .w_1_5()
                    .border_l_1()
                    .border_color(rgb(0x2f3136))
                    .bg(rgb(0x2b2d31))
                    .p_2()
                    .gap_1()
                    .child(
                        div()
                            .text_color(rgb(0x949ba4))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("Online - {}", self.chat.users.len())),
                    )
                    .children(
                        self.chat
                            .users
                            .iter()
                            .map(|user| div().p_1().child(user.get_username().to_string())),
                    ),
            )
    }

    pub fn get_chat_users(&self, window: &mut Window, cx: &mut Context<Self>) {
        let network = self.network.clone();

        let Some(community_id) = self.chat.selected_community.to_owned() else {
            return;
        };

        cx.spawn_in(window, async move |this, cx| {
            let user_ids = match network.users_in(&community_id).await {
                Ok(user_ids) => user_ids,
                Err(error_value) => {
                    let _ = this.update_in(cx, |_, window, cx| {
                        window.push_notification(Notification::error(error_value.to_string()), cx);
                    });

                    return;
                }
            };

            let mut users = Vec::with_capacity(user_ids.len());
            for user_id in &user_ids {
                if let Ok(user) = network.read_user(user_id).await {
                    users.push(user);
                }
            }

            let _ = this.update(cx, |yumush, cx| {
                yumush.chat.update_user_list(users);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn get_communities(&self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(user) = self.get_user() else {
            return;
        };

        let network = self.network.clone();

        cx.spawn_in(window, async move |this, cx| {
            let community_ids = match network.community_of(user.get_user_id()).await {
                Ok(community_ids) => community_ids,
                Err(error_value) => {
                    let _ = this.update_in(cx, |_, window, cx| {
                        window.push_notification(Notification::error(error_value.to_string()), cx);
                    });

                    return;
                }
            };

            let mut communities = Vec::with_capacity(community_ids.len());
            for community_id in community_ids {
                if let Ok(community) = network.read_community(&community_id).await {
                    communities.push(community);
                }
            }

            let _ = this.update(cx, |yumush, cx| {
                yumush.chat.update_community_list(communities);
                cx.notify();
            });
        })
        .detach();
    }
}

fn render_message(message: &Message, users: &[User]) -> impl IntoElement {
    let username = users
        .iter()
        .find(|user| user.get_user_id() == message.get_user_id())
        .map(User::get_username)
        .unwrap_or(message.get_user_id());
    div()
        .flex()
        .flex_col()
        .p_1()
        .child(
            div()
                .text_color(rgb(0x7289da))
                .font_weight(FontWeight::BOLD)
                .child(username.to_string()),
        )
        .child(div().pl_2().child(message.get_message_body().to_string()))
}
