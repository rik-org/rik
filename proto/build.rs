fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("./src/controller.proto")?;
    tonic_build::compile_protos("google/protobuf/empty.proto")?;
    tonic_build::compile_protos("./src/worker.proto")?;
    Ok(())
}
