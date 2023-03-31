fn main() {
    let received_text = std::env::var("text").unwrap();
    println!("Received text: {}", received_text);
}
