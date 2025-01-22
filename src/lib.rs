use std::{
    fs,
    io::{Read, Write},
    path::Path,
    ptr::eq,
    sync::Arc,
    time::Duration,
};

use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time::interval
};

use chrono::{Days, Utc};

use flate2::write::GzEncoder;
use flate2::Compression;

use httparse::Request;
struct HttpError {
    message: String,
    err_data: Vec<u8>,
}

impl From<io::Error> for HttpError {
    fn from(error: io::Error) -> Self {
        HttpError {
            message: error.to_string(),
            err_data: vec![],
        }
    }
}

#[warn(non_snake_case)]
struct RequestConfig {
    request_path: Option<String>,
    keep_alive: bool,
    encoder: bool,
}

impl RequestConfig {
    fn new() -> Self {
        RequestConfig {
            request_path: None,
            keep_alive: false,
            encoder: false,
        }
    }

    fn set_encode(&mut self, gzip_value: &str) {
        self.encoder = gzip_value.contains("gzip");
    }

    fn set_keep_alive(&mut self, keep_alive_value: &str) {
        self.keep_alive = eq(keep_alive_value, "keep-alive");
    }

    fn set_request_path_is_exist(&mut self, top_path: &str, uri_path: &str) -> bool {
        let mut full_path: String = String::from(top_path);
        full_path += uri_path;
        if full_path.ends_with('/') {
            full_path += "index.html"
        }
        let file_path: &Path = Path::new(&full_path);
        if file_path.exists() {
            self.request_path = Some(full_path);
            true
        } else {
            false
        }
    }

    fn set_content_type(&mut self) -> Result<Vec<u8>, HttpError> {
        let mut return_buffer: Vec<u8> = vec![];
        let file_name = match &self.request_path {
            Some(file) => file,
            None => {
                let err_message = "There is No File For Request";
                return Err(HttpError {
                    message: err_message.to_string(),
                    err_data: "HTTP/1.1 404 OK\r\n\r\n".as_bytes().to_vec(),
                });
            }
        };
        match file_name.find('.') {
            Some(pox) => {
                let _file_type = match &file_name[pox..] {
                    ".html" => "text/html; charset=UTF-8",
                    ".css" => "text/css",
                    ".bmp" => "application/x-bmp",
                    ".img" => "application/x-img",
                    ".jpe" => "image/jpeg",
                    ".jpeg" => "image/jpeg",
                    ".jpg" => "image/jpeg",
                    ".js" => "application/x-javascript",
                    ".mp4" => "video/mpeg4",
                    ".xml" => "	text/xml",
                    ".xquery" => "text/xml",
                    ".xsl" => "text/xml",
                    _ => "application/octet-stream",
                };

                let file_type = format!("HTTP/1.1 200 OK\r\nContent-Type: {}\r\n", _file_type);
                return_buffer.extend_from_slice(file_type.as_bytes());
            }
            None => {
                let err_message = format!("The File Name {} Error,No Dot", file_name);
                return Err(HttpError {
                    message: err_message,
                    err_data: "HTTP/1.1 404 OK\r\n\r\n".as_bytes().to_vec(),
                });
            }
        }
        Ok(return_buffer)
    }

