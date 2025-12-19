use regex::Regex;

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn link_type(type_str: &str) -> String {
    let mut html = escape_html(type_str);
    
    // Primitives
    let primitives = [
        ("str", "string.html"), 
        ("num", "num.html"), 
        ("bool", "bool.html"), 
        ("void", "void.html"), 
        ("Array", "array.html")
    ];
    
    for (prim, link) in primitives {
            let pattern = format!(r"\b{}\b", regex::escape(prim));
            if let Ok(re) = Regex::new(&pattern) {
                let replacement = format!("<a href=\"{}\">{}</a>", link, prim);
                html = re.replace_all(&html, replacement.as_str()).to_string();
            }
    }
    html
}

fn main() {
    let input = "Array<str>";
    let output = link_type(input);
    println!("Input: {}", input);
    println!("Output: {}", output);

    let input2 = "str";
    let output2 = link_type(input2);
    println!("Input: {}", input2);
    println!("Output: {}", output2);
}
