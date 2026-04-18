#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("dm-api v{} (scaffold)", env!("CARGO_PKG_VERSION"));
    Ok(())
}
