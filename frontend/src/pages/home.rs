use chrono::{DateTime, Local};
use leptos::ev::MessageEvent;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys::{CloseEvent, Event, WebSocket};
use shared::SensorData;
use wasm_bindgen::prelude::Closure;

#[component]
pub fn Home() -> impl IntoView {
    let data = RwSignal::new(None::<SensorData>);

    Effect::new(move |_| {
        let ws = WebSocket::new("ws://127.0.0.1:8080/ws").unwrap();

        let on_open = Closure::wrap(Box::new(move |_: Event| {
            web_sys::console::log_1(&"✅ WebSocket connected".into());
        }) as Box<dyn FnMut(_)>);
        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();

        let on_close = Closure::wrap(Box::new(move |_: CloseEvent| {
            web_sys::console::warn_1(&"❌ WebSocket closed".into());
        }) as Box<dyn FnMut(_)>);
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        on_close.forget();

        let on_message = {
            let data = data.clone();
            Closure::wrap(Box::new(move |e: MessageEvent| {
                if let Some(txt) = e.data().as_string() {
                    match serde_json::from_str::<SensorData>(&txt) {
                        Ok(parsed) => data.set(Some(parsed)),
                        Err(e) => {
                            web_sys::console::error_1(&format!("❌ JSON parse error: {e}").into())
                        }
                    }
                } else {
                    web_sys::console::warn_1(&"⚠️ Received non-string WebSocket data".into());
                }
            }) as Box<dyn FnMut(_)>)
        };
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();

        || drop(ws)
    });

    view! {
        <main class="p-4 max-w-md mx-auto">
            <h1 class="text-2xl font-bold mb-4 text-center">"Live Marine Sensor Data"</h1>
            <Show
                when=move || data.get().is_some()
                fallback=|| {
                    view! {
                        <div class="bg-white rounded-xl shadow-md p-6 flex flex-col gap-4 border border-gray-300">
                            <p class="text-gray-500 text-center italic">
                                "Waiting for sensor data..."
                            </p>
                        </div>
                    }
                }
            >
                {move || {
                    let d = data.get().unwrap();
                    let formatted_time = DateTime::parse_from_rfc3339(&d.timestamp)
                        .map(|dt| dt.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|_| d.timestamp.clone());

                    view! {
                        <div class="bg-white rounded-xl shadow-md p-6 flex flex-col gap-4 border border-gray-300">
                            <div class="flex justify-between">
                                <span class="font-semibold text-gray-600">"Timestamp:"</span>
                                <span>{formatted_time}</span>
                            </div>
                            <div class="flex justify-between">
                                <span class="font-semibold text-gray-600">"Temperature:"</span>
                                <span class="text-blue-700">
                                    {format!("{:.2} °C", d.temperature)}
                                </span>
                            </div>
                            <div class="flex justify-between">
                                <span class="font-semibold text-gray-600">"Salinity:"</span>
                                <span class="text-green-700">
                                    {format!("{:.2} PSU", d.salinity)}
                                </span>
                            </div>
                            <div class="flex justify-between">
                                <span class="font-semibold text-gray-600">"Turbidity:"</span>
                                <span class="text-yellow-700">
                                    {format!("{:.2} NTU", d.turbidity)}
                                </span>
                            </div>
                        </div>
                    }
                }}
            </Show>
        </main>
    }
}
