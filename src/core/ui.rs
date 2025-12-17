pub fn section(title: &str){
    println!();
    println!("{}", title);
    println!("{}", "-".repeat(title.len()));
}

pub fn kv(key: &str, value: &str){
    println!("{:<8} {}", format!("{}:", key), value);
}