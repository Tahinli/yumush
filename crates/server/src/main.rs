use server::showtime;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    showtime().await;
}
