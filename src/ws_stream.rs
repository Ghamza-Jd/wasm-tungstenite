use crate::safe_ws::SafeWebSocket;
use futures_util::future::Either;
use js_sys::wasm_bindgen::closure::Closure;
use js_sys::wasm_bindgen::JsCast;
use web_sys::BinaryType;
use web_sys::Event;

pub struct WebSocketStream {
    inner: SafeWebSocket,
}

impl WebSocketStream {
    pub async fn new(url: &str) -> crate::Result<Self> {
        let ws = SafeWebSocket::new(url)?;
        ws.set_binary_type(BinaryType::Arraybuffer);

        // Setting the onopen callback
        let (open_tx, open_rx) = futures_channel::oneshot::channel();
        let on_open_callback = {
            let mut open_tx = Some(open_tx);
            Closure::wrap(Box::new(move |_e| {
                open_tx.take().map(|open_tx| open_tx.send(()));
            }) as Box<dyn FnMut(Event)>)
        };
        ws.set_onopen(Some(on_open_callback.as_ref().unchecked_ref()));

        // Settings the onerror callback
        let (err_tx, err_rx) = futures_channel::oneshot::channel();
        let on_err_callback = {
            let mut err_tx = Some(err_tx);
            Closure::wrap(Box::new(move |_e| {
                err_tx.take().map(|err_tx| err_tx.send(()));
            }) as Box<dyn FnMut(Event)>)
        };
        ws.set_onerror(Some(on_err_callback.as_ref().unchecked_ref()));

        // Await futures
        // if open_tx resolves => success
        // if err_tx resolves => failure
        let result = futures_util::future::select(open_rx, err_rx).await;
        ws.set_onopen(None);
        ws.set_onerror(None);
        let ws = match result {
            Either::Left((_, _)) => ws,
            Either::Right((_, _)) => return Err(crate::Error::ConnectionClosed),
        };

        Ok(Self { inner: ws })
    }
}
