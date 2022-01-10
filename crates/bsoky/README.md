
the u16 version can handle max block of 2^15=32768 blocks, it cannot handle 2^9 or 4^5 scene...

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


## more

### the rsvo scene

4^4, u16: 
total dim 256
block count 17530
memory used 2243968
memory ratio 0.13375092
avg 0.022031s

2^8, u16: -- result wrong???
total dim 256
block count 34129
memory used 546080
memory ratio 0.032548904
avg 0.018861s

4^4, u32:
total dim 256
block count 17530
memory used 4487936
memory ratio 0.26750183
avg 0.020623s

2^8, u32:
total dim 256
block count 96648
memory used 3092768
memory ratio 0.18434334
avg 0.023820s

4^5, u32:
total dim 1024
block count 338043
memory used 86539264
memory ratio 0.08059597
avg 0.033006s

2^10, u32:
total dim 1024
block count 1720910
memory used 55069152
memory ratio 0.051287144
avg 0.045746s