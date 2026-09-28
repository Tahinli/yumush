use client::{gui::Yumush, network, ClientConfig, NAME};
use gpui_kit::{
    application, component::Root, init, px, size, App, AppContext, Bounds, TitlebarOptions,
    WindowBounds, WindowOptions,
};

fn main() {
    println!("Hello, world!");

    let (network_handle, network_event_receiver) = network::start(ClientConfig::default());

    application().run(|cx: &mut App| {
        init(cx);

        let window_bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

        let titlebar = TitlebarOptions {
            title: Some(NAME.into()),
            ..Default::default()
        };

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: Some(titlebar),
                ..Default::default()
            },
            |window, cx| {
                let view =
                    cx.new(|cx| Yumush::new(network_handle, network_event_receiver, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
