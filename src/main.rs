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
    let mut buf_reader = BufReader::new(&stream);

    let mut request_line = String::new();

    buf_reader.read_line(&mut request_line).unwrap();

    let request_line = request_line.trim();

    println!("{}", request_line);

    //split status line by space
    let mut request_line_split_iter = request_line.split_whitespace();

    let method = request_line_split_iter.next().unwrap();
    let _path = request_line_split_iter.next().unwrap();
    let _protocol = request_line_split_iter.next().unwrap();



    let mut content_length: usize = 0;

    loop {

        let mut header_line = String::new();

        buf_reader.read_line(&mut header_line).unwrap();

        let header_line = header_line.trim();

        if header_line.is_empty() {
            //println!("empty");
            break;
        }

        if header_line.starts_with("Content-Length") {
            content_length = header_line.split_whitespace().nth(1).unwrap().parse().unwrap();
                   
            //println!("content length is: {content_length}");
        }


        //println!("{}", header_line);
    }

    let mut body_vec = Vec::new();

    if method == "POST" {
        //println!("post method used");

        let mut body: Vec<u8> = vec![0; content_length];

        buf_reader.read_exact(&mut body).unwrap(); 

        let body = std::str::from_utf8(&body).unwrap();
        let body_trim: &str = &body[1..body.len() - 1];
        println!("!!!body trim: {}", body_trim);
                
        for item in body_trim.split(",") {
            println!("{}", item);
            let mut item_iter = item.split(":");
            
            let key = item_iter.next().unwrap();
            let key_trim: &str = &key[1..key.len() - 1];
            println!("{}",key_trim);

            let value = item_iter.next().unwrap();
            let value_trim: &str = &value[1..value.len() - 1];
            println!("{}",value_trim);

            body_vec.push((key_trim, value_trim));
        };
 
        println!("{:?}", body_vec);

        println!("body: {}", body);

        let status_line = "HTTP/1.1 200 OK";

        let contents = "{\"user\":\"ramin\"}";
        let length = contents.chars().count();

        let response = 
            format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: application/json\r\n\r\n{contents}");

        stream.write_all(response.as_bytes()).unwrap();


          
    } else {

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

}
