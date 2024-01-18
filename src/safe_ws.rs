use std::ops::Deref;
use std::ptr::NonNull;
use web_sys::WebSocket;

pub struct SafeWebSocket {
    ws: NonNull<WebSocket>,
}

unsafe impl Send for SafeWebSocket {}
unsafe impl Sync for SafeWebSocket {}

impl SafeWebSocket {
    pub fn new(url: &str) -> crate::Result<Self> {
        let ws = WebSocket::new(url);
        match ws {
            Ok(ws) => Ok(Self {
                ws: NonNull::from(Box::leak(Box::new(ws))),
            }),
            Err(_) => Err(crate::Error::UnsupportedUrlScheme),
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
        _ = self.data().close();
        unsafe {
            drop(Box::from_raw(self.ws.as_ptr()));
        }
    }
}
