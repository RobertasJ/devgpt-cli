#[macro_export]
macro_rules! print_chat {
    ($l:ident) => {
        while let Some(content) = $l.receive_content(0).await.unwrap() {
            print!("{content}");
            stdout().flush().unwrap();
        }

        println!();
    };
}
