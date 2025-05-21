use std::{
    fs,
    fs::{File},
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
    collections::HashMap,
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

    /*
    let mut database_connection = DatabaseConnectionPool::get_connection(&database_connections).unwrap();
    let data = database_connection.query("SELECT * FROM users");
    database_connections.release_connection(database_connection);

    let json = json_encode(&data);
    println!("json: {}", json);
    */

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
    
    let mut database_connection = DatabaseConnectionPool::get_connection(&database_connections).unwrap();
    let db_data = database_connection.query("SELECT * FROM users");

    //debug
    //println!("response from DB: {:?}", db_data[0]);
    //debug

    let _json = json_encode(&db_data);
    database_connections.release_connection(database_connection);

    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!new connection");

    //create empty read to read stream into
    let mut buf_reader = BufReader::new(&stream);

    let request_line = read_request_line(&mut buf_reader);

    //debug
    println!("request line: {}", request_line);
    //debug

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
    let header_accept = match http_headers.contains_key("Accept") {
        true => Some(parse_header_accept(&http_headers["Accept"])),
        false => None,

    };

    let content_type = match http_headers.get("Content-Type") {
        Some(ctype) => ctype.to_string(),
        None => String::from("no content type defined"),
    };

    //variable for body data
    let body: String;

    //if its a post request, check if there is a body
    if let Method::POST = request_line.method {

        //TODO next lines do not really make sense, could be reduced
        //create option for content lengh
        let content_length: Option<usize> = match http_headers.get("Content-Length") {
            Some(length) => Some(length.parse().unwrap()), //TODO handling, if parse fails
            None => None,
        };

        //read the content of the body of the post request
        if let Some(clength) = content_length {
            println!("content length: {}", clength);
            body = read_http_body(&mut buf_reader, clength);
            println!("body: {}", body);
        }
    }

    let full_request = HTTPRequest {
        stream: stream,
        request_line: request_line 
    };

    //DEBUG
    //println!("content type: {}", content_type);
    //println!("path: {}", request_line.path);
    //DEBUG

    send_http_response(full_request);

    /*
    //variable to hold the content length of the body
    let mut content_length: usize = 0;

    //varible to hold the content type
    let mut content_type = String::new();

    let mut body_hash = HashMap::new();

    if method == "POST" {


        let status_line = "HTTP/1.1 200 OK";

        //let json = "{\"user\":\"ramin\"}";
        //let json = json.trim();
        let length = json.chars().count();

        let response = 
            format!("{status_line}\r\nContent-Length: {length}\r\nContent-Type: application/json\r\n\r\n{json}");

        stream.write_all(response.as_bytes()).unwrap();


          
    } else {

        if &request_line[..] == "GET /favicon.ico HTTP/1.1" {
            println!("fav icon get");
            let mut favicon_content = Vec::new();
            let path_favicon = format!("{}/favicon.png", ROOT);
            println!("path to favicon: {}", path_favicon);
            let mut file = File::open(path_favicon).unwrap();
            file.read_to_end(&mut favicon_content).unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/png/r/nContent-Length: {}\r\n\r\n",
                favicon_content.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.write_all(&favicon_content).unwrap();

            return;
        }

        let (status_line, filename) = match &request_line[..] {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
            "GET /sleep HTTP/1.1" => {
                thread::sleep(Duration::from_secs(5));
                ("HTTP/1.1 200 OK", "hello.html")
            },
            _ => ("HTTP/1.1 200 OK", "404.html")

        };

        let contents = fs::read_to_string(filename).unwrap();
        let length = contents.len();

        let response = 
            format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

        stream.write_all(response.as_bytes()).unwrap();
    }

    let status_line = "HTTP/1.1 200 OK";
    let contents = fs::read_to_string("hello.html").unwrap();
    let length = contents.len();

    let response = 
    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    */

}
