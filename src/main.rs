use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
    collections::HashMap,
};
use webserver::ThreadPool;
use webserver::DatabaseConnection;


fn main() {
    println!("Server started");
    //db_query();
    let mut dbcon = DatabaseConnection::new("127.0.0.1", 5432, "smgadmin", "admin", "memeoff", 0);
    dbcon.query("SELECT * FROM users WHERE id = '0'");
    dbcon.query("SELECT * FROM users WHERE id = '1'");
    //DatabaseConnection::new("127.0.0.1", 5432, "smgadmin", "admin", "memeoff", 1);
    //DatabaseConnection::new("127.0.0.1", 5432, "smgadmin", "admin", "memeoff", 2);
    //DatabaseConnection::new("127.0.0.1", 5432, "smgadmin", "admin", "memeoff", 3);

    let listener = TcpListener::bind("212.132.120.118:7878").unwrap();
    let threadpool = ThreadPool::new(4);

    
    println!("after pool creation");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        
        threadpool.execute(|| {
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

    let mut body_hash = HashMap::new();

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

            body_hash.insert(key_trim, value_trim);
        };

        println!("body: {}", body);
        println!("hashmap: {:?}", body_hash);
        println!("user: {}", body_hash.get("user").unwrap());

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

//write to database stream
fn write_to_db_stream (stream: &mut TcpStream, message: &Vec<u8>) {
    stream.write_all(message).unwrap();
    
}

//read from database stream
//read exact into vector
fn read_from_db_stream<R> (buf_reader: &mut BufReader<R>, response_vector: &mut Vec<u8>) where R: std::io::Read {
    buf_reader.read_exact(response_vector).unwrap(); 

}

#[allow(dead_code)]
fn db_query () {
    /*

    //send query
    let query = String::from("SELECT * FROM users");
    let mut query_vec_head: Vec<u8> = vec![];
    let mut query_vec_body: Vec<u8> = vec![];
    let mut query_length: i32 = 4;

    //add username
    for bytes in query.as_bytes() {
        query_vec_body.push(*bytes);
        query_length += 1;
    }

    //add null terminator after user string
    query_vec_body.push(0x00);
    query_length += 1;

    //add char tag
    query_vec_head.push(b'Q');


    //add password message length
    for bytes in query_length.to_be_bytes() {
        query_vec_head.push(bytes);
    }
    
    //merge vectors
    for bytes in query_vec_body {
        query_vec_head.push(bytes);
    }

    println!("{:?}", query_vec_head);
    
    write_to_db_stream(buf_reader.get_mut(), &query_vec_head);

    //retrieve query answer
    loop {
        //read the first five bytes to get message tag and length
        let mut db_response_after_query: Vec<u8> = vec![0; 5];
        read_from_db_stream(&mut buf_reader, &mut db_response_after_query);

        println!("query head: {:?}", db_response_after_query);

        //extract response length
        let db_response_length: i32 = i32::from_be_bytes(db_response_after_query[1..5].try_into().unwrap());

        let mut db_response_after_query_body: Vec<u8> = vec![0; db_response_length as usize - 4];
        read_from_db_stream(&mut buf_reader, &mut db_response_after_query_body);
        println!("query body: {:?}", db_response_after_query_body);

        if db_response_after_query[0] == 68 {
            println!("data row");
            //extract number of columns
            let number_of_columns_bytes = &db_response_after_query_body[0..2];
            let number_of_columns: i16 = i16::from_be_bytes(number_of_columns_bytes.try_into().unwrap());

            println!("number of columns: {}", number_of_columns);

            //column value length
            let column_value_length: i32 = i32::from_be_bytes(db_response_after_query_body[3..7].try_into().unwrap());
            println!("column value length: {}", column_value_length);
            
            let end: usize = 7 + column_value_length as usize;
            let value = &db_response_after_query_body[7..];
            println!("value: {:?}", value);
            let value = std::str::from_utf8(&value).unwrap();
            println!("value: {}", value);

        }

        if db_response_after_query[0] == 90 {
            break;
        }

    }
    */

}
