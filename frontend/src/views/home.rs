use crate::components::icons::{ClockIcon, ShieldIcon};
use crate::crypto;
use crate::Route;
use dioxus::prelude::*;

const MAX_PLAINTEXT_BYTES: usize = 7000;

#[component]
pub fn Home() -> Element {
    let nav = use_navigator();
    let mut secret_text = use_signal(String::new);
    let mut ttl_seconds = use_signal(|| 86400i64); // default 24 hours
    let mut is_loading = use_signal(|| false);
    let mut error_msg = use_signal(String::new);

    let char_count = secret_text.read().len();
    let is_over_limit = char_count > MAX_PLAINTEXT_BYTES;

    let create_secret = move |_| {
        if secret_text.read().is_empty() {
            error_msg.set("Please enter a secret.".to_string());
            return;
        }
        if is_over_limit {
            error_msg.set(format!(
                "Secret is too long. Max size is {} bytes.",
                MAX_PLAINTEXT_BYTES
            ));
            return;
        }

        is_loading.set(true);
        error_msg.set(String::new());

        spawn(async move {
            let plaintext = secret_text.read().clone();
            let ttl = *ttl_seconds.read();

            let encrypt_res = crypto::encrypt_text(&plaintext).await;
            if encrypt_res.is_err() {
                error_msg.set("Failed to encrypt client-side.".to_string());
                is_loading.set(false);
                return;
            }
            let (ciphertext, nonce, key) = encrypt_res.unwrap();

            let export_res = crypto::export_key(&key).await;
            if export_res.is_err() {
                error_msg.set("Failed to export key.".to_string());
                is_loading.set(false);
                return;
            }
            let b64url_key = export_res.unwrap();

            let req = shared::CreateSecretRequest {
                ciphertext,
                nonce,
                ttl_seconds: Some(ttl),
            };
            match crate::api::create_secret(req).await {
                Ok(data) => {
                    nav.push(Route::ShareView {
                        secret_id: data.id.to_string(),
                        b64_key: b64url_key,
                        ttl: ttl.to_string(),
                    });
                }
                Err(err_msg) => {
                    error_msg.set(err_msg);
                }
            }
            is_loading.set(false);
        });
    };

    rsx! {
        div { class: "flex flex-col gap-5 w-full",
            div { class: "space-y-2",
                label { class: "block text-[10px] uppercase tracking-widest text-[#555]", "YOUR SECRET" }
                div { class: "relative",
                    textarea {
                        id: "secret-text",
                        rows: 8,
                        class: "w-full bg-black border border-[#222] rounded-lg p-4 text-[#ddd] placeholder-[#444] focus:outline-none focus:border-[#444] resize-y text-xs sm:text-sm leading-relaxed",
                        placeholder: "> Paste or type your secret here...",
                        autocomplete: "off",
                        autocorrect: "off",
                        autocapitalize: "off",
                        spellcheck: "false",
                        oninput: move |evt| secret_text.set(evt.value()),
                    }
                    div { class: "absolute bottom-3 right-4 pointer-events-none",
                        p { class: format!("text-[10px] {}", if is_over_limit { "text-red-500" } else { "text-[#444]" }),
                            "{char_count} / {MAX_PLAINTEXT_BYTES} bytes"
                        }
                    }
                }
            }

            div { class: "flex flex-col sm:flex-row gap-4",
                div { class: "space-y-2 flex-1",
                    label { class: "block text-[10px] uppercase tracking-widest text-[#555] flex items-center gap-1.5",
                        ClockIcon {}
                        "EXPIRES AFTER"
                    }
                    div { class: "relative",
                        select {
                            class: "w-full bg-black border border-[#222] rounded-lg p-3 text-[#ddd] focus:outline-none focus:border-[#444] appearance-none text-xs sm:text-sm",
                            onchange: move |evt| {
                                if let Ok(val) = evt.value().parse::<i64>() {
                                    ttl_seconds.set(val);
                                }
                            },
                            option { value: "300", "5 minutes" }
                            option { value: "900", "15 minutes" }
                            option { value: "3600", "1 hour" }
                            option { value: "86400", selected: true, "24 hours" }
                        }
                        div { class: "pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3",
                            svg { width: "12", height: "12", view_box: "0 0 24 24", fill: "none", stroke: "#555", "stroke-width": "2", "stroke-linecap": "round", "stroke-linejoin": "round", polyline { points: "6 9 12 15 18 9" } }
                        }
                    }
                }
                div { class: "space-y-2 flex-1 opacity-50 cursor-not-allowed",
                    label { class: "block text-[10px] uppercase tracking-widest text-[#555] flex items-center gap-1.5",
                        ShieldIcon {}
                        "IP RESTRICTION"
                    }
                    input {
                        type: "text",
                        class: "w-full bg-black border border-[#222] rounded-lg p-3 text-[#ddd] placeholder-[#555] focus:outline-none appearance-none text-xs sm:text-sm cursor-not-allowed",
                        placeholder: "Optional",
                        disabled: true,
                    }
                }
            }

            button {
                class: "mt-2 w-full bg-transparent border border-[#222] hover:bg-[#111] hover:text-white text-[#bbb] py-3 px-8 rounded-lg transition-colors duration-200 text-xs sm:text-sm flex items-center justify-center",
                onclick: create_secret,
                disabled: *is_loading.read() || is_over_limit,
                if *is_loading.read() {
                    span { class: "animate-pulse", "creating..." }
                } else {
                    span { "Create Secure Link" }
                }
            }

            if !error_msg.read().is_empty() {
                div { class: "mt-2 text-red-500 text-xs p-3 bg-red-950/20 border border-red-900/50 rounded-lg",
                    "{error_msg}"
                }
            }
        }
    }
}
