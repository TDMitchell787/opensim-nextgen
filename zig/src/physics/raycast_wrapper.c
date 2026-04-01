typedef struct { float x, y, z; } BSVec3;
typedef struct { unsigned int id; float fraction; BSVec3 normal; BSVec3 point; } BSRayHit;

extern BSRayHit RayTest2(void* world, BSVec3 from, BSVec3 to, unsigned int fg, unsigned int fm);

void RayTest2_safe(void* world,
                   float fx, float fy, float fz,
                   float tx, float ty, float tz,
                   unsigned int fg, unsigned int fm,
                   unsigned int* out_id, float* out_fraction,
                   float* out_nx, float* out_ny, float* out_nz,
                   float* out_px, float* out_py, float* out_pz) {
    BSVec3 from = {fx, fy, fz};
    BSVec3 to = {tx, ty, tz};
    BSRayHit hit = RayTest2(world, from, to, fg, fm);
    *out_id = hit.id;
    *out_fraction = hit.fraction;
    *out_nx = hit.normal.x;
    *out_ny = hit.normal.y;
    *out_nz = hit.normal.z;
    *out_px = hit.point.x;
    *out_py = hit.point.y;
    *out_pz = hit.point.z;
}
