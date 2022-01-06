my graphics playground in Rust, using:

* https://github.com/gfx-rs/naga
* https://github.com/gfx-rs/wgpu
* https://github.com/EmbarkStudios/rust-gpu
* https://github.com/bevyengine/bevy


## crates

* `bosky`: svo ray tracing (run with `cargo run -p bosky --release`)



## problems with rust-gpu

* you cannnot do `vec[0]`, use `vec.x` instead
* https://github.com/EmbarkStudios/rust-gpu/issues/836
* no specialization constant