    fn sl_http_fill_file_buffer(&mut self) -> Result<Vec<u8>, HttpError> {
        let mut file_buff: Vec<u8> = Vec::new();
        let file_name = match &self.request_path {
            Some(files) => files,
            None => {
                let err_message = "There Is No File For Request";
                return Err(HttpError {
                    message: err_message.to_string(),
                    err_data: vec![],
                });
            }
        };
        let files = fs::OpenOptions::new().read(true).open(file_name);
        match files {
            Ok(mut files) => match files.read_to_end(&mut file_buff) {
                Ok(_) => {
                    if self.encoder {
                        let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
                        let gzip_buff_result = match gzip.write_all(&file_buff) {
                            Ok(_) => match gzip.finish() {
                                Ok(buffer) => buffer,
                                Err(_) => {
                                    let err_message =
                                        format!("The File ({}) Encoder Gzip Error", file_name);
                                    return Err(HttpError {
                                        message: err_message,
                                        err_data: vec![],
                                    });
                                }
                            },
                            Err(_) => {
                                let err_message =
                                    format!("The File ({}) Encoder Gzip Error", file_name);
                                return Err(HttpError {
                                    message: err_message,
                                    err_data: vec![],
                                });
                            }
                        };
                        let mut file_content =
                            format!("Content-Length: {}\r\n\r\n", gzip_buff_result.len())
                                .as_bytes()
                                .to_vec();
                        file_content.extend(gzip_buff_result);
                        Ok(file_content)
                    } else {
                        let mut file_content =
                            format!("Content-Length: {}\r\n\r\n", file_buff.len())
                                .as_bytes()
                                .to_vec();
                        file_content.extend(file_buff);
                        Ok(file_content)
                    }
                }
                Err(_) => {
                    let err_message = format!("Can't Read The File ({})", file_name);
                    Err(HttpError {
                        message: err_message,
                        err_data: file_buff,
                    })
                }
            },
            Err(_) => {
                let err_message = format!("Can't Open The File ({})", file_name);
                Err(HttpError {
                    message: err_message,
                    err_data: file_buff,
                })
            }
        }
    }

    fn static_http_process_request(&mut self, top_path: &str, recv_request: &[u8]) -> Vec<u8> {
        let mut send_buffer: Vec<u8> = Vec::new();
        let mut header = [httparse::EMPTY_HEADER; 64];
        let mut request = Request::new(&mut header);
        request.parse(recv_request).unwrap();
        if !self.set_request_path_is_exist(top_path, request.path.unwrap()) {
            send_buffer.extend_from_slice(b"HTTP/1.1 404 OK\r\n\r\n");
            return send_buffer;
        }

        let response_code = match self.set_content_type() {
            Ok(response) => response,
            Err(e) => {
                println!("{}", e.message);
                return e.err_data;
            }
        };

        send_buffer.extend(response_code);

        for header_item in request.headers {
            if header_item.name.eq_ignore_ascii_case("Accept-Encoding") {
                self.set_encode(std::str::from_utf8(header_item.value).unwrap());
            }
            if header_item.name.eq_ignore_ascii_case("Connection") {
                self.set_keep_alive(std::str::from_utf8(header_item.value).unwrap());
            }
        }
        if self.encoder {
            send_buffer.extend_from_slice(b"Content-Encoding: gzip\r\n");
        }

        let now: chrono::DateTime<Utc> = Utc::now();
        let date = format!("Date: {}\r\n", now.format("%a, %d %b %Y %H:%M:%S GMT"));
        send_buffer.extend_from_slice(date.as_bytes());

        let expire = now.checked_add_days(Days::new(3)).unwrap();
        let expire_date = format!("expires: {}\r\n", expire.format("%a, %d %b %Y %H:%M:%S GMT"));
        send_buffer.extend_from_slice(expire_date.as_bytes());

        send_buffer.extend_from_slice(b"Server: StaticHttp\r\n");

        let file_content = match self.sl_http_fill_file_buffer() {
            Ok(file) => file,
            Err(e) => {
                println!("{}", e.message);
                e.err_data
            }
        };
        send_buffer.extend(file_content);
        send_buffer
    }
}

#[allow(non_snake_case)]
pub struct HttpServer<'a> {
    listen: Option<tokio::net::TcpListener>,

    paths: Option<&'a str>,

    keep_alive_timeouts: Option<u64>,
}

