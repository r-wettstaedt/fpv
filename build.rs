extern crate cpp_build;

fn main() {
    cpp_build::Config::new()
        .include("/usr/local/Cellar/opencv3/3.2.0/include/opencv")
        .include("/usr/local/Cellar/opencv3/3.2.0/include")
        .build("src/main.rs");
}