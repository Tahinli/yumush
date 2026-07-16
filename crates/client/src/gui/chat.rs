use common::{community::Community, message::Message, user::User};
use gpui::{
    AppContext, Context, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, rgb,
};
use gpui_component::input::{Input, InputEvent, InputState};

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

        let community = Community::new("1", "Community");

        let community = || community.clone();
        cx.subscribe_in(&message_input, window, |this, input, event, window, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                let message = Message::new(
                    "1",
                    this.chat.community.get_community_id(),
                    this.get_username().unwrap(),
                    &input.read(cx).text().to_string(),
                );

                this.chat.chat_messages.push(message);
                input.update(cx, |input_state, cx| {
                    input_state.set_value("", window, cx);
                });
            }
        })
        .detach();

        let person_1 = User::new("Ahmet", "Ahmet");
        let person_2 = User::new("Kaan", "Kaan");
        let message_1 = Message::new(
            "1",
            community().get_community_id(),
            person_1.get_user_id(),
            "Hello, how are you?",
        );
        let message_2 = Message::new(
            "1",
            community().get_community_id(),
            person_2.get_user_id(),
            "I'm doing well. How are you?",
        );
        let person_1 = User::new("1", "Ahmet");
        let message_3 = Message::new(
            "1",
            community().get_community_id(),
            person_1.get_username(),
            "I'm good too. Thanks for asking!",
        );
        let person_2 = User::new("1", "Kaan");
        let message_4 = Message::new(
            "1",
            community().get_community_id(),
            person_2.get_username(),
            &"A".repeat(1024),
        );
        let person_1 = User::new("1", "Ahmet");
        let person_2 = User::new("1", "Kaan");
        Self {
            community: community(),
            users: vec![person_1, person_2],
            chat_messages: vec![message_1, message_2, message_3, message_4],
            message_input,
        }
    }

    pub fn focus(&self, window: &mut Window, cx: &mut Context<'_, Yumush>) {
        self.message_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
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
