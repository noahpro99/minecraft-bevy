# Fixes Applied

## Movement & Physics
- ✅ Physics tick rate: 20 Hz → 60 Hz
- ✅ Physics substeps: 1 → 8 (effective 480 Hz collision detection)
- ✅ Player collider: cuboid(0.3, 0.9, 0.3)
- ✅ Velocity threshold: 0.001 → 0.01 (reduces micro-stops)

## Chunk Colliders
- ✅ TriMesh → Voxel grid collider (no edge catching!)
- ✅ Tall grass: Sensor collider (walk-throughable)
- ✅ Collision logic: excludes grass from solid collider

## Performance
- ✅ Chunk loading: 1 chunk/frame (reduces new chunk lag)
- ✅ Mesh updates: 8 chunks/frame (keeps block breaking responsive)
