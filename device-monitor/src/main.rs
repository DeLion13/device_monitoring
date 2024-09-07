#[tokio::main]
async fn main() {
    env_logger::init();

    let routes = device_monitor::create_routes().await;

    // Run the server on localhost with opened 8080 port
    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
