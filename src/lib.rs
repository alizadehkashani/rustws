use std::{
    sync::{mpsc, Arc, Mutex, Condvar},
    io::BufReader,
    io::Write,
    io::Read,
    net::TcpStream,
    thread,
    collections::{HashMap, VecDeque},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,

}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {

        //code will panic, if number of threads is zero
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers: workers, sender: Some(sender) }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop (&mut self) {

        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {

        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => { 
                    println!("Worker {id} got a job, executing.");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected");
                    break;
                }
            }

        });

        Worker { id, thread: Some(thread) }
    }

}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct DatabaseConnectionPool {
    connections: Arc<Mutex<VecDeque<DatabaseConnection>>>,
    condvar: Arc<Condvar>,

}

impl DatabaseConnectionPool {
    pub fn new (size: usize, ip: &str, port: u16, user: &str, password: &str, database: &str) -> DatabaseConnectionPool {
        let mut connections = VecDeque::new();
        for i in 0..size {
            connections.push_back(DatabaseConnection::new(ip, port, user, password, database, i));
        }
        
        DatabaseConnectionPool{ 
            connections: Arc::new(Mutex::new(connections)),
            condvar: Arc::new(Condvar::new())
        }
    }

    pub fn get_connection (&self) -> Option<DatabaseConnection> {
        let mut connections = self.connections.lock().unwrap();

        while connections.is_empty() {
            println!("tried to get db connection, but its empty");
            connections = self.condvar.wait(connections).unwrap();
        }

        connections.pop_front()
    }

    pub fn release_connection (&self, connection: DatabaseConnection) {
        let mut connections = self.connections.lock().unwrap();

        connections.push_back(connection);

        self.condvar.notify_one();
    }

}

pub struct DatabaseConnection {
    id: usize,
    reader: BufReader<TcpStream>,
}

impl DatabaseConnection {
    //---public----
    pub fn new (ip: &str, port: u16, user: &str, password: &str, database: &str, id: usize) -> DatabaseConnection {

        let stream = TcpStream::connect((ip, port)).unwrap(); 
        let mut reader = BufReader::new(stream);

        Self::send_startup(&mut reader, &user, &database);
        Self::read_authentication_method(&mut reader);
        Self::send_password(&mut reader, password);
        Self::read_authentication_response(&mut reader);
        Self::read_paramters(&mut reader);

        DatabaseConnection { id, reader } 
    }
    
    pub fn query(&mut self, query: &str) {
        Self::send_query(&mut self.reader, &query);
        Self::read_query_response(&mut self.reader);

    }
    

    //---private
    //write to database stream
    fn write_to_db_stream (stream: &mut TcpStream, message: &Vec<u8>) {
        stream.write_all(message).unwrap();
            
    }

    //read from database stream
    //read exact into vector
    fn read_from_db_stream (reader: &mut BufReader<TcpStream>, response_vector: &mut Vec<u8>) {
        reader.read_exact(response_vector).unwrap(); 
    }

    fn add_i16_as_be_bytes_to_vec (number: &i16, vector: &mut Vec<u8>) {
        for bytes in number.to_be_bytes() {
            vector.push(bytes);
        }
    }

    fn add_i32_as_be_bytes_to_vec (number: &i32, vector: &mut Vec<u8>) {
        for bytes in number.to_be_bytes() {
            vector.push(bytes);
        }
    }

    fn add_str_as_bytes_to_vec (string: &str, vector: &mut Vec<u8>) {
        for bytes in string.as_bytes() {
            vector.push(*bytes);
        }
    }


    fn send_startup (reader: &mut BufReader<TcpStream>, user: &str, database: &str) {

        //version
        let version_major: i16 = 3;
        let version_minor: i16 = 0;
        //variable to store the length of the startup message
        //will be at least 8 bytes for content length and protocol version bytes
        let mut content_length: i32 = 8;

        let mut startup_message: Vec<u8> = vec![];
        let mut startup_message_body: Vec<u8> = vec![];

        //add protocol version
        Self::add_i16_as_be_bytes_to_vec(&version_major, &mut startup_message_body);
        Self::add_i16_as_be_bytes_to_vec(&version_minor, &mut startup_message_body);

        //add 'key' for user
        Self::add_str_as_bytes_to_vec("user", &mut startup_message_body);
        //increase length according to key length in bytes
        content_length += "user".len() as i32;

        //add null terminator after user key
        startup_message_body.push(0x00);
        content_length += 1;

        //add username
        Self::add_str_as_bytes_to_vec(user, &mut startup_message_body);
        //increase length according to user name length in bytes
        content_length += user.len() as i32;

        //add null terminator after user string
        startup_message_body.push(0x00);
        content_length += 1;

        //add 'key' for database
        Self::add_str_as_bytes_to_vec("database", &mut startup_message_body);
        content_length += "database".len() as i32;

        //add null terminator after database key
        startup_message_body.push(0x00);
        content_length += 1;

        //add database
        Self::add_str_as_bytes_to_vec(database, &mut startup_message_body);
        content_length += database.len() as i32;

        //add null terminator after user string
        startup_message_body.push(0x00);
        content_length += 1;

        //add null terminator after end of startup message
        startup_message_body.push(0x00);
        content_length += 1;
        
        //add content length
        Self::add_i32_as_be_bytes_to_vec(&content_length, &mut startup_message);
        
        //merge vectors
        for bytes in startup_message_body {
            startup_message.push(bytes);
        }

        //println!("startup message: {:?}", startup_message);

        //send startup message to db server
        Self::write_to_db_stream(reader.get_mut(), &startup_message);

    }
    
