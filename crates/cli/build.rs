use tonic_prost_build::configure;

fn main() {
    configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(
            &[
                "../server/proto/pyrion/v1/discovery.proto",
                "../server/proto/pyrion/v1/session.proto",
            ],
            &["../server/proto"],
        )
        .unwrap();
}
