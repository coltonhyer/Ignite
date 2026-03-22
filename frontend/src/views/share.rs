use crate::components::icons::CopyIcon;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn ShareView(secret_id: String, ttl: String, b64_key: String) -> Element {
    let nav = use_navigator();
    let mut copied = use_signal(|| false);

    let origin = crate::api::get_origin();
    let link = format!("{}/s/{}#{}", origin, secret_id, b64_key);
    let link_for_copy = link.clone();

    let expires_text = match ttl.as_str() {
        "300" => "5 minutes",
        "900" => "15 minutes",
        "3600" => "1 hour",
        _ => "24 hours",
    };

    let copy_link = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let clipboard = window.navigator().clipboard();
                let promise: js_sys::Promise = clipboard.write_text(&link_for_copy);
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                    copied.set(true);

                    let window = web_sys::window().unwrap();
                    let cb: wasm_bindgen::closure::Closure<dyn FnMut()> =
                        wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                            copied.set(false);
                        }));
                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        wasm_bindgen::JsCast::unchecked_ref(cb.as_ref()),
                        2000,
                    );
                    cb.forget();
                });
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(&link_for_copy);
                copied.set(true);
                spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    copied.set(false);
                });
            }
        }
    };

    rsx! {
        div { class: "flex flex-col items-center gap-6 w-full mt-4",
            // Small badge matching v0 mockup
            div { class: "flex items-center justify-center gap-2 px-3 py-1 bg-green-950/20 border border-green-900/40 rounded-full",
                div { class: "w-[6px] h-[6px] rounded-full bg-green-500" }
                span { class: "text-green-500 text-[10px] tracking-widest", "Secret encrypted and ready" }
            }

            div { class: "w-full space-y-2",
                label { class: "block text-center text-[10px] uppercase tracking-widest text-[#555]", "SHARE THIS LINK" }
                div { class: "flex items-center w-full bg-transparent border border-[#333] rounded-lg p-1.5 gap-2",
                    input {
                        type: "text",
                        readonly: true,
                        value: "{link}",
                        class: "flex-1 bg-transparent text-[#ddd] focus:outline-none px-2 text-xs",
                        onclick: move |evt| { evt.stop_propagation(); },
                    }
                    button {
                        class: format!("px-3 py-1.5 border border-[#333] rounded-md hover:bg-[#111] transition-colors text-xs flex items-center gap-1.5 {}", if *copied.read() { "text-green-500" } else { "text-[#888]" }),
                        onclick: copy_link,
                        if *copied.read() {
                            span { "copied" }
                        } else {
                            CopyIcon {}
                            span { "Copy" }
                        }
                    }
                }
            }

            div { class: "flex w-full gap-3",
                div { class: "flex-1 bg-transparent border border-[#222] rounded-lg p-4 flex flex-col gap-1.5",
                    span { class: "text-[10px] uppercase tracking-widest text-[#555]", "EXPIRES" }
                    span { class: "text-white text-xs", "{expires_text}" }
                }
                div { class: "flex-1 bg-transparent border border-[#222] rounded-lg p-4 flex flex-col gap-1.5",
                    span { class: "text-[10px] uppercase tracking-widest text-[#555]", "BURN AFTER" }
                    span { class: "text-white text-xs", "First read" }
                }
            }

            button {
                class: "mt-4 w-full bg-transparent border border-[#222] hover:bg-[#111] hover:text-white text-[#bbb] py-3 px-8 rounded-lg transition-colors text-xs sm:text-sm",
                onclick: move |_| {
                    nav.push(Route::Home {});
                },
                "Create Another Secret"
            }
        }
    }
}
