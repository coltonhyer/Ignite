use crate::components::icons::{CopyIcon, LockIcon};
use crate::crypto;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn ReadView(id: String, secret_key: String) -> Element {
    let mut is_loading = use_signal(|| false);
    let mut error_msg = use_signal(String::new);
    let mut plaintext = use_signal(String::new);
    let mut is_revealed = use_signal(|| false);
    let mut copied = use_signal(|| false);

    let reveal_secret = move |_| {
        is_loading.set(true);
        error_msg.set(String::new());

        let key_str = secret_key.clone();
        let current_id = id.clone();

        spawn(async move {
            if key_str.is_empty() {
                error_msg.set("Missing decryption key in URL.".to_string());
                is_loading.set(false);
                return;
            }

            let import_res = crypto::import_key(&key_str).await;
            if import_res.is_err() {
                error_msg.set("Invalid or malformed decryption key.".to_string());
                is_loading.set(false);
                return;
            }
            let key = import_res.unwrap();

            match crate::api::burn_secret(&current_id).await {
                Ok(data) => match crypto::decrypt_text(&data.ciphertext, &data.nonce, &key).await {
                    Ok(pt) => {
                        plaintext.set(pt);
                        is_revealed.set(true);

                        #[cfg(target_arch = "wasm32")]
                        if let Some(win) = web_sys::window() {
                            let history = win.history().unwrap();
                            let _ = history.replace_state_with_url(
                                &wasm_bindgen::JsValue::NULL,
                                "",
                                Some(&win.location().pathname().unwrap()),
                            );
                        }
                    }
                    Err(_) => {
                        error_msg.set("Decryption failed. Invalid key.".to_string());
                    }
                },
                Err(err_msg) => {
                    error_msg.set(err_msg);
                }
            }
            is_loading.set(false);
        });
    };

    if *is_revealed.read() {
        let secret_for_copy = plaintext.read().clone();
        let copy_secret = move |_| {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    let clipboard = window.navigator().clipboard();
                    let promise: js_sys::Promise = clipboard.write_text(&secret_for_copy);
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
                    let _ = clipboard.set_text(&secret_for_copy);
                    copied.set(true);
                    spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        copied.set(false);
                    });
                }
            }
        };

        return rsx! {
            div { class: "flex flex-col gap-6 w-full mt-8",
                div { class: "flex items-center justify-between",
                    h2 { class: "text-orange-500 tracking-wide text-sm", "Secret Message" }
                    div { class: "flex items-center gap-3",
                        button {
                            class: format!("px-3 py-1.5 border border-[#333] rounded-md hover:bg-[#111] transition-colors text-xs flex items-center gap-1.5 {}", if *copied.read() { "text-green-500" } else { "text-[#888]" }),
                            onclick: copy_secret,
                            if *copied.read() {
                                span { "copied" }
                            } else {
                                CopyIcon {}
                                span { "Copy" }
                            }
                        }
                        // Burn tag match
                        span { class: "text-[10px] px-2 py-0.5 bg-red-950/30 text-red-500 rounded border border-red-900/50 uppercase tracking-widest",
                            "Destroyed"
                        }
                    }
                }

                div { class: "bg-transparent border border-[#333] rounded-lg p-5 w-full",
                    pre { class: "text-[#ddd] text-xs sm:text-sm whitespace-pre-wrap break-words leading-relaxed",
                        "{plaintext}"
                    }
                }

                button {
                    class: "mt-6 text-[#777] hover:text-[#bbb] transition-colors duration-200 text-xs",
                    onclick: move |_| {
                        let nav = use_navigator();
                        nav.push(Route::Home {});
                    },
                    "create your own secret"
                }
            }
        };
    }

    rsx! {
        div { class: "flex flex-col items-center text-center gap-6 mt-12",
            div { class: "w-12 h-12 bg-[#111] rounded-full flex items-center justify-center border border-[#222]",
                LockIcon {}
            }

            div { class: "space-y-3 px-4",
                h2 { class: "text-base font-bold text-white tracking-wide", "Encrypted Message" }
                p { class: "text-[#666] text-xs leading-relaxed max-w-sm", "You have received a secure, single-use message. Once you view it, it will be permanently deleted from the server." }
            }

            button {
                class: "mt-4 w-full sm:w-[320px] bg-transparent border border-[#333] hover:border-[#555] hover:text-white text-[#ccc] py-3.5 rounded-lg transition-all duration-200 text-xs tracking-widest uppercase",
                onclick: reveal_secret,
                disabled: *is_loading.read(),
                if *is_loading.read() {
                    span { class: "animate-pulse", "Decrypting..." }
                } else {
                    span { "Reveal Secret" }
                }
            }

            if !error_msg.read().is_empty() {
                div { class: "mt-4 text-red-500 text-xs p-3 bg-red-950/20 border border-red-900/40 rounded-lg w-full sm:w-[320px] text-center",
                    "{error_msg}"
                }
            }
        }
    }
}
