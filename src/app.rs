use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use sycamore::futures::spawn_local_scoped;
use sycamore::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize)]
struct NoArgs {}

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub entries: RcSignal<Vec<RcSignal<String>>>,
}

impl AppState {
    fn add_entry(&self, content: String) {
        self.entries.modify().push(create_rc_signal(content))
    }
}

#[component]
pub fn App<G: Html>(cx: Scope) -> View<G> {
    let app_state = AppState {
        entries: Default::default(),
    };
    let _app_state = provide_context(cx, app_state);

    view! { cx,
        main(class="container") {
            "Hey there! Im the copy 🐪"
            div(class="container") {
                EntryList {}
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SetContentsArgs<'a> {
    content: &'a str,
}

#[component(inline_props)]
fn EntryItem<G: Html>(cx: Scope, content: RcSignal<String>) -> View<G> {
    let content_handle = create_ref(cx, content);

    let content = || content_handle.get();

    let click_handler = move |_| {
        spawn_local_scoped(cx, async move {
            invoke(
                "set_contents",
                to_value(&SetContentsArgs {
                    content: &content_handle.get(),
                })
                .unwrap(),
            )
            .await;
        })
    };

    view! { cx,
        li {
            div(class="view") {
                label(on:click=click_handler) {
                    (content())
                }
            }
        }
    }
}

#[component]
fn EntryList<G: Html>(cx: Scope) -> View<G> {
    let app_state = use_context::<AppState>(cx);

    on_mount(cx, move || {
        spawn_local_scoped(cx, async move {
            let history_msg = invoke("retrieve_history", to_value(&NoArgs {}).unwrap()).await;
            let history_msg = serde_wasm_bindgen::from_value::<Vec<String>>(history_msg).unwrap();
            for msg in history_msg {
                app_state.add_entry(msg);
            }
        })
    });

    let filtered_entries = create_memo(cx, || {
        app_state.entries.get().iter().cloned().collect::<Vec<_>>()
    });

    view! { cx,
        ul(class="entry-list") {
            Indexed(
                iterable=filtered_entries,
                view=|cx, content| view! { cx,
                    EntryItem(content=content)
                },
            )
        }
    }
}
