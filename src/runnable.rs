pub trait Runnable {
    async fn run(&mut self) -> Result<(), reqwest::Error>;
}
