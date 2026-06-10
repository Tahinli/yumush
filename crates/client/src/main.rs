use client::gui::Yumush;
use gpui::{
    App, AppContext, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};
use gpui_component::Root;

fn main() {
    println!("Hello, world!");

    Application::new().run(|cx: &mut App| {
        gpui_component::init(cx);

        let window_bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

        let titlebar = TitlebarOptions {
            title: Some("Nefes".into()),
            ..Default::default()
        };

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: Some(titlebar),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| Yumush::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
