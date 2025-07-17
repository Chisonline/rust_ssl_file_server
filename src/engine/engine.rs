use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::Arc,
};

use base64::{Engine as _, engine::general_purpose};
use dashmap::DashMap;
use env_logger::fmt::style::{self, RgbColor};
use log::*;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::time::Duration;
use crate::{engine::return_code::{into_handler, Handler, ReturnCode}, make_failed_resp};

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

    pub fn register<F, Fut>(&mut self, path: &str, func: F) -> &mut Self
    where 
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ReturnCode> + Send + 'static,
    {
        let wrapper = into_handler(func);
        self.register.insert(path.to_string(), wrapper);
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

    
    async fn run_handler(&self, method: &str, arg: String) -> Option<ReturnCode> 
    {
        if let Some(entry) = self.register.get(method) {
            let func = entry.value();
            Some(func.call(arg).await)
        } else {
            None
        }
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

        let style = style::Style::new().bold().fg_color(Some(style::Color::Rgb(RgbColor(0, 164, 164))));

        info!("Listening at {style}{}:{}{style:#}", self.addr, self.port);

        Ok((acceptor, listener))
    }

    fn log_engine_info(&self) {

        println!(r#" 
          _____  ______ _____ _      ______  
         |  __ \|  ____|_   _| |    |  ____| 
         | |__) | |__    | | | |    | |__    
         |  _  /|  __|   | | | |    |  __|   
         | | \ \| |     _| |_| |____| |____  
         |_|  \_\_|    |_____|______|______| 
                                             
        "#);

        let style = style::Style::new().bold().fg_color(Some(style::Color::Rgb(RgbColor(96, 196, 0))));

        let reg = Arc::clone(&self.register);
        reg.iter().for_each(|entry| {
            info!("Register handler: {style}{}{style:#}", entry.key());
        });
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.log_engine_info();

        let (acceptor, listener) = self.build()?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    debug!("new connection established");

                    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
                    stream.set_write_timeout(Some(Duration::from_secs(30)))?;

                    let acceptor_clone = Arc::clone(&acceptor);
                    let register = Arc::clone(&self.register);

                    tokio::spawn(async move {
                        debug!("Starting SSL handshake");

                        match acceptor_clone.accept(stream) {
                            Ok(mut ssl_stream) => {
                                debug!("SSL handshake success");

                                let mut metadata_buffer = [0; 1024];

                                match ssl_stream.read(&mut metadata_buffer) {
                                    Ok(n) => {
                                        let metadata_str =
                                            String::from_utf8_lossy(&metadata_buffer[..n]);
                                        let parts: Vec<&str> = metadata_str.split(' ').collect();
                                        let method = parts[0];
                                        if register.contains_key(method) {
                                            debug!("enter handler {}", method);
                                            
                                            let result = if let Some(handler) = register.get(method) {
                                                handler.call(metadata_str.to_string()).await
                                            } else {
                                                make_failed_resp!(payload: "method not found")
                                            };
                                            
                                            debug!("Resp: {:?}", result);

                                            let response = format!(
                                                "{} {} {}\n",
                                                result.success,
                                                if let Some(control_block) = result.control_block {
                                                    let control_block =
                                                        serde_json::to_string(&control_block)
                                                            .unwrap();
                                                    general_purpose::STANDARD.encode(&control_block)
                                                } else {
                                                    ".".to_string()
                                                },
                                                if let Some(payload) = result.payload {
                                                    general_purpose::STANDARD.encode(payload)
                                                } else {
                                                    "".to_string()
                                                }
                                            );

                                            if let Err(e) =
                                                ssl_stream.write_all(response.as_bytes())
                                            {
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
                        }
                    });
                }
                Err(e) => {
                    error!("failed to establish TCP connection: {}", e);
                }
            }
        }
        Ok(())
    }
}
