use std::ops::Deref;
use std::ptr::NonNull;
use web_sys::WebSocket;

pub struct SafeWebSocket {
    ws: NonNull<WebSocket>,
}

impl SafeWebSocket {
    pub fn new(url: &str) -> Self {
        let ws = WebSocket::new(url).unwrap();
        Self {
            ws: NonNull::from(Box::leak(Box::new(ws))),
        }
    }

    fn data(&self) -> &WebSocket {
        unsafe { self.ws.as_ref() }
    }
}

impl Deref for SafeWebSocket {
    type Target = WebSocket;

    fn deref(&self) -> &Self::Target {
        &self.data()
    }
}

impl Drop for SafeWebSocket {
    fn drop(&mut self) {
        self.data().close().unwrap();
        unsafe {
            drop(Box::from_raw(self.ws.as_ptr()));
        }
    }
}
