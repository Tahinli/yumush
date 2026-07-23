use std::time::Duration;

use gpui::{Context, Window};

use crate::gui::Yumush;

const CHECK_INTERVAL: Duration = Duration::from_secs(3);

impl Yumush {
    pub fn time_based_repetitive_checks_after_login(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor().timer(CHECK_INTERVAL).await;

                let keep_going = this.update_in(cx, |yumush, window, cx| {
                    if !yumush.is_logged_in() {
                        return false;
                    }

                    if yumush.is_connected() {
                        yumush.get_chat_users(window, cx);
                        yumush.get_communities(window, cx);
                    }

                    true
                });

                if !matches!(keep_going, Ok(true)) {
                    break;
                }
            }
        })
        .detach();
    }
}
