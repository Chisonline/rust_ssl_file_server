use std::{
    io::{Read, Write}, net::TcpListener, pin::Pin, sync::Arc
};

use dashmap::DashMap;
use log::*;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use crate::control_block::ControlBlock;

pub struct ReturnCode {
    pub success: bool,
    pub payload: Option<String>,
    pub control_block: Option<ControlBlock>,
}

pub struct SyncHandler(pub fn(&str) -> ReturnCode);
pub struct AsyncHandler(pub fn(&str) -> Pin<Box<dyn Future<Output = ReturnCode> + Send>>);


pub enum Handler {
    Sync(SyncHandler),
    Async(AsyncHandler),
}

impl Handler {
    async fn run(&self, param: &str) -> ReturnCode {
        match self {
            Handler::Async(async_handler) => async_handler.run(param).await,
            Handler::Sync(sync_handler) => sync_handler.run(param).await,
        }
    }
}

pub trait IntoHandler {
    fn into_handler(self) -> Handler;
    async fn run(&self, param: &str) -> ReturnCode;
}

impl IntoHandler for SyncHandler {
    fn into_handler(self) -> Handler {
        Handler::Sync(self)
    }
    async fn run(&self, param: &str) -> ReturnCode {
        (self.0)(param)
    }
}

impl IntoHandler for AsyncHandler {
    fn into_handler(self) -> Handler {
        Handler::Async(self)
    }
    async fn run(&self, param: &str) -> ReturnCode {
        (self.0)(param).await
    }
}

#[derive(Default)]
pub struct Engine {
    register: Arc<DashMap<String, Handler>>,
    private_key_file: String,
    cert_file: String,
    addr: String,
    port: u16,
}

#[allow(unused)]
impl Engine {
    pub fn new() -> Self {
        Engine {
            register: Arc::new(DashMap::new()),
            private_key_file: "private.key".to_string(),
            cert_file: "certificate.crt".to_string(),
            addr: "127.0.0.1".to_string(),
            port: 7878,
        }
    }

    pub fn register<H>(&mut self, path: &str, func: H) -> &mut Self 
        where H: IntoHandler
    {
        let handler = IntoHandler::into_handler(func);
        self.register.insert(path.to_string(), handler);
        self
    }

    pub fn set_private_key_file(&mut self, file_path: &str) -> &mut Self {
        self.private_key_file = file_path.to_string();
        self
    }

    pub fn set_cert_file(&mut self, file_path: &str) -> &mut Self {
        self.cert_file = file_path.to_string();
        self
    }

    pub fn set_addr(&mut self, addr: &str) -> &mut Self {
        self.addr = addr.to_string();
        self
    }

    pub fn set_port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    fn build(&self) -> Result<(Arc<SslAcceptor>, TcpListener), Box<dyn std::error::Error>> {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
        builder.set_private_key_file(&self.private_key_file, SslFiletype::PEM)?;
        builder.set_certificate_chain_file(&self.cert_file)?;
        builder.check_private_key()?;
        let acceptor = builder.build();
        let acceptor = Arc::new(acceptor);

        let listener = TcpListener::bind(format!("{}:{}", &self.addr, &self.port))?;
        listener.set_nonblocking(false)?;

        Ok((acceptor, listener))
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (acceptor, listener) = self.build()?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    info!("new connection established");
                    let acceptor_clone = Arc::clone(&acceptor);
                    let register = Arc::clone(&self.register);
                    std::thread::spawn(async move || match acceptor_clone.accept(stream) {
                        Ok(mut ssl_stream) => {
                            debug!("SSL shakehand success");
                            if let Err(e) = ssl_stream.write_all(b"Hello, OpenSSL\n") {
                                warn!("Failed to send msg: {}", e);
                            }
                            let mut metadata_buffer = [0; 1024];
                            match ssl_stream.read(&mut metadata_buffer) {
                                Ok(n) => {
                                    let metadata_str =
                                        String::from_utf8_lossy(&metadata_buffer[..n]);
                                    let parts: Vec<&str> = metadata_str.split(' ').collect();
                                    let method = parts[0];
                                    if register.contains_key(method) {
                                        debug!("enter handler {}", method);
                                        let handler =
                                            (&register).get(method).unwrap();
                                        
                                        let result = handler.run(&metadata_str).await;
                                        let response = format!(
                                            "{} {} {}\n",
                                            result.success,
                                            if let Some(control_block) = result.control_block {
                                                serde_json::to_string(&control_block).unwrap()
                                            } else {
                                                ".".to_string()
                                            },
                                            if let Some(payload) = result.payload {
                                                payload
                                            } else {
                                                "".to_string()
                                            }
                                        );
                                        
                                        if let Err(e) = ssl_stream.write_all(response.as_bytes()) {
                                            warn!("Failed to send msg: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to read msg: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("SSL shakehand failed {}", e)
                        }
                    });
                }
                Err(e) => {
                    error!("failed to establish TCP connection");
                }
            }
        }
        Ok(())
    }
}
