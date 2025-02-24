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

//const ROOT: String = String::from("www");
const ROOT: &str = "www";


fn main() {

    println!("root path: {}", ROOT);
    println!("Server started"); let database_connections = Arc::new(DatabaseConnectionPool::new( 
        4, //number of connections 
        "127.0.0.1", //ip to database
        5432, //port to database
        "smgadmin", //user name
        "admin", //user password
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
    let threadpool = ThreadPool::new(4);

    println!("after pool creation");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let database_connections_clone = Arc::clone(&database_connections);
        
        threadpool.execute(|| {
            handle_connection(stream, database_connections_clone);
        });
    }

}

fn handle_connection(mut stream: TcpStream, database_connections: Arc<DatabaseConnectionPool>) {
    
    let mut database_connection = DatabaseConnectionPool::get_connection(&database_connections).unwrap();
    let db_data = database_connection.query("SELECT * FROM users");
    let json = json_encode(&db_data);
    database_connections.release_connection(database_connection);

    println!("!!!!!!!!!!!!!!!!!!!!!!!!!!new connection");
    //create empty read to read stream into
    let mut buf_reader = BufReader::new(&stream);

    let request_line = read_request_line(&mut buf_reader);

    //parse the request line
    let request_line = parse_request_line(request_line);

    //check if request line is empty
    if request_line.empty {
        println!("handle function says request line is empty");
        return;
    }

    //parse the headers
    parse_http_headers(&mut buf_reader);

    /*
    //variable to hold the content length of the body
    let mut content_length: usize = 0;

    //varible to hold the content type
    let mut content_type = String::new();

    let mut body_hash = HashMap::new();

    if method == "POST" {

        //create empty vector with the length of the content
        let mut body: Vec<u8> = vec![0; content_length];

        //read content into vector
        buf_reader.read_exact(&mut body).unwrap(); 

        //turn boty from bytes into a string
        let body = std::str::from_utf8(&body).unwrap();

        println!("body string: {}", body);

        //TODO turn json into hashmap
        //
        let body_trim: &str = &body[1..body.len() - 1];
                
        for item in body_trim.split(",") {
            let mut item_iter = item.split(":");
            
            let key = item_iter.next().unwrap();
            let key_trim: &str = &key[1..key.len() - 1];

            let value = item_iter.next().unwrap();
            let value_trim: &str = &value[1..value.len() - 1];

            body_hash.insert(key_trim, value_trim);
        };

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
    */

}
