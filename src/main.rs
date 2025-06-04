use std::{
    io::BufReader,
    net::{TcpListener, TcpStream},
    sync::Arc,
};
use webserver::*;

mod constants;

fn main() {

    println!("root path: {}", constants::ROOT);
    println!("Server started"); let database_connections = Arc::new(DatabaseConnectionPool::new( 
        4, //number of connections 
        "127.0.0.1", //ip to database
        5432, //port to database
        "smgadmin", //user name databae
        "admin", //user password database
        "memeoff" //database
    ));

    let listener = TcpListener::bind("212.132.120.118:7878").unwrap();
    let threadpool = ThreadPool::new(8);

    println!("after pool creation");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let database_connections_clone = Arc::clone(&database_connections);
        
        threadpool.execute(|| {
            handle_connection(stream, database_connections_clone);
        });
    }

}

fn handle_connection(stream: TcpStream, database_connections: Arc<DatabaseConnectionPool>) {

    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!new connection");

    //create empty read to read stream into
    let mut buf_reader = BufReader::new(&stream);

    let request_line = read_request_line(&mut buf_reader);

    //debug
    println!("request line: {}", request_line);

    //parse the request line
    let request_line = parse_request_line(request_line);

    //check if request line is empty
    if request_line.empty {
        println!("handle function says request line is empty");
        return;
    }

    //read the headers
    let http_headers = read_http_headers(&mut buf_reader);

    //if the header contains information for the accept of media type
    //parse the information
    let _header_accept = match http_headers.contains_key("Accept") {
        true => Some(parse_header_accept(&http_headers["Accept"])),
        false => None,

    };

    let _content_type = match http_headers.get("Content-Type") {
        Some(ctype) => ctype.to_string(),
        None => String::from("no content type defined"),
    };

    //TODO rewrite this, so that a body variable is only created, if there is content
    //variable for body data
    let mut body = String::from("");

    //if its a post request, check if there is a body
    if let Method::POST = request_line.method {

        //create option for content lengh
        let content_length: Option<usize> = match http_headers.get("Content-Length") {
            Some(length) => Some(length.parse().unwrap()), //TODO handling, if parse fails
            None => None,
        };

        //read the content of the body of the post request
        body = match content_length {
            Some(clength) => {
                read_http_body(&mut buf_reader, clength)
            },
            None => String::from("")
        };
    }

    let full_request = HTTPRequest {
        stream: stream,
        request_line: request_line ,
        body: body,
    };

    send_http_response(full_request, database_connections);

}
