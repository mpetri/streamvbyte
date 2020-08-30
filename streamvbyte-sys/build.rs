use cmake;

fn main() {
    // compile library
    let dst = cmake::build("external/streamvbyte");
    println!("cargo:rustc-link-search=native={}/lib/", dst.display());
    println!("cargo:rustc-link-lib=static=streamvbyte_static");
}
