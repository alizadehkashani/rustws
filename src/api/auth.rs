use std::sync::Arc;
use std::collections::BTreeMap;

use crate::HTTPRequest;
use crate::DatabaseConnectionPool;
use crate::DatabaseValue;
use crate::APIValue;
use crate::JsonType;
use crate::api_send_response_json;
use crate::parse_json_string;

pub fn api_auth_auth_user (request: HTTPRequest, database_connections: Arc<DatabaseConnectionPool>) {

    //parse the post data from the client
    let post_data = parse_json_string(&request.body);

    //get the token from the client data
    let user_token = match post_data.get("UserToken").unwrap() {
        JsonType::String(token) => token.to_string(),
        _ => String::from(""),
    };


    //create variables for the response
    let mut api_response_vector: Vec<BTreeMap<String, Option<APIValue>>> = Vec::new();
    let mut api_response_btreemap: BTreeMap<String, Option<APIValue>> = BTreeMap::new();

    //see if a user with this token is in the database
    let query = format!(
        "SELECT id, username, token FROM users WHERE token = '{}'", 
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
            Some(APIValue::String(String::from("User could not be authenticated")))
        );

        api_response_vector.push(api_response_btreemap); 
        api_send_response_json(request, api_response_vector);

        return;

    } else {
        if let Some(DatabaseValue::Varchar(db_token)) = data[0].get("token").unwrap(){
            println!("db token: {}", db_token);
            if *db_token == user_token {
                api_response_btreemap.insert(
                    String::from("AuthenticationSuccessfull"), 
                    Some(APIValue::Boolean(true))
                );
                api_response_btreemap.insert(
                    String::from("Message"), 
                    Some(APIValue::String(String::from("User successfully authenticated")))
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
                    Some(APIValue::String(String::from("User could not be authenticated")))
                );

                api_response_vector.push(api_response_btreemap); 
                api_send_response_json(request, api_response_vector);

                return;
            }
        }
    }
}
