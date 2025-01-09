fn main() {
    cc::Build::new()
        .file("src/fourier_transform.c")
        .compile("fourier_transform");
    println!("cargo::rerun-if-changed=src/fourier_transform.c")
}
