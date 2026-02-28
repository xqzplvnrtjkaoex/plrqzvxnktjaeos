fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fds = protox::compile(
        [
            "proto/user.proto",
            "proto/library.proto",
            "proto/notification.proto",
        ],
        ["proto/"],
    )?;

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_fds(fds)?;

    Ok(())
}
