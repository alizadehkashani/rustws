use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use webserver::ThreadPool;

fn main() {
    println!("Server started");
    let listener = TcpListener::bind("212.132.120.118:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("new connection");
    let buf_reader = BufReader::new(&stream);

    println!("about to read new request line");

    let request_line = match buf_reader.lines().next() {
        Some(Ok(x)) => x,
        Some(Err(_e)) => String::from("{e}"),
        None => String::from("empty"),
    };

    println!("{request_line}");

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 200 OK", "404.html")

    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = 
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();

}
