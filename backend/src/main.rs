mod config;
mod error;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Feedback System Backend starting...");
}

#[cfg(test)]
mod tests {
    #[test]
    fn project_compiles() {
        assert!(true);
    }
}
