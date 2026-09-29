fn main() {
    // A missing `.env` is normal during bootstrap; dotenvy leaves the process
    // environment unchanged in that case.
    dotenvy::dotenv().ok();

    println!("Hello from gdk_hello");
}
