// migrations/ değişince sqlx::migrate! makrosunun yeniden derlenmesi için (SQLx belgesi).
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
