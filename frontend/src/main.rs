#![allow(non_snake_case)]
#![allow(unexpected_cfgs)]

pub mod api;
pub mod components;
pub mod crypto;
pub mod views;

use dioxus::prelude::*;

use components::layout::Layout;
use views::home::Home;
use views::read::ReadView;
use views::share::ShareView;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/c/:secret_id/:ttl#:b64_key")]
    ShareView {
        secret_id: String,
        ttl: String,
        b64_key: String,
    },
    #[route("/s/:id#:secret_key")]
    ReadView { id: String, secret_key: String },
}

fn main() {
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[allow(unused_mut)]
        let window = dioxus::desktop::WindowBuilder::new()
            .with_theme(Some(dioxus::desktop::tao::window::Theme::Dark))
            .with_title("")
            .with_inner_size(dioxus::desktop::LogicalSize::new(500, 750));

        #[cfg(target_os = "macos")]
        use dioxus::desktop::tao::platform::macos::WindowBuilderExtMacOS;

        #[cfg(target_os = "macos")]
        let window = window.with_titlebar_transparent(true);

        let cfg = dioxus::desktop::Config::new()
            .with_background_color((0, 0, 0, 255))
            .with_custom_head(r##"
                <script src="https://cdn.tailwindcss.com"></script>
                <script>
                    tailwind.config = {
                        theme: {
                            extend: {
                                fontFamily: {
                                    mono: ['"JetBrains Mono"', 'monospace'],
                                }
                            }
                        }
                    }
                </script>
                <link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;700&display=swap" rel="stylesheet">
                <style>body { background-color: #000; color: #e5e7eb; }</style>
            "##.to_string())
            .with_window(window);

        dioxus::LaunchBuilder::new().with_cfg(cfg).launch(App);
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        launch(App);
    }
}

fn App() -> Element {
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(target_os = "macos")]
    use_effect(move || {
        use dioxus::desktop::tao::platform::macos::WindowExtMacOS;
        use objc::runtime::Object;
        use objc::{class, msg_send, sel, sel_impl};

        let desktop = dioxus::desktop::window();
        let window = &desktop.window;
        let ns_window = window.ns_window() as *mut Object;
        unsafe {
            let color_class = class!(NSColor);
            let color: *mut Object = msg_send![color_class, blackColor];
            let _: () = msg_send![ns_window, setBackgroundColor: color];
        }
    });

    rsx! {
        document::Title { "Ignite" }
        Router::<Route> {}
    }
}
