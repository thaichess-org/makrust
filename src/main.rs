use makrust::startup::Application;

#[tokio::main]
async fn main() {
    let _ = Application::run().await;
}
