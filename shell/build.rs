fn main() {
    println!("cargo:rustc-link-arg=-Tshell.ld");
    println!("cargo:rerun-if-changed=shell.ld");
}
