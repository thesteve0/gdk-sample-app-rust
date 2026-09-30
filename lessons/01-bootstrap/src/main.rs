fn main() {
    // This binary will become the application that orchestrates provider calls
    // and tools. Bootstrap only prepares its safe local configuration.
    // A missing `.env` is normal during bootstrap; dotenvy leaves the process
    // environment unchanged in that case.
    dotenvy::dotenv().ok();

    println!("Hello from gdk_hello");
}
