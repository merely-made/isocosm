fn main() {
    let mut endpoint = isocosm_graphshell::IsocosmEndpoint::fixture();
    graphshell_endpoint::stdio::serve_basic(
        &mut endpoint,
        std::io::stdin().lock(),
        std::io::stdout().lock(),
    )
    .expect("Isocosm Graphshell endpoint failed");
}
