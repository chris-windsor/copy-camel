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

#[component]
pub fn App<G: Html>(cx: Scope) -> View<G> {
    let history_list = create_signal(cx, vec!["".to_string()]);

    on_mount(cx, move || {
        spawn_local_scoped(cx, async move {
            let history_msg = invoke("retrieve_history", to_value(&NoArgs {}).unwrap()).await;
            let history_msg = serde_wasm_bindgen::from_value::<Vec<String>>(history_msg).unwrap();
            history_list.set(history_msg);
        })
    });

    view! { cx,
        main(class="container") {
            "Hey there! Im the copy 🐪"
            div(class="container") {
                Indexed(
                    iterable=history_list,
                    view=|cx, x| view! { cx,
                        ClipboardEntry(content=x)
                    }
                )
            }
        }
    }
}

#[component(inline_props)]
fn ClipboardEntry<'a, G: Html>(cx: Scope<'a>, content: String) -> View<G> {
    view! {cx,
        div(class="clipboard-entry") {
            (content)
        }
    }
}
