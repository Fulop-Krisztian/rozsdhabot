use crate::{fetcher::Fetcher, hardverapro_parser::parse_hardverapro, notifier::Notifier};

pub struct Monitor {
    fetcher: Fetcher,

    // store: Arc<SledStore>,
    notifiers: Vec<Box<dyn Notifier>>,
    // If I undestand correctly, this should be a vector of pointers that point to objects that implement the Notifier trait.
}

impl Monitor {
    pub fn new(notifiers: Vec<Box<dyn Notifier>>) -> Self {
        Self {
            fetcher: Fetcher::new(),
            notifiers,
        }
    }

    pub async fn watch_url(&self, url: &str, interval: u64) -> Result<(), ()> {
        let mut fetchtries = 0;
        loop {
            let body = match self.fetcher.fetch(url).await {
                Ok(body) => {
                    fetchtries = 0;
                    body
                }
                Err(e) => {
                    fetchtries += 1;
                    if fetchtries == 5 {
                        eprintln!("can't fetch site. giving up after {} trires", fetchtries);
                        return Err(());
                    }
                    eprintln!("{e}. retrying in {} seconds...", 3 * fetchtries);
                    // Perhaps exiting after a few errors would be better.
                    // Maybe send a notification to the telegram channel?

                    std::thread::sleep(std::time::Duration::from_secs(3 * fetchtries));
                    continue;
                }
            };

            let listings = parse_hardverapro(body.as_str());

            // TODO: some kind of test to see if a listing is new

            for notifier in &self.notifiers {
                // TODO: this is a test with the first listing that we encounter.
                listings.first().map(|x| notifier.notify_new_listing(x));
            }
            std::thread::sleep(std::time::Duration::from_secs(interval));
        }
    }
}
