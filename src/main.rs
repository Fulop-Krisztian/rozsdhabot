mod fetcher;
mod hardverapro_parser;
mod models;
mod monitor;
mod notifier;

// TODO: kód összetartó kókusz
// TODO: a kókuszt ÉJÁJJÁL ellenőrzi
// https://www.youtube.com/watch?v=SmM0653YvXU

#[tokio::main]
async fn main() -> Result<(), ()> {
    // In rust, files are closed automatically when they go out of scope.

    match std::fs::File::open("coconut.jpg") {
        Ok(_) => (),
        Err(_) => panic!("Nincs kokusz, nincs program."),
    }

    // Load the environment variables
    dotenv::from_filename(".env").expect("Couldn't find .env file");
    // Get the token. TODO: We might be able to do this later.
    dotenv::var("TOKEN").expect("TOKEN not found in .env file");

    match dotenv::var("INTEGRATION")
        .expect("INTEGRATION not found in .env file")
        .as_str()
    {
        "telegram" => println!("Telegram integration"),
        "discord" => println!("Discord integration"),
        // This just prints what would be sent as messages to telegram as text to the
        // terminal. Mostly meant for debug purposes
        "terminal" => println!("Terminal integration"),
        _ => println!("Invalid integration"),
    }

    // let body = include_str!("../index.html");
    //
    // let results = hardverapro_parser::parse_hardverapro(body);
    // let length = results.len();
    // for result in results {
    //     println!("{result:?}");
    // }
    // println!("Found {} listings", length);

    let monitor = monitor::Monitor::new(vec![Box::new(notifier::TerminalNotifier::new())]);
    println!("Hello, coconut!");

    return monitor.watch_url("https://hardverapro.hu", 2).await;
}