impl Default for HttpServer<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> HttpServer<'a> {
    pub fn new() -> Self {
        HttpServer {
            paths: None,
            listen: None,
            keep_alive_timeouts: None,
        }
    }

    pub fn set_keep_alive(mut self, keep_alive: u64) -> Self {
        self.keep_alive_timeouts = Some(keep_alive);
        self
    }

    pub async fn bind(mut self, ip_port: &'a str) -> Result<Self, ()> {
        let tcp_listen = match TcpListener::bind(ip_port).await {
            Ok(tcp_listen) => tcp_listen,
            Err(_) => {
                let err_message = format!("Can't Bind The Addr {}", ip_port);
                println!("{}", err_message);
                return Err(());
            }
        };
        self.listen = Some(tcp_listen);
        Ok(self)
    }

    pub async fn run(self) {
        if self.paths.is_none() {
            println!("Please Fill in The Route Path \".route()\"");
            return;
        }
        let listener = match self.listen {
            Some(listener) => listener,
            None => {
                let err_message = "There is No Listener Used";
                println!("{}", err_message);
                return;
            }
        };
        println!("The Server Start Listen!");
        let top_path = match self.paths {
            Some(paths) => paths.to_string(),
            None => {
                println!("StaticHttp Path Error");
                return;
            }
        };
        let top_path_with_lifetime = Arc::new(top_path);

        loop {
            match listener.accept().await {
                Ok((socket, _)) => {
                    let keepalive_timeout = self.keep_alive_timeouts.unwrap();
                    let top_path = Arc::clone(&top_path_with_lifetime);
                    tokio::spawn(async move {
                        static_http_handle_process(&top_path, socket, keepalive_timeout).await;
                    });
                }
                Err(_) => {
                    let err_message = "The Listener Accept Error,Please Check The Addr Port";
                    println!("{}", err_message);
                    return;
                }
            }
        }
    }

    pub fn route(mut self, route_path: &'a str) -> Result<Self, ()> {
        if !route_path.is_empty() {
            self.paths = Some(route_path);
        } else {
            let err_message = "The Route Path Is Null";
            println!("{}", err_message);
            return Err(());
        }
        Ok(self)
    }
}

async fn static_http_handle_process(top_path: &str, mut stream: TcpStream, timeout: u64) {
    let (interval_tick_tx, mut interval_tick_rx) = mpsc::channel::<bool>(16);
    let (recv_it_tick_tx, mut recv_it_tick_rx) = mpsc::channel::<bool>(16);
    let mut interval = interval(Duration::from_secs(1));
    let mut tick :u64  = 0;
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    tick+=1;
                    if tick > timeout {
                        interval_tick_tx.send(true).await.unwrap();
                        break;
                    }
                }

                is_recv_request = recv_it_tick_rx.recv() => {
                    if is_recv_request.unwrap() {
                        tick = 0;
                    } else {
                        break;
                    }
                }
            }
        }
    });
    loop {
        let mut request_config = RequestConfig::new();
        let mut recv_request_buffer: [u8; 2048] = [0; 2048];
        tokio::select! {
            tick = interval_tick_rx.recv() => {
                if tick.unwrap() {
                    break;
                }
            }

            recv_size = stream.read(&mut recv_request_buffer) => {
                let recv_size = recv_size.unwrap();
                if recv_size == 0 {
                    recv_it_tick_tx.send(false).await.unwrap();
                    break;
                }
                recv_it_tick_tx.send(true).await.unwrap();
                let capture_request_buffer = &recv_request_buffer[..recv_size];
                let send_buffer = request_config.static_http_process_request(top_path, capture_request_buffer);
                stream.write_all(&send_buffer).await.unwrap();
                if !request_config.keep_alive {
                    recv_it_tick_tx.send(false).await.unwrap();
                    break;
                }
            }
        }
    };
    
    stream.shutdown().await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn http_server() {
        // HttpServer::new()
        //     .bind("127.0.0.1:789")
        //     .await
        //     .unwrap()
        //     .route("C:\\Users\\Desktop\\html")
        //     .unwrap()
        //     .run()
        //     .await
    }
}
