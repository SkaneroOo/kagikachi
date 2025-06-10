use mini_json::Value;

use std::collections::HashMap;

use sockets::{errors::SocketError, frame::DataFrame, response::Response, SocketServer, frame::Opcode};


fn set_cmd(args: &str, storage: &mut HashMap<String, Value>) -> Response {
    let (key, value) = match args.split_once(" ") {
        Some((key, value)) => (key, value),
        None => return Response::builder().set_body("Invalid arguments")
    };
    let value = match Value::deserialize(value) {
        Ok(v) => v,
        Err(e) => return Response::builder().set_body(format!("Invalid value: {e:?}"))
    };
    let (key, path) = match key.split_once(".") {
        Some((key, path)) => (key, Some(path)),
        None => (key, None)
    };
    match path {
        None => {
            storage.insert(key.to_string(), value);
            Response::builder().set_body("OK")
        },
        Some(path) => {
            match storage.get_mut(key) {
                Some(val) => match val.get_mut_element(path) {
                    Ok(val) => {
                        *val = value;
                        Response::builder().set_body("OK")
                    },
                    Err(e) => Response::builder().set_body(e)
                },
                None => Response::builder().set_body("Key not found")
            }
        }
    }
}

fn get_cmd(args: &str, storage: &mut HashMap<String, Value>) -> Response {
    let (key, path) = match args.split_once(".") {
        Some((key, path)) => (key, Some(path)),
        None => (args, None)
    };
    let value = match storage.get(key) {
        Some(value) => match path {
            None => value,
            Some(path) => match value.get_element(path) {
                Ok(value) => value,
                Err(e) => return Response::builder().set_body(e)
            }
        },
        None => return Response::builder().set_body("Key not found")
    };
    Response::builder().set_body(Into::<String>::into(value))
}

fn del_cmd(args: &str, storage: &mut HashMap<String, Value>) -> Response {
    let (key, path) = match args.split_once(".") {
        Some((key, path)) => (key, Some(path)),
        None => (args, None)
    };
    match path {
        None => {
            storage.remove(key);
            Response::builder().set_body("OK")
        }
        Some(path) => {
            match storage.get_mut(key) {
                Some(val) => {
                    let (parent, key) = match path.rsplit_once(".") {
                        Some((parent, key)) => (parent, key),
                        None => {
                            match val {
                                Value::Object(map) => {
                                    match map.remove(key) {
                                        Some(_) => return Response::builder().set_body("OK"),
                                        None => return Response::builder().set_body("Key not found")
                                    }
                                },
                                Value::Array(arr) => {
                                    let index = match path.parse::<usize>() {
                                        Ok(i) => i,
                                        Err(_) => return Response::builder().set_body("Invalid index")
                                    };
                                    if index >= arr.len() {
                                        return Response::builder().set_body("Index out of range")
                                    }
                                    arr.remove(index);
                                    return Response::builder().set_body("OK")
                                },
                                _ => return Response::builder().set_body("Invalid type")
                            }
                        }
                    };
                    match val.get_mut_element(parent) {
                        Ok(val) => match val {
                            Value::Object(map) => {
                                match map.remove(key) {
                                    Some(_) => Response::builder().set_body("OK"),
                                    None => Response::builder().set_body("Key not found")
                                }
                            },
                            Value::Array(arr) => {
                                let index = match key.parse::<usize>() {
                                    Ok(i) => i,
                                    Err(_) => return Response::builder().set_body("Invalid index")
                                };
                                arr.remove(index);
                                Response::builder().set_body("OK")
                            },
                            _ => Response::builder().set_body("Invalid type")
                        },
                        Err(e) => Response::builder().set_body(e)
                    }
                },
                None => {
                    Response::builder().set_body("Key not found")
                }
            }
        }
    }
}

fn dump_cmd(storage: &HashMap<String, Value>) -> Response {
    let value = Value::Object(storage.clone());
    Response::builder().set_body(Into::<String>::into(value))
}

fn load_cmd(args: &str, storage: &mut HashMap<String, Value>) -> Response {
    let value = match Value::deserialize(args) {
        Ok(v) => v,
        Err(e) => return Response::builder().set_body(format!("Invalid value: {e:?}"))
    };
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                storage.insert(key, value);
            }
            Response::builder().set_body("OK")
        },
        _ => Response::builder().set_body("Invalid type")
    }
}

fn ping_cmd() -> Response {
    Response::builder().set_body("PONG")
}

fn message_handler(msg: DataFrame, storage: &mut HashMap<String, Value>) -> Response {
    match msg.opcode {
        Opcode::Text => (),
        _ => return Response::builder().set_body("Invalid message type")
    }
    let message = msg.payload.string().expect("Assertion failed, check if payload was properly decoded");
    let (command, args) = match message.split_once(" ") {
        Some((command, args)) => (command, args),
        None => (message.as_str(), "")
    };

    match command.to_ascii_lowercase().as_str() {
        "set" => set_cmd(args, storage),
        "get" => get_cmd(args, storage),
        "del" => del_cmd(args, storage),
        "dump" => dump_cmd(storage),
        "load" => load_cmd(args, storage),
        "ping" => ping_cmd()
        _ => Response::builder().set_body("Unknown command")
    }
}

