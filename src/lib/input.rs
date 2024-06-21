use std::collections::HashMap;

pub fn parseArguments(args: Vec<String>) -> HashMap<String, String> {
    let mut parsedArgs: HashMap<String, String> = HashMap::new();

    for arg in args {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();

        if parts.len() == 2 {
            parsedArgs.insert(parts[0].to_string(), parts[1].to_string());
        } else {
            println!("Warning: argument '{}' is malformed.", arg)
        }
    }

    parsedArgs
}