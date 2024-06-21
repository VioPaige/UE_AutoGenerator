extern crate regex;
use self::regex::Regex;

pub fn colouriseLog(text: String, clearLine: bool) {
    let mut result = text.to_string();
    let codes = [
        ("red", "\x1b[31m"),
        ("green", "\x1b[32m"),
        ("yellow", "\x1b[33m"),
        ("grey", "\x1b[90m"),
    ];

    for (ref name, ref code) in codes.iter() {
        let target = format!(r"<<{}>>(.*?)</{}>>", name, name);
        let replacement = format!("{}$1\x1b[0m", code);
        let re = Regex::new(&target).unwrap();

        result = re.replace_all(&result, replacement).to_string();
    }

    if clearLine {
       print!("\r{}", result); 
    } else {
        println!("{}", result);
    }
}