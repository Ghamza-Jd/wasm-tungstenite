use crate::safe_ws::SafeWebSocket;
use futures_util::future::Either;
use js_sys::wasm_bindgen::closure::Closure;
use js_sys::wasm_bindgen::JsCast;
use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;
use std::task::Waker;
use web_sys::BinaryType;
use web_sys::Event;
use web_sys::MessageEvent;

pub struct WebSocketStream {
    inner: SafeWebSocket,
    waker: Arc<Mutex<Option<Waker>>>,
    queue: Arc<Mutex<VecDeque<String>>>,
}

impl WebSocketStream {
    pub async fn new(url: &str) -> crate::Result<Self> {
        let ws = SafeWebSocket::new(url)?;
        ws.set_binary_type(BinaryType::Arraybuffer);

        // Setting the onopen callback
        let (open_tx, open_rx) = futures_channel::oneshot::channel();
        let on_open_cb = {
            let mut open_tx = Some(open_tx);
            Closure::wrap(Box::new(move |_e| {
                open_tx.take().map(|open_tx| open_tx.send(()));
            }) as Box<dyn FnMut(Event)>)
        };
        ws.set_onopen(Some(on_open_cb.as_ref().unchecked_ref()));

        // Settings the onerror callback
        let (err_tx, err_rx) = futures_channel::oneshot::channel();
        let on_err_cb = {
            let mut err_tx = Some(err_tx);
            Closure::wrap(Box::new(move |_e| {
                err_tx.take().map(|err_tx| err_tx.send(()));
            }) as Box<dyn FnMut(Event)>)
        };
        ws.set_onerror(Some(on_err_cb.as_ref().unchecked_ref()));

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

        // Setting the onmessage callback to push the event and wake the task.
        let waker = Arc::new(Mutex::new(Option::<Waker>::None));
        let queue = Arc::new(Mutex::new(VecDeque::new()));

        let on_message_cb = {
            let waker = waker.clone();
            let queue = queue.clone();

            Closure::wrap(Box::new(move |_e: MessageEvent| {
                queue.lock().unwrap().push_back(String::new());
                if let Some(waker) = waker.lock().unwrap().borrow_mut().take() {
                    waker.wake();
                }
            }) as Box<dyn FnMut(MessageEvent)>)
        };
        ws.set_onmessage(Some(on_message_cb.as_ref().unchecked_ref()));

        Ok(Self {
            inner: ws,
            waker,
            queue,
        })
    }
}

impl Drop for WebSocketStream {
    fn drop(&mut self) {
        self.inner.set_onmessage(None);
    }
}
