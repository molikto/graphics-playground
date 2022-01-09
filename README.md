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

## TODO

* data structure
    * ray traversal bugs
        * [ ] infinie looping (white)
        * [ ] there are red error codes with SDF scene
        * [ ] 4^4 result in wrong stuff drawn
    * performance
        * [ ] compare with ESVO performance
        * [ ] beam optimization
        * [ ] redirect rays
    * API
        * [ ] load `.vox` file
* material
    * [ ] volume fog material in "Next Week"
    * [ ] `.vox` complete
    * [ ] more physics based materials?
* renderer
    * [ ] progressive rendering (Bevy `RenderGraph`)
    * [ ] realtime rendering with TAA or DLSS or???
    * [ ] realtime rendering hardware API
* read more code & paper
    * https://github.com/AdamYuan/SparseVoxelOctree
    * https://github.com/oskarbraten/voxel-path-tracer
    * https://github.com/stijnherfst/BrickMap
    * https://github.com/rebelroad-reinhart/brickmap-vulkan
