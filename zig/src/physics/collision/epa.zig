const std = @import("std");
const v3 = @import("vec3.zig");
const gjk_mod = @import("gjk.zig");
const shapes_mod = @import("shapes.zig");

const Vec3 = v3.Vec3;
const CollisionBody = shapes_mod.CollisionBody;
const Contact = shapes_mod.Contact;
const Simplex = gjk_mod.Simplex;

const MAX_ITERATIONS = 64;
const MAX_FACES = 128;
const TOLERANCE = 1e-6;

const Face = struct {
    a: u32,
    b: u32,
    c: u32,
    normal: Vec3,
    distance: f32,
};

const Polytope = struct {
    vertices: [128]Vec3 = undefined,
    vertex_count: u32 = 0,
    faces: [MAX_FACES]Face = undefined,
    face_count: u32 = 0,

    fn addVertex(self: *Polytope, v: Vec3) u32 {
        if (self.vertex_count >= 128) return self.vertex_count - 1;
        self.vertices[self.vertex_count] = v;
        self.vertex_count += 1;
        return self.vertex_count - 1;
    }

    fn addFace(self: *Polytope, a: u32, b: u32, c: u32) void {
        if (self.face_count >= MAX_FACES) return;
        const va = self.vertices[a];
        const vb = self.vertices[b];
        const vc = self.vertices[c];
        const ab = v3.sub(vb, va);
        const ac = v3.sub(vc, va);
        var normal = v3.normalize(v3.cross(ab, ac));
        var dist = v3.dot(normal, va);
        if (dist < 0) {
            normal = v3.negate(normal);
            dist = -dist;
        }
        self.faces[self.face_count] = .{ .a = a, .b = b, .c = c, .normal = normal, .distance = dist };
        self.face_count += 1;
    }

    fn closestFace(self: *const Polytope) ?u32 {
        if (self.face_count == 0) return null;
        var best: u32 = 0;
        var best_dist: f32 = self.faces[0].distance;
        var i: u32 = 1;
        while (i < self.face_count) : (i += 1) {
            if (self.faces[i].distance < best_dist) {
                best_dist = self.faces[i].distance;
                best = i;
            }
        }
        return best;
    }

    fn removeFace(self: *Polytope, idx: u32) void {
        if (idx < self.face_count - 1) {
            self.faces[idx] = self.faces[self.face_count - 1];
        }
        self.face_count -= 1;
    }
};

const Edge = struct { a: u32, b: u32 };

fn minkowskiSupport(a: *const CollisionBody, b: *const CollisionBody, dir: Vec3) Vec3 {
    const sa = shapes_mod.supportPoint(a, dir);
    const sb = shapes_mod.supportPoint(b, v3.negate(dir));
    return v3.sub(sa, sb);
}

pub fn epa(a: *const CollisionBody, b: *const CollisionBody, simplex: *const Simplex) ?Contact {
    if (simplex.count < 4) {
        return fallbackContact(a, b);
    }

    var polytope = Polytope{};
    for (0..4) |i| {
        _ = polytope.addVertex(simplex.points[i]);
    }
    polytope.addFace(0, 1, 2);
    polytope.addFace(0, 2, 3);
    polytope.addFace(0, 3, 1);
    polytope.addFace(1, 3, 2);

    var iter: u32 = 0;
    while (iter < MAX_ITERATIONS) : (iter += 1) {
        const closest_idx = polytope.closestFace() orelse return null;
        const face = polytope.faces[closest_idx];
        const new_point = minkowskiSupport(a, b, face.normal);
        const dist = v3.dot(new_point, face.normal);

        if (dist - face.distance < TOLERANCE) {
            const contact_point = v3.scale(face.normal, face.distance);
            const world_point = v3.add(
                v3.scale(v3.add(a.position, b.position), 0.5),
                v3.scale(face.normal, face.distance * 0.5),
            );
            _ = contact_point;
            return Contact{
                .point = world_point,
                .normal = face.normal,
                .depth = face.distance,
                .body_id_a = a.local_id,
                .body_id_b = b.local_id,
            };
        }

        var edges: [256]Edge = undefined;
        var edge_count: u32 = 0;

        var fi: u32 = 0;
        while (fi < polytope.face_count) {
            const f = polytope.faces[fi];
            if (v3.dot(f.normal, v3.sub(new_point, polytope.vertices[f.a])) > 0) {
                addEdge(&edges, &edge_count, f.a, f.b);
                addEdge(&edges, &edge_count, f.b, f.c);
                addEdge(&edges, &edge_count, f.c, f.a);
                polytope.removeFace(fi);
            } else {
                fi += 1;
            }
        }

        const new_idx = polytope.addVertex(new_point);
        var ei: u32 = 0;
        while (ei < edge_count) : (ei += 1) {
            polytope.addFace(edges[ei].a, edges[ei].b, new_idx);
        }
    }

    const closest_idx = polytope.closestFace() orelse return null;
    const face = polytope.faces[closest_idx];
    return Contact{
        .point = v3.scale(v3.add(a.position, b.position), 0.5),
        .normal = face.normal,
        .depth = face.distance,
        .body_id_a = a.local_id,
        .body_id_b = b.local_id,
    };
}

