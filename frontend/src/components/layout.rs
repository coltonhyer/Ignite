use super::icons::FlameIcon;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Layout() -> Element {
    rsx! {
        div { class: "bg-black text-gray-200 min-h-screen flex flex-col font-mono antialiased w-full h-full border-t border-[#222]",
            Header {}
            main {
                class: "flex-grow w-full max-w-[640px] mx-auto p-4 flex flex-col pt-12",
                Outlet::<Route> {}
            }
            Footer {}
        }
    }
}

pub fn Header() -> Element {
    let nav = use_navigator();
    rsx! {
        header {
            class: "w-full pt-16 pb-8",
            div {
                class: "flex flex-col items-center justify-center gap-2",
                h1 { class: "text-lg md:text-xl font-bold text-white flex items-center gap-2 cursor-pointer",
                    onclick: move |_| {
                        nav.push(Route::Home {});
                    },
                    span { class: "text-orange-500 flex items-center", FlameIcon {} }
                    span { class: "lowercase tracking-wide", "ignite" }
                }
                p { class: "text-[#555] text-xs lowercase", "share secrets that burn after reading" }
            }
        }
    }
}

pub fn Footer() -> Element {
    rsx! {
        footer { class: "mt-auto py-8 text-center text-[#444] text-[10px] tracking-wide",
            p { "end-to-end encrypted · zero knowledge · open source" }
        }
    }
}
