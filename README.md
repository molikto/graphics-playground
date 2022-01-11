my graphics playground in Rust, using:

* https://github.com/gfx-rs/naga
* https://github.com/gfx-rs/wgpu
* https://github.com/EmbarkStudios/rust-gpu
* https://github.com/bevyengine/bevy

run with `cargo run -p bosky --release`

## done and TODOs

* data structures (in `common-no-std`)
    * sparse voxel N-tree (`svt.rs`) and ray casting
        * ray traversal bugs
            * [ ] infinie looping (white)
            * [ ] there are red error codes with SDF scene
            * [ ] 4^4 result in wrong stuff drawn
        * performance
            * [ ] compare with ESVO performance -- 5~6 times slower
            * [ ] base layer
            * [ ] beam optimization
            * [ ] redirect rays
        * API
            * [ ] load `.vox` file
* material (in `common-no-std`)
    * [X] basic material in "One Weekend"
    * [ ] volume fog material in "Next Week"
    * [ ] `.vox` complete
    * [ ] more physics based materials?
* renderer (in `bosky`)
    * [ ] **progressive rendering (Bevy `RenderGraph`)**
    * [ ] realtime rendering with TAA or DLSS or???
    * [ ] realtime rendering hardware API

## problems with rust-gpu

* you cannnot do `vec[0]`, use `vec.x` instead
* https://github.com/EmbarkStudios/rust-gpu/issues/836
* no specialization constant


## referneces

* data structure
    * GigaVoxels: http://gigavoxels.inrialpes.fr/
    * ESVO: https://research.nvidia.com/sites/default/files/pubs/2010-02_Efficient-Sparse-Voxel/laine2010i3d_paper.pdf
        * https://github.com/AdamYuan/SparseVoxelOctree
        * https://github.com/Ria8651/voxel-tracing-playground
    * GVDB: https://developer.nvidia.com/gvdb
    * NanoVDB: https://developer.nvidia.com/nanovdb
    * BrickMap: https://dspace.library.uu.nl/handle/1874/315917
        * https://github.com/stijnherfst/BrickMap
        * https://github.com/rebelroad-reinhart/brickmap-vulkan
    * SSVDAG: https://jcgt.org/published/0006/02/01/
        * https://github.com/RvanderLaan/DAGger
    * *more random ones bellow*
    * https://jacco.ompf2.com/2021/02/01/a-voxel-renderer-for-learning-c-c/
    * https://www.reddit.com/r/VoxelGameDev/comments/cmsu9a/raytraced_voxel_engine/
* ray tracing
    * Ray Tracing in One Weekend: https://raytracing.github.io/
    * PBRT: https://www.pbrt.org/
    * GPU Wavefront Path Tracing: https://research.nvidia.com/sites/default/files/pubs/2013-07_Megakernels-Considered-Harmful/laine2013hpg_paper.pdf
        * https://jacco.ompf2.com/2019/07/18/wavefront-path-tracing/
    * TAA: http://behindthepixels.io/assets/files/TemporalAA.pdf 
        * https://github.com/oskarbraten/voxel-path-tracer
    * *more random ones bellow*
    * https://github.com/chunky-dev/chunky/
    * https://github.com/erichlof/THREE.js-PathTracing-Renderer
    * https://github.com/EmbarkStudios/kajiya

