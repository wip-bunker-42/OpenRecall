fn main() -> openrecall_core::AppResult<()> {
    let destination = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "src/shared/generated".into());
    openrecall_core::contracts::export(std::path::Path::new(&destination))
}
