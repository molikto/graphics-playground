my graphics playground in Rust, using:

* https://github.com/gfx-rs/naga
* https://github.com/gfx-rs/wgpu
* https://github.com/EmbarkStudios/rust-gpu
* https://github.com/bevyengine/bevy

run with `cargo run -p toy-xxx --release`

## done and TODOs

* data structures (in `common`)
    * sparse voxel N-tree (`svt.rs`) and ray casting: no children masks, each voxel is 4 byte pointer or data
        * performance
            * [X] compare with ESVO performance
            * [ ] optimize it to be faster
                * [X] rust-gpu performance problem
                * [ ] why the compute shader version is even slower?
            * [ ] beam optimization
            * [ ] redirect rays
        * API
            * [ ] load `.vox` file
* shadering (in `common`)
    * [X] textures
    * [X] basic material in "One Weekend"
    * [ ] glass material seems wrong??
    * [ ] shadow rays
    * [ ] volume fog material in "Next Week"
    * [ ] `.vox` complete
    * [ ] more physics based materials?
    * [ ] TAA???
    * [ ] DLSS???
* engine
    * [X] shader hot reloading of rust-gpu shader in bevy
    * [ ] compute shader hot reloading (using include_raw is very bad)
    * [ ] creating RenderNode in bevy is really not a good exp, lot of code
    * [ ] group everything better with switchable demo, adjustable specialized constants

## problems with rust-gpu

* you cannnot do `vec[0]`, use `vec.x` instead
* no specialization constant
* you need to be sure not to write performance bad code
    * array initialization
    * array bound check
* perf is ~1.25 times of GLSL, it might produce branchy code seems?
* code generation: it expand vec ops not as SpirV vec ops, but this doesn't hurt performance most of time

## referneces

* data structure
    * GigaVoxels: http://gigavoxels.inrialpes.fr/
    * ESVO: https://research.nvidia.com/sites/default/files/pubs/2010-02_Efficient-Sparse-Voxel/laine2010i3d_paper.pdf
        * https://github.com/AdamYuan/SparseVoxelOctree
        * https://github.com/Ria8651/voxel-tracing-playground
        * https://github.com/Neo-Zhixing/dust
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
    * https://github.com/DavidWilliams81/cubiquity
    * https://github.com/mgerhardy/engine
* ray tracing
    * Ray Tracing in One Weekend: https://raytracing.github.io/
        * https://github.com/mitchmindtree/nannou-rustgpu-raytracer
    * PBRT: https://www.pbrt.org/
    * GPU Wavefront Path Tracing: https://research.nvidia.com/sites/default/files/pubs/2013-07_Megakernels-Considered-Harmful/laine2013hpg_paper.pdf
        * https://jacco.ompf2.com/2019/07/18/wavefront-path-tracing/
    * TAA: http://behindthepixels.io/assets/files/TemporalAA.pdf 
        * https://github.com/oskarbraten/voxel-path-tracer
    * *more random ones bellow*
    * https://github.com/gkjohnson/three-gpu-pathtracing
    * https://github.com/chunky-dev/chunky/
    * https://github.com/erichlof/THREE.js-PathTracing-Renderer
    * https://github.com/EmbarkStudios/kajiya
* webgpu & rust-gpu
    * the compute shader are adapted from https://github.com/googlefonts/compute-shader-101


