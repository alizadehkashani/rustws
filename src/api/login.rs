use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::APIValue;
use crate::JsonType;
use crate::DatabaseConnectionPool;
use crate::DatabaseValue;
use crate::HTTPRequest;
use crate::generate_token;
use crate::api_send_response_json;
use crate::parse_json_string;

//function for auto login of user
pub fn api_login_auto_logon (
    request: HTTPRequest, 
    database_connections: Arc<DatabaseConnectionPool>,
) {
    let post_data = parse_json_string(&request.body);

    let user_token = match post_data.get("UserToken").unwrap() {
        JsonType::String(token) => token.to_string(),
        _ => String::from(""),
    };


    //create variables for the response
    let mut api_response_vector: Vec<BTreeMap<String, Option<APIValue>>> = Vec::new();
    let mut api_response_btreemap: BTreeMap<String, Option<APIValue>> = BTreeMap::new();

    //see if a user with this token is in the database
    let query = format!(
        "SELECT token, rememberlogin FROM users WHERE token = '{}'", 
        user_token
    );

    let mut db_con = DatabaseConnectionPool::get_connection(&database_connections).unwrap();
    let data = db_con.query(&query);
    database_connections.release_connection(db_con);

    //check if no user has been found, if yes, return
    if data.len() == 0 {
        api_response_btreemap.insert(
            String::from("AuthenticationSuccessfull"), 
            Some(APIValue::Boolean(false))
        );
        api_response_btreemap.insert(
            String::from("Message"), 
            Some(APIValue::String(String::from("No user found with matching token")))
        );

        api_response_vector.push(api_response_btreemap); 
        api_send_response_json(request, api_response_vector);
        
        return;
    }

    //check if the user wants to logged in automatically
    let user_remember_login = match data[0].get("rememberlogin").unwrap() {
        Some(DatabaseValue::Boolean(true)) => true,
        Some(DatabaseValue::Boolean(false)) => false,
        _ => false,
    };

    if user_remember_login {
        api_response_btreemap.insert(
            String::from("AuthenticationSuccessfull"), 
            Some(APIValue::Boolean(true))
        );
        api_response_btreemap.insert(
            String::from("Message"), 
            Some(APIValue::String(String::from("Matching user with token has been found")))
        );

        api_response_vector.push(api_response_btreemap); 
        api_send_response_json(request, api_response_vector);
        
        return;

    } else {
        api_response_btreemap.insert(
            String::from("AuthenticationSuccessfull"), 
            Some(APIValue::Boolean(false))
        );
        api_response_btreemap.insert(
            String::from("Message"), 
            Some(APIValue::String(String::from("Matching user with token has been found, but remember login is not active")))
        );

        api_response_vector.push(api_response_btreemap); 
        api_send_response_json(request, api_response_vector);
        
        return;
    }
}

pub fn api_login_logon (    
    request: HTTPRequest, 
    database_connections: Arc<DatabaseConnectionPool>,
) {

    //parse the data from the fetch request
    let post_data = parse_json_string(&request.body);

    //initalizse the paramters
    let mut user = String::from("");
    let mut user_pw = String::from("");
    let mut remember: bool = false;

    //check the data from the http request, if data is available
    if let JsonType::String(json_user) = post_data.get("user").unwrap() {
        user = json_user.to_string()
    }; 

    if let JsonType::String(json_pw) = post_data.get("password").unwrap() {
        user_pw = json_pw.to_string()
    };

    if let JsonType::Boolean(json_remember) = post_data.get("remember").unwrap() {
        remember = *json_remember
    }; 

    let mut api_response_vector: Vec<BTreeMap<String, Option<APIValue>>> = Vec::new();
    let mut api_response_btreemap: BTreeMap<String, Option<APIValue>> = BTreeMap::new();

    let query = format!(
        "SELECT id, password FROM users WHERE username = '{}'", 
        user
    );

    let mut db_con = DatabaseConnectionPool::get_connection(&database_connections).unwrap();
    let data = db_con.query(&query);
    database_connections.release_connection(db_con);

    //check if a user has been found
    //if not, return
    if data.len() == 0 {

        api_response_btreemap.insert(
            String::from("LoginSuccessfull"), 
            Some(APIValue::Boolean(false))
        );
        api_response_btreemap.insert(
            String::from("Message"), 
            Some(APIValue::String(String::from("User not found")))
        );
        api_response_vector.push(api_response_btreemap); 

        api_send_response_json(request, api_response_vector);
        
        return;
    }

    //get password from database response
    let db_pw = match data[0].get("password").unwrap() {
        Some(DatabaseValue::Varchar(pw)) => pw.clone(),
        _ => String::from(""),
    };

    //check if the password matches
    if db_pw == user_pw.as_str() {//TODO when pw matches

        api_response_btreemap.insert(
            String::from("LoginSuccessfull"), 
            Some(APIValue::Boolean(true))
        );

        //generate new token
        let user_token = generate_token();

        //insert token into btreemap
        api_response_btreemap.insert(
            String::from("UserToken"), 
            Some(APIValue::String(user_token.to_string()))
        );

        let token_creation_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let query = format!(
            "UPDATE users SET token = '{}', tokencrtime = {}, rememberlogin = {} WHERE username = '{}'", 
            user_token,
            token_creation_timestamp,
            remember,
            user,
        );

        let mut db_con = DatabaseConnectionPool::get_connection(&database_connections).unwrap();
        db_con.query(&query);
        database_connections.release_connection(db_con);

    } else {//TODO what to do, when password does not match
        api_response_btreemap.insert(
            String::from("LoginSuccessfull"), 
            Some(APIValue::Boolean(false))
        );
        api_response_btreemap.insert(
            String::from("Message"), 
            Some(APIValue::String(String::from("Incorrect password")))
        );
    }

    api_response_vector.push(api_response_btreemap); 

    crate::api_send_response_json(request, api_response_vector);

}

