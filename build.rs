fn main() {
    if std::is_x86_feature_detected!("avx2") {
        println!("cargo:rustc-cfg=support_avx2");
    }
    
    if std::is_x86_feature_detected!("rdrand") {
        println!("cargo:rustc-cfg=support_rdrand");
    }
    
    println!("cargo:rustc-cfg=prime_with_thread");
}