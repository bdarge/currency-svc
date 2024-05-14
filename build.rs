fn main() {
    tonic_build::compile_protos("pb/currency.proto")
    .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
}