    fn read_authentication_method (reader: &mut BufReader<TcpStream>) {
        //create vector to hold initial ascii char 1byte of reply and content length 4bytes
        let mut auth_response_head: Vec<u8> = vec![0; 9];
        
        Self::read_from_db_stream(reader, &mut auth_response_head);

        //turn ascii packet identifier into char
        let packet_identifier = char::from_u32(auth_response_head[0] as u32).unwrap();

        //make sure, that the response of the db matches an authentication request
        assert_eq!(packet_identifier, 'R', "response does not match authentication request");

        //read the inital packet an retrieve the individual bytes of the content length
        let db_response_length: i32 = i32::from_be_bytes(auth_response_head[1..5].try_into().unwrap());
        assert_eq!(db_response_length, 8, "length should be 8, 4 bytes length, 4 bytes int authm");

        //read authentication method, 3 equals plain text password
        let auth_method: i32 = i32::from_be_bytes(auth_response_head[5..].try_into().unwrap());

        //in this case, its always 3, plain text password
        assert_eq!(auth_method, 3, "authentication method must be plain password");
    }

    fn send_password (reader: &mut BufReader<TcpStream>, password: &str) {

        //send password
        let mut password_message: Vec<u8> = vec![];
        let mut password_message_body: Vec<u8> = vec![];
        let mut password_message_length: i32 = 4;

        //add passwordas plain text
        Self::add_str_as_bytes_to_vec(password, &mut password_message_body);
        //incease length according to password
        password_message_length += password_message_body.len() as i32;

        //add null terminator after user string
        password_message_body.push(0x00);
        password_message_length += 1;

        //add char tag
        password_message.push(b'p');

        //add password message length
        Self::add_i32_as_be_bytes_to_vec(&password_message_length, &mut password_message);
        
        //merge vectors
        for bytes in password_message_body {
            password_message.push(bytes);
        }

        //send password to database
        Self::write_to_db_stream(reader.get_mut(), &password_message);
    }
    
    fn read_authentication_response (reader: &mut BufReader<TcpStream>) {
        
        //create vector to read response
        //total resonse length should be 9
        //1 byte identifyer, 4 bytes length, 4 bythes response
        let mut auth_response_head: Vec<u8> = vec![0; 9];
        //read from stream into initial response
        Self::read_from_db_stream(reader, &mut auth_response_head);

        //check if auth response is OK
        //loop through the auth response body and extract the auth code
        //array holding the bytes of the authentication result
        let auth_response: i32 = i32::from_be_bytes(auth_response_head[5..].try_into().unwrap());

        //panic if connection failed
        assert_eq!(auth_response, 0, "authentication has not been accpeted");

    }

    fn read_paramters (reader: &mut BufReader<TcpStream>) {
        loop {
            let mut response: Vec<u8> = vec![0; 5];
            Self::read_from_db_stream(reader, &mut response);

            //check if there are incoming unexpeted messge types, should be either S, K or Z
            assert!(matches!(response[0], 83 | 75 | 90));

            //extract response length
            let response_length: i32 = i32::from_be_bytes(response[1..].try_into().unwrap());

            let mut body: Vec<u8> = vec![0; response_length as usize - 4];
            Self::read_from_db_stream(reader, &mut body);

            //90 equals Z which means connection is ready for query
            if response[0] == 90 {
                break;
            }

        }

    }
    
    fn send_query (reader: &mut BufReader<TcpStream>, query: &str) {
        
        let mut query_vec: Vec<u8> = vec![];
        let query_length: i32 = 5 + query.len() as i32;

        //add char tag
        query_vec.push(b'Q');
        
        //add length of message to vector in bytes
        Self::add_i32_as_be_bytes_to_vec(&query_length, &mut query_vec); 

        //add query to vector as bytes
        Self::add_str_as_bytes_to_vec(&query, &mut query_vec);

        //add null terminator after user string
        query_vec.push(0x00);
        
        //send query to tcp stream
        Self::write_to_db_stream(reader.get_mut(), &query_vec);

    }

