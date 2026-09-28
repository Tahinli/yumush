use common::{
    community::Community,
    constant::NEWEST_MESSAGE_MARKER,
    message::Message,
    user::User,
    validate::{validate_community_name, validate_message_body},
};
use gpui_kit::{
    AppContext, Context, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window,
    component::{
        WindowExt,
        button::Button,
        input::{Input, InputEvent, InputState},
        notification::Notification,
    },
    div, rgb,
};

use crate::gui::Yumush;

pub struct Chat {
    communities: Vec<Community>,
    selected_community_id: Option<String>,
    users: Vec<User>,
    chat_messages: Vec<Message>,
    message_input: Entity<InputState>,
    community_name_input: Entity<InputState>,
    creating_community: bool,
    community_search_input: Entity<InputState>,
    community_search_results: Vec<Community>,
    joining_community: bool,
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
                let Some(community_id) = this.chat.selected_community_id.clone() else {
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

        let community_name_input = cx.new(|cx| {
            let mut input_state = InputState::new(window, cx);
            input_state.set_placeholder("Community Name...", window, cx);

            input_state
        });

        cx.subscribe_in(
            &community_name_input,
            window,
            |this, input, event, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    create_community_action(this, input, window, cx);
                }
            },
        )
        .detach();

        let community_search_input = cx.new(|cx| {
            let mut input_state = InputState::new(window, cx);
            input_state.set_placeholder("Community Name...", window, cx);

            input_state
        });

        cx.subscribe_in(
            &community_search_input,
            window,
            |this, input, event, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    search_community_action(this, input, window, cx);
                }
            },
        )
        .detach();

        Self {
            communities: vec![],
            selected_community_id: None,
            users: vec![],
            chat_messages: vec![],
            message_input,
            community_name_input,
            creating_community: false,
            community_search_input,
            community_search_results: vec![],
            joining_community: false,
        }
    }

    pub fn focus(&self, window: &mut Window, cx: &mut Context<'_, Yumush>) {
        self.message_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
    }

    pub fn push_message(&mut self, message: Message) {
        self.merge_messages(vec![message]);
    }

    fn merge_messages(&mut self, messages: Vec<Message>) {
        self.chat_messages.extend(messages);
        self.chat_messages
            .sort_by(|left, right| left.get_id().cmp(right.get_id()));
        self.chat_messages
            .dedup_by(|left, right| left.get_id() == right.get_id());
    }

    pub fn update_user_list(&mut self, users: Vec<User>) {
        self.users = users;
    }

    pub fn update_community_list(&mut self, communities: Vec<Community>) {
        if self.selected_community_id.is_none() {
            self.selected_community_id = communities
                .first()
                .map(|community| community.get_community_id().to_string());
        }

        self.communities = communities;
    }

    pub fn get_selected_community_id(&self) -> Option<String> {
        self.selected_community_id.to_owned()
    }

    pub fn set_selected_community_id(&mut self, selected_community_id: Option<String>) {
        self.selected_community_id = selected_community_id
    }

    pub fn update_searched_community_list(&mut self, communities: Vec<Community>) {
        self.community_search_results = communities;
    }

    pub fn set_joining_community(&mut self, joining_community: bool) {
        self.joining_community = joining_community
    }

    pub fn reset(&mut self) {
        self.communities.clear();
        self.selected_community_id = None;
        self.users.clear();
        self.chat_messages.clear();
        self.creating_community = false;
        self.community_search_results.clear();
        self.joining_community = false;
    }
}

fn create_community_action(
    this: &mut Yumush,
    input: &Entity<InputState>,
    window: &mut Window,
    cx: &mut Context<'_, Yumush>,
) {
    let community_name = input.read(cx).text().to_string();

    if let Err(error_value) = validate_community_name(&community_name) {
        window.push_notification(Notification::error(error_value.to_string()), cx);

        return;
    }

    let network = this.network.clone();

    input.update(cx, |input_state, cx| {
        input_state.set_value("", window, cx);
    });
    this.chat.creating_community = false;

    cx.spawn_in(window, async move |this, cx| {
        let community = match network.create_community(&community_name).await {
            Ok(community) => community,
            Err(error_value) => {
                let _ = this.update_in(cx, |_, window, cx| {
                    window.push_notification(Notification::error(error_value.to_string()), cx);
                });

                return;
            }
        };

        let _ = this.update_in(cx, |yumush, window, cx| {
            yumush
                .chat
                .set_selected_community_id(Some(community.get_community_id().to_string()));
            yumush.chat.update_user_list(vec![]);
            yumush.get_communities(window, cx);
            yumush.get_chat_users(window, cx);
            yumush.get_chat_messages(window, cx);
            cx.notify();
        });
    })
    .detach();
}