fn addEdge(edges: *[256]Edge, count: *u32, a_idx: u32, b_idx: u32) void {
    var i: u32 = 0;
    while (i < count.*) {
        if (edges[i].a == b_idx and edges[i].b == a_idx) {
            if (i < count.* - 1) {
                edges[i] = edges[count.* - 1];
            }
            count.* -= 1;
            return;
        }
        i += 1;
    }
    if (count.* < 256) {
        edges[count.*] = .{ .a = a_idx, .b = b_idx };
        count.* += 1;
    }
}

fn fallbackContact(a: *const CollisionBody, b: *const CollisionBody) ?Contact {
    const dir = v3.sub(b.position, a.position);
    var normal = v3.normalize(dir);
    if (v3.lengthSq(normal) < TOLERANCE) {
        normal = v3.up;
    }
    const sa = shapes_mod.supportPoint(a, normal);
    const sb = shapes_mod.supportPoint(b, v3.negate(normal));
    const overlap = v3.dot(v3.sub(sa, sb), normal);
    if (overlap <= 0) return null;
    return Contact{
        .point = v3.scale(v3.add(sa, sb), 0.5),
        .normal = normal,
        .depth = overlap,
        .body_id_a = a.local_id,
        .body_id_b = b.local_id,
    };
}

test "epa sphere penetration depth" {
    const a = CollisionBody{
        .local_id = 1,
        .position = .{ .x = 0, .y = 0, .z = 0 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .sphere = .{ .radius = 2.0 } },
        .flags = 0,
        .is_avatar = false,
    };
    const b = CollisionBody{
        .local_id = 2,
        .position = .{ .x = 3, .y = 0, .z = 0 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .sphere = .{ .radius = 2.0 } },
        .flags = 0,
        .is_avatar = false,
    };
    const gjk_result = gjk_mod.gjk(&a, &b);
    try std.testing.expect(gjk_result.intersecting);
    const contact = epa(&a, &b, &gjk_result.simplex);
    try std.testing.expect(contact != null);
    if (contact) |c| {
        try std.testing.expectApproxEqAbs(c.depth, 1.0, 0.15);
        try std.testing.expectApproxEqAbs(c.normal.x, 1.0, 0.2);
    }
}

test "epa capsule on platform" {
    const platform = CollisionBody{
        .local_id = 1,
        .position = .{ .x = 0, .y = 0, .z = 50 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .box = .{ .half_extents = .{ .x = 5, .y = 5, .z = 0.25 } } },
        .flags = 0,
        .is_avatar = false,
    };
    const avatar = CollisionBody{
        .local_id = 2,
        .position = .{ .x = 0, .y = 0, .z = 50.7 },
        .rotation = .{ .x = 0, .y = 0, .z = 0, .w = 1 },
        .shape = .{ .capsule = .{ .radius = 0.37, .half_height = 0.53 } },
        .flags = 0,
        .is_avatar = true,
    };
    const gjk_result = gjk_mod.gjk(&platform, &avatar);
    try std.testing.expect(gjk_result.intersecting);
    const contact = epa(&platform, &avatar, &gjk_result.simplex);
    try std.testing.expect(contact != null);
    if (contact) |c| {
        try std.testing.expect(c.normal.z > 0.5);
        try std.testing.expect(c.depth > 0);
    }
}
