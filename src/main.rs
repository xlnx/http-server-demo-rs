use std::io::{prelude::*, BufRead, BufReader, Result};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::thread::{self, spawn};
use std::{env, fs};

struct ServerConfig<'a> {
    root: &'a str,
}

fn handle_request(buf: &String, stream: &mut TcpStream, conf: &ServerConfig) -> Result<()> {
    let lines: Vec<&str> = buf.split('\n').collect();

    let request_line: Vec<&str> = lines[0].split(' ').collect();
    match request_line[0..=2] {
        [request_method, request_url, protocol_version] => match request_method {
            "GET" => {
                let mut path = PathBuf::from(conf.root);
                let request_path: Vec<&str> = request_url.split('?').collect();
                path.push(String::from(".") + request_path[0]);
                if path.is_dir() {
                    path.push("index.html");
                }
                println!("GET {:?}", path);
                let content_type = match path.extension() {
                    Some(ext) => match ext.to_str() {
                        Some(ext) => match ext {
                            "html" | "htm" | "htx" => "text/html",
                            "css" => "text/css",
                            "js" => "application/x-javascript",
                            "jpg" | "jpeg" => "image/jpeg",
                            "png" => "image/png",
                            "gif" => "image.gif",
                            "woff" => "application/font-woff",
                            _ => "text/plain",
                        },
                        _ => "text/plain",
                    },
                    None => "text/plain",
                };
                let content = fs::read(path)?;
                let response = format!(
                    "{} 200 OK\r\nServer: koishi\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\n\r\n",
                    protocol_version,
                    content_type,
                    content.len()
                );
                stream.write(response.as_bytes())?;
                stream.flush()?;
                stream.write(content.as_ref())?;
            }
            "POST" => {
                println!("POST {}", request_url);
            }
            _ => (),
        },
        _ => (),
    }

    Ok(())
}

fn client(mut stream: TcpStream, conf: &ServerConfig) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut buf = String::new();
    loop {
        match reader.read_line(&mut buf) {
            Ok(0) => break,
            Err(e) => println!("{:?} occured {}", thread::current().id(), e),
            _ => match handle_request(&buf, &mut stream, &conf) {
                Err(e) => println!("[ Error ] {:?}", e),
                _ => (),
            },
        }
    }

    Ok(())
}

fn http_server(addr: &str, conf: &'static ServerConfig) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New incoming {:?}", stream);
                spawn(move || {
                    println!("[ Started ] {:?}", thread::current().id());
                    client(stream, conf).expect("Client went into error state");
                    println!("[ Exited ] {:?}", thread::current().id());
                });
            }
            Err(e) => println!("{}", e),
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let conf: &'static ServerConfig = &ServerConfig { root: "./cluster" };
    http_server(
        format!(
            "127.0.0.1:{}",
            if args.len() < 2 {
                "5140"
            } else {
                args[1].as_str()
            }
        )
        .as_str(),
        &conf,
    )
    .expect("Server went into error state");
}