fn search_community_action(
    this: &mut Yumush,
    input: &Entity<InputState>,
    window: &mut Window,
    cx: &mut Context<'_, Yumush>,
) {
    let community_name = input.read(cx).text().to_string();

    if let Err(error_value) = validate_community_name(&community_name) {
        window.push_notification(Notification::error(error_value.to_string()), cx);

        return;
    }

    let network = this.network.clone();

    cx.spawn_in(window, async move |this, cx| {
        let communities = match network.search_community(&community_name).await {
            Ok(communities) => communities,
            Err(error_value) => {
                let _ = this.update_in(cx, |_, window, cx| {
                    window.push_notification(Notification::error(error_value.to_string()), cx);
                });

                return;
            }
        };

        let _ = this.update(cx, |yumush, cx| {
            yumush.chat.update_searched_community_list(communities);
            cx.notify();
        });
    })
    .detach();
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
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_color(rgb(0x949ba4))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Communities"),
                            )
                            .child({
                                let entity = entity.clone();

                                Button::new("create_community_button").label("+").on_click(
                                    move |_, window, cx| {
                                        entity.update(cx, |yumush, cx| {
                                            yumush.chat.creating_community =
                                                !yumush.chat.creating_community;

                                            if yumush.chat.creating_community {
                                                yumush.chat.community_name_input.update(
                                                    cx,
                                                    |input_state, cx| {
                                                        input_state.focus(window, cx);
                                                    },
                                                );
                                            }

                                            cx.notify();
                                        });
                                    },
                                )
                            })
                            .child({
                                let entity = entity.clone();

                                Button::new("join_community_button").label("Join").on_click(
                                    move |_, window, cx| {
                                        entity.update(cx, |yumush, cx| {
                                            yumush.chat.joining_community =
                                                !yumush.chat.joining_community;

                                            if yumush.chat.joining_community {
                                                yumush.chat.community_search_input.update(
                                                    cx,
                                                    |input_state, cx| {
                                                        input_state.focus(window, cx);
                                                    },
                                                );
                                            } else {
                                                yumush.chat.update_searched_community_list(vec![]);
                                            }

                                            cx.notify();
                                        });
                                    },
                                )
                            }),
                    )
                    .children(
                        self.chat
                            .creating_community
                            .then(|| Input::new(&self.chat.community_name_input)),
                    )
                    .children(
                        self.chat
                            .joining_community
                            .then(|| Input::new(&self.chat.community_search_input)),
                    )
                    .children(
                        self.chat
                            .community_search_results
                            .iter()
                            .filter(|result| {
                                !self.chat.communities.iter().any(|community| {
                                    community.get_community_id() == result.get_community_id()
                                })
                            })
                            .enumerate()
                            .map(|(index, result)| {
                                let community_id = result.get_community_id().to_string();
                                let entity = entity.clone();

                                div()
                                    .id(("community_search_result", index))
                                    .p_1()
                                    .cursor_pointer()
                                    .bg(rgb(0x1e1f22))
                                    .text_color(rgb(0x949ba4))
                                    .child(result.get_community_name().to_string())
                                    .on_click(move |_, window, cx| {
                                        entity.update(cx, |yumush, cx| {
                                            yumush.join_community(&community_id, window, cx);
                                        })
                                    })
                            }),
                    )
                    .children(self.chat.communities.iter().enumerate().map(
                        |(index, community)| {
                            let community_id = community.get_community_id().to_string();
                            let selected = self.chat.get_selected_community_id().as_deref()
                                == Some(&community_id);
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
                                        yumush.chat.set_selected_community_id(Some(
                                            community_id.to_owned(),
                                        ));
                                        yumush.chat.update_user_list(vec![]);
                                        yumush.get_chat_users(window, cx);
                                        yumush.get_chat_messages(window, cx);
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
                                            == self.chat.get_selected_community_id().as_deref()
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

        let Some(community_id) = self.chat.get_selected_community_id().to_owned() else {
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

    pub fn get_chat_messages(&self, window: &mut Window, cx: &mut Context<Self>) {
        let network = self.network.clone();

        let Some(community_id) = self.chat.get_selected_community_id() else {
            return;
        };

        cx.spawn_in(window, async move |this, cx| {
            let messages = match network
                .read_message_by_community_id_with_marker_and_limit(
                    community_id.as_str(),
                    NEWEST_MESSAGE_MARKER,
                )
                .await
            {
                Ok(messages) => messages,
                Err(error_value) => {
                    let _ = this.update_in(cx, |_, window, cx| {
                        window.push_notification(Notification::error(error_value.to_string()), cx);
                    });

                    return;
                }
            };

            let _ = this.update(cx, |yumush, cx| {
                yumush.chat.merge_messages(messages);
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

    pub fn join_community(&self, community_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(user) = self.get_user() else {
            return;
        };

        let network = self.network.clone();
        let community_id = community_id.to_string();

        cx.spawn_in(window, async move |this, cx| {
            if let Err(error_value) = network
                .join_community(user.get_user_id(), &community_id)
                .await
            {
                let _ = this.update_in(cx, |_, window, cx| {
                    window.push_notification(Notification::error(error_value.to_string()), cx);
                });

                return;
            }

            let _ = this.update_in(cx, |yumush, window, cx| {
                yumush.chat.set_joining_community(false);
                yumush.chat.update_searched_community_list(vec![]);
                yumush.chat.set_selected_community_id(Some(community_id));
                yumush.chat.update_user_list(vec![]);
                yumush.get_communities(window, cx);
                yumush.get_chat_users(window, cx);
                yumush.get_chat_messages(window, cx);

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
