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
    community: Community,
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

        let community = Community::new("oko1k3hk3r21dhqa20az", "The Community");

        let community = || community.clone();
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
                let community_id = this.chat.community.get_community_id().to_string();

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
            community: community(),
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

    pub fn push_user(&mut self, user: &User) {
        self.users.push(user.to_owned());
    }
}

impl Yumush {
    pub fn chat_page(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .size_full()
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
                            .children(self.chat.chat_messages.iter().map(render_message)),
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
}

fn render_message(message: &Message) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .p_1()
        .child(
            div()
                .text_color(rgb(0x7289da))
                .font_weight(FontWeight::BOLD)
                .child(message.get_user_id().to_string()),
        )
        .child(div().pl_2().child(message.get_message_body().to_string()))
}
