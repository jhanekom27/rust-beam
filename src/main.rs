#[tokio::main]
async fn main() {
    // println!("Hello, world!");
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        println!("Usage: p2p_file_share [send|receive] [FILE_PATH|ADDRESS:PORT]");
        return;
    }

    match args[1].as_str() {
        "send" if args.len() == 3 => {
            let filepath = &args[2];
            println!("Filepath: {filepath}");
            // start_sender(filepath).await;
        }
        "receive" if args.len() == 3 => {
            let address = &args[2];
            println!("Address: {address}");
            // start_receiver(address).await;
        }
        _ => println!("Invalid usage"),
    }
}

// #[tokio::main]
// async fn main() {
//     let args: Vec<String> = std::env::args().collect();
//     if args.len() < 2 {
//         println!("Usage: p2p_file_share [send|receive] [FILE_PATH|ADDRESS:PORT]");
//         return;
//     }

// match args[1].as_str() {
//     "send" if args.len() == 3 => {
//         let filepath = &args[2];
//         start_sender(filepath).await;
//     },
//     "receive" if args.len() == 3 => {
//         let address = &args[2];
//         start_receiver(address).await;
//     },
//     _ => println!("Invalid usage"),
// }
// }
