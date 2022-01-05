references:
* https://research.nvidia.com/sites/default/files/pubs/2010-02_Efficient-Sparse-Voxel/laine2010i3d_paper.pdf
* https://developer.nvidia.com/gvdb
* https://github.com/Ria8651/voxel-tracing-playground



the u16 version can handle max block of 2^15=32768 blocks

tests:

* sdf scene with + u16 + 8^3 config: block count 5010, memory used 5131264, frame time 0.037606s
* sdf scene with + u16 + 2^9 config: pointer size cannot handle 

* sdf scene with + u32 + 8^3 config: block count 5010, memory used 10262528, frame time 0.040383s
* sdf scene with + u32 + 2^9 config: block count 72772, memory used 2328736, frame time 0.047637s

* sdf scene with + u16 + 2^8 config: block count 18076, memory used 289232, 0.038304s
* sdf scene with + u16 + 4^4 config: block count 4647, memory used 594944, frame time 0.024614s

my analysis:

it seems having bigger block (that is what I call it, a block is a 2^3 in case of octree tree, or N^3 in case of other sized blocks) will increase the memory, and also make ray tracing performance better

another thing is having bigger block makes block count way down, so we can handle bigger scene with much smaller pointer size, and you might be able to use 15 bit children pointer

and u16 children pointer compared to a u32 children pointer will always half the memory requirement 


## TODO

* fix the pixel bug --- on hold, cannot use RenderDoc
* there are red error codes
* 4^4 result in wrong stuff drawn
* fix the problem with dynamic-vec branch
* create my own render graph, don't use the builtin PBR pipeline