    fn read_query_response (reader: &mut BufReader<TcpStream>) {
        //read row description
        let mut query_response_head: Vec<u8> = vec![0; 5];
        Self::read_from_db_stream(reader, &mut query_response_head);
        //get the length of the message
        let query_response_length: i32 = i32::from_be_bytes(query_response_head[1..].try_into().unwrap());

        //make sure, that the response is a row desciption
        //or error
        assert!(matches!(query_response_head[0], 69 | 84));

        

        //if error read error
        //and exit out of function
        if query_response_head[0] == 69 {
            //if the response is an error, read the error
            Self::read_error(reader, query_response_length);
            //after reading the error, check if the db is ready for a new query
            Self::read_ready_command(reader);
            return;
        }

        //create vector big enough to hold the rest of the message
        let mut row_description_body: Vec<u8> = vec![0; query_response_length as usize - 4];
        //read from stream
        Self::read_from_db_stream(reader, &mut row_description_body);

        //get the number of fields
        let number_of_fields: i16 = i16::from_be_bytes(row_description_body[..2].try_into().unwrap());

        //set array position to beginning for first value string
        let mut array_pos: usize = 2;
        //position where the field name starts
        let mut value_start_pos: usize = 2;
        //create a vector holding die individual field names
        let mut field_names: Vec<String> = Vec::new();
        //loop through fields
        for _field_number in 0..number_of_fields as usize {

            loop {
                //check if the current array position is larger than the actual message
                if array_pos > query_response_length as usize {
                    break;
                }

                //check if the current character is a null terminator
                //if no, advance one character
                //if yes, read the field name and move the position beyond the additional
                //information bytes
                match row_description_body[array_pos] {
                    0x00 => {
                        let field_name =  std::str::from_utf8(&row_description_body[value_start_pos..array_pos]).unwrap();
                        field_names.push(field_name.to_string());
                        array_pos += 19;
                        value_start_pos = array_pos;
                        break;
                    },
                    _ => {
                        array_pos += 1;
                    },
                }
            }
        }
        
        //create vector holding the individual rows
        let mut rows: Vec<HashMap<String, String>> = Vec::new();
        //read next message, its either command complete ,a datarow or empty query
        Self::read_rows(reader, &field_names, &mut rows);

        //after successfull query, read the ready for new query command
        Self::read_ready_command(reader);


    }

    fn read_rows (reader: &mut BufReader<TcpStream>, field_names: &Vec<String>, rows: &mut Vec<HashMap<String, String>>) {
        loop {
            //67 'C' is command complete
            //68 'D' is datarow
            //73 'I' is empty query

            //create vector to hold the head information of the response message
            //1 byte identifyer, 4 bytes message length
            let mut response_head: Vec<u8> = vec![0; 5];
            Self::read_from_db_stream(reader, &mut response_head);
            let response_length: i32 = i32::from_be_bytes(response_head[1..].try_into().unwrap());

            //check if the response matches expected messages
            assert!(matches!(response_head[0], 67 | 68 | 73));


            let packet_identifier = char::from_u32(response_head[0] as u32).unwrap();
            match  packet_identifier {
                //datarow
                'D' => {
                    let mut number_of_columns: Vec<u8> = vec![0; 2];
                    Self::read_from_db_stream(reader, &mut number_of_columns);
                    let number_of_columns: i16 = i16::from_be_bytes(number_of_columns[0..].try_into().unwrap());
                    //check if the number of columns matches the number in die field desc. message
                    assert_eq!(number_of_columns as usize, field_names.len(), "number of columns does not match");

                    //create hash map, holding the values
                    let mut values = HashMap::new();

                    //loop through the columns
                    for i in 0..number_of_columns {
                        //get the length of the value (4 bytes)
                        let mut value_length: Vec<u8> = vec![0; 4];
                        Self::read_from_db_stream(reader, &mut value_length);
                        let value_length: i32 = i32::from_be_bytes(value_length[0..].try_into().unwrap());

                        //get the value
                        let mut value: Vec<u8> = vec![0; value_length as usize];
                        Self::read_from_db_stream(reader, &mut value);

                        //turn the individual byres into a string
                        let value_string =  std::str::from_utf8(&value[0..]).unwrap();

                        //insert the value with the desciption into the hash map
                        values.insert(field_names[i as usize].to_string(), value_string.to_string());

                    }

                    //add the hash map to the rows vector
                    rows.push(values);
                },
                'C' => {

                    //get the value
                    let mut command_tag: Vec<u8> = vec![0; response_length as usize - 4];
                    Self::read_from_db_stream(reader, &mut command_tag);
                    let _command_tag_string = std::str::from_utf8(&command_tag[..]).unwrap();
                    break;
                },
                'I' => {
                    println!("query empty");
                    break;
                },
                _ => {
                    break;   
                },

            }
        }
    }

    fn read_ready_command (reader: &mut BufReader<TcpStream>) {
        //check query result and if db is ready for another query
        //create vector to hold the head information of the response message
        //1 byte identifyer, 4 bytes message length
        let mut ready_command: Vec<u8> = vec![0; 6];
        Self::read_from_db_stream(reader, &mut ready_command);

        //check if the response matches expected messages
        //90 = 'Z' ReadyForQuery
        assert_eq!(ready_command[0], 90);
        //check if db connection is in status idle
        //73 = 'I'
        assert_eq!(ready_command[5], 73);
    }

    fn read_error (reader: &mut BufReader<TcpStream>, error_length: i32) {
        println!("error");
        //read the error message
        let mut error_message: Vec<u8> = vec![0; error_length as usize - 4];
        Self::read_from_db_stream(reader, &mut error_message);
        

    }
}