fn error_handler(e: SocketError) {
    println!("Error: {e}")
}


fn main() {
    let cache: HashMap<String, Value> = HashMap::new();
    let server = SocketServer::new(message_handler, error_handler, cache);
    server.run();
    // json_benchmark();
}

#[allow(unused)]
fn json_benchmark() {
    use std::time::Instant;
    let json = r#"{"val": 1,
    "web-app": {
  "servlet": [   
    {
      "servlet-name": "cofaxCDS",
      "servlet-class": "org.cofax.cds.CDSServlet",
      "init-param": {
        "configGlossary:installationAt": "Philadelphia, PA",
        "configGlossary:adminEmail": "ksm@pobox.com",
        "configGlossary:poweredBy": "Cofax",
        "configGlossary:poweredByIcon": "/images/cofax.gif",
        "configGlossary:staticPath": "/content/static",
        "templateProcessorClass": "org.cofax.WysiwygTemplate",
        "templateLoaderClass": "org.cofax.FilesTemplateLoader",
        "templatePath": "templates",
        "templateOverridePath": "",
        "defaultListTemplate": "listTemplate.htm",
        "defaultFileTemplate": "articleTemplate.htm",
        "useJSP": false,
        "jspListTemplate": "listTemplate.jsp",
        "jspFileTemplate": "articleTemplate.jsp",
        "cachePackageTagsTrack": 200,
        "cachePackageTagsStore": 200,
        "cachePackageTagsRefresh": 60,
        "cacheTemplatesTrack": 100,
        "cacheTemplatesStore": 50,
        "cacheTemplatesRefresh": 15,
        "cachePagesTrack": 200,
        "cachePagesStore": 100,
        "cachePagesRefresh": 10,
        "cachePagesDirtyRead": 10,
        "searchEngineListTemplate": "forSearchEnginesList.htm",
        "searchEngineFileTemplate": "forSearchEngines.htm",
        "searchEngineRobotsDb": "WEB-INF/robots.db",
        "useDataStore": true,
        "dataStoreClass": "org.cofax.SqlDataStore",
        "redirectionClass": "org.cofax.SqlRedirection",
        "dataStoreName": "cofax",
        "dataStoreDriver": "com.microsoft.jdbc.sqlserver.SQLServerDriver",
        "dataStoreUrl": "jdbc:microsoft:sqlserver://LOCALHOST:1433;DatabaseName=goon",
        "dataStoreUser": "sa",
        "dataStorePassword": "dataStoreTestQuery",
        "dataStoreTestQuery": "SET NOCOUNT ON;select test='test';",
        "dataStoreLogFile": "/usr/local/tomcat/logs/datastore.log",
        "dataStoreInitConns": 10,
        "dataStoreMaxConns": 100,
        "dataStoreConnUsageLimit": 100,
        "dataStoreLogLevel": "debug",
        "maxUrlLength": 500}},
    {
      "servlet-name": "cofaxEmail",
      "servlet-class": "org.cofax.cds.EmailServlet",
      "init-param": {
      "mailHost": "mail1",
      "mailHostOverride": "mail2"}},
    {
      "servlet-name": "cofaxAdmin",
      "servlet-class": "org.cofax.cds.AdminServlet"},
 
    {
      "servlet-name": "fileServlet",
      "servlet-class": "org.cofax.cds.FileServlet"},
    {
      "servlet-name": "cofaxTools",
      "servlet-class": "org.cofax.cms.CofaxToolsServlet",
      "init-param": {
        "templatePath": "toolstemplates/",
        "log": 1,
        "logLocation": "/usr/local/tomcat/logs/CofaxTools.log",
        "logMaxSize": "",
        "dataLog": 1,
        "dataLogLocation": "/usr/local/tomcat/logs/dataLog.log",
        "dataLogMaxSize": "",
        "removePageCache": "/content/admin/remove?cache=pages&id=",
        "removeTemplateCache": "/content/admin/remove?cache=templates&id=",
        "fileTransferFolder": "/usr/local/tomcat/webapps/content/fileTransferFolder",
        "lookInContext": 1,
        "adminGroupID": 4,
        "betaServer": true}}],
  "servlet-mapping": {
    "cofaxCDS": "/",
    "cofaxEmail": "/cofaxutil/aemail/*",
    "cofaxAdmin": "/admin/*",
    "fileServlet": "/static/*",
    "cofaxTools": "/tools/*"},
 
  "taglib": {
    "taglib-uri": "cofax.tld",
    "taglib-location": "/WEB-INF/tlds/cofax.tld"}}}"#;

    let mut start = Instant::now();
    for _ in 0..100000 {
        // let _ = serde_json::from_str::<serde_json::Value>(json).unwrap();
    }
    println!("Serde: {}ms", start.elapsed().as_millis());

    start = Instant::now();
    for _ in 0..100000 {
        let _ = Value::deserialize(json).unwrap();
    }
    println!("My impl: {}ms", start.elapsed().as_millis());
}