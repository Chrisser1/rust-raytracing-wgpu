struct Sphere {
    center: vec3<f32>,
    radius: f32,
    color: vec3<f32>,
}

struct Triangle {
    corner_a: vec3<f32>,
    corner_b: vec3<f32>,
    corner_c: vec3<f32>,
    color: vec3<f32>,
}

struct GeometricPrimitive {
    data_type: f32, // 0 for sphere, 1 for triangle
    data: array<f32, 16>, // Encoded data for both types
}

struct Node {
    minCorner: vec3<f32>,
    leftChild: f32,
    maxCorner: vec3<f32>,
    objectCount: f32,
}

struct BVH {
    nodes: array<Node>,
}

struct ObjectIndices {
    indices: array<f32>,
}

struct Ray {
    direction: vec3<f32>,
    origin: vec3<f32>,
}

struct SceneData {
    cameraPos: vec3<f32>,
    cameraForwards: vec3<f32>,
    cameraRight: vec3<f32>,
    maxBounces: f32,
    cameraUp: vec3<f32>,
    objectCount: f32,
}

struct RenderState {
    t: f32,
    color: vec3<f32>,
    hit: bool,
    position: vec3<f32>,
    normal: vec3<f32>,
}

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> scene: SceneData;
@group(0) @binding(2) var<storage, read> objects: array<GeometricPrimitive>;
@group(0) @binding(3) var<storage, read> tree: BVH;
@group(0) @binding(4) var<storage, read> objectLookup: ObjectIndices;
@group(0) @binding(5) var skyMaterial: texture_cube<f32>;
@group(0) @binding(6) var skySampler: sampler;

@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) GlobalInvocationID : vec3<u32>) {
    let screen_size: vec2<i32> = vec2<i32>(textureDimensions(color_buffer));
    let screen_pos : vec2<i32> = vec2<i32>(i32(GlobalInvocationID.x), i32(GlobalInvocationID.y));

    let horizontal_coefficient: f32 = (f32(screen_pos.x) - f32(screen_size.x) / 2) / f32(screen_size.x);
    let vertical_coefficient: f32 = (f32(screen_pos.y) - f32(screen_size.y) / 2) / f32(screen_size.x);

    let forwards: vec3<f32> = scene.cameraForwards;
    let right: vec3<f32> = scene.cameraRight;
    let up: vec3<f32> = scene.cameraUp;

    var myRay: Ray;
    myRay.direction = normalize(forwards + horizontal_coefficient * right + vertical_coefficient * up);
    myRay.origin = scene.cameraPos;

    // let pixel_color : vec3<f32> = rayColor(myRay);

    let pixel_color : vec3<f32> = rayColor(myRay);

    textureStore(color_buffer, screen_pos, vec4<f32>(pixel_color, 1.0));
}

fn rayColor (ray: Ray) -> vec3<f32> {
    var color: vec3<f32> = vec3(1.0, 1.0, 1.0);
    var result: RenderState;

    var temp_ray: Ray;
    temp_ray.origin = ray.origin;
    temp_ray.direction = ray.direction;
    
    let bounces: u32 = u32(scene.maxBounces);
    for (var bounce: u32 = 0; bounce < bounces; bounce++) {
        result = trace(temp_ray);

        //unpack color
        color = color * result.color;

        //early exit
        if (!result.hit) {
            break;
        }

        //Set up for next trace
        temp_ray.origin = result.position;
        temp_ray.direction = normalize(reflect(temp_ray.direction, result.normal));
    }

    //Rays which reached terminal state and bounced indefinitely
    if (result.hit) {
        color = vec3(0.0, 0.0, 0.0);
    }

    return color;
}

fn trace(ray: Ray) -> RenderState {
    // Set up the render state 
    var renderState: RenderState;

    // Set up the start
    renderState.hit = false;
    var nearestHit: f32 = 9999.0;

    // Set up for BVH traversal
    var node: Node = tree.nodes[0];
    var stack: array<Node, 15>;
    var stackLocation: i32 = 0;

    while (true) {
        var objectCount: u32 = u32(node.objectCount);
        var contents: u32 = u32(node.leftChild);

        if (objectCount == 0) {
            var child1: Node = tree.nodes[contents];
            var child2: Node = tree.nodes[contents + 1];

            var distance1: f32 = hit_aabb(ray, child1);
            var distance2: f32 = hit_aabb(ray, child2);
            if (distance1 > distance2) {
                var tempDist: f32 = distance1;
                distance1 = distance2;
                distance2 = tempDist;

                var tempChild: Node = child1;
                child1 = child2;
                child2 = tempChild;
            }

            if (distance1 > nearestHit) {
                if (stackLocation == 0) {
                    break;
                }
                else {
                    stackLocation -= 1;
                    node = stack[stackLocation];
                }
            }
            else {
                node = child1;
                if (distance2 < nearestHit) {
                    stack[stackLocation] = child2;
                    stackLocation += 1;
                }
            }
        }
        else {
            for (var i: u32 = 0; i < objectCount; i++) {
                var newRenderState: RenderState = hit_geometric_primitive(
                    ray, 
                    objects[u32(objectLookup.indices[i + contents])], 
                    0.001, nearestHit, renderState
                );

                if (newRenderState.hit) {
                    nearestHit = newRenderState.t;
                    renderState = newRenderState;
                }
            }

            if (stackLocation == 0) {
                break;
            }
            else {
                stackLocation -= 1;
                node = stack[stackLocation];
            }
        }
    }

    if (!renderState.hit) {
        // Sky color 
        renderState.color = textureSampleLevel(skyMaterial, skySampler, ray.direction, 0.0).xyz;
    }
    
    return renderState;
}

// Function to decode a Sphere from the GeometricPrimitive data array
fn decode_sphere(data: array<f32, 16>) -> Sphere {
    return Sphere(
        vec3(data[0], data[1], data[2]), // Center
        data[3], // Radius
        vec3(data[4], data[5], data[6]), // Color
    );
}

// Function to decode a Triangle from the GeometricPrimitive data array
fn decode_triangle(data: array<f32, 16>) -> Triangle {
    return Triangle(
        vec3(data[7], data[8], data[9]), // corner_a
        vec3(data[10], data[11], data[12]), // corner_b
        vec3(data[13], data[14], data[15]), // corner_c
        vec3(data[4], data[5], data[6]) // color
    );
}

// Function to interpret the GeometricPrimitive and perform collision detection
fn hit_geometric_primitive(ray: Ray, primitive: GeometricPrimitive, tMin: f32, tMax: f32, renderState: RenderState) -> RenderState {
    var state: RenderState;
    if (primitive.data_type == 0.0) {
        // Sphere
        let sphere: Sphere = decode_sphere(primitive.data);
        state = hit_sphere(ray, sphere, tMin, tMax, renderState);
    } else if (primitive.data_type == 1.0) {
        // Triangle
        let triangle: Triangle = decode_triangle(primitive.data);
        state = hit_triangle(ray, triangle, tMin, tMax, renderState);
    }
    return state;
}

fn hit_sphere(ray: Ray, sphere: Sphere, tMin: f32, tMax: f32, oldRenderState: RenderState) -> RenderState {
    
    let co: vec3<f32> = ray.origin - sphere.center;
    let a: f32 = dot(ray.direction, ray.direction);
    let b: f32 = 2.0 * dot(ray.direction, co);
    let c: f32 = dot(co, co) - sphere.radius * sphere.radius;
    let discriminant: f32 = b * b - 4.0 * a * c;

    var renderState: RenderState;
    renderState.color = oldRenderState.color;

    if (discriminant > 0.0) {

        let t: f32 = (-b - sqrt(discriminant)) / (2 * a);

        if (t > tMin && t < tMax) {

            renderState.position = ray.origin + t * ray.direction;
            renderState.normal = normalize(renderState.position - sphere.center);
            renderState.t = t;
            renderState.color = sphere.color;
            renderState.hit = true;
            return renderState;
        }
    }

    renderState.hit = false;
    return renderState;
    
}

fn hit_triangle(ray: Ray, tri: Triangle, tMin: f32, tMax: f32, oldRenderState: RenderState) -> RenderState {
    
    //Set up a blank renderstate,
    //right now this hasn't hit anything
    var renderState: RenderState;
    renderState.color = oldRenderState.color;
    renderState.hit = false;

    //Direction vectors
    let edge_ab: vec3<f32> = tri.corner_b - tri.corner_a;
    let edge_ac: vec3<f32> = tri.corner_c - tri.corner_a;
    //Normal of the triangle
    var n: vec3<f32> = normalize(cross(edge_ab, edge_ac));
    var ray_dot_tri: f32 = dot(ray.direction, n);
    //backface reversal
    if (ray_dot_tri > 0.0) {
        ray_dot_tri = ray_dot_tri * -1;
        n = n * -1;
    }
    //early exit, ray parallel with triangle surface
    if (abs(ray_dot_tri) < 0.00001) {
        return renderState;
    }

    var system_matrix: mat3x3<f32> = mat3x3<f32>(
        ray.direction,
        tri.corner_a - tri.corner_b,
        tri.corner_a - tri.corner_c
    );
    let denominator: f32 = determinant(system_matrix);
    if (abs(denominator) < 0.00001) {
        return renderState;
    }

    system_matrix = mat3x3<f32>(
        ray.direction,
        tri.corner_a - ray.origin,
        tri.corner_a - tri.corner_c
    );
    let u: f32 = determinant(system_matrix) / denominator;
    
    if (u < 0.0 || u > 1.0) {
        return renderState;
    }

    system_matrix = mat3x3<f32>(
        ray.direction,
        tri.corner_a - tri.corner_b,
        tri.corner_a - ray.origin,
    );
    let v: f32 = determinant(system_matrix) / denominator;
    if (v < 0.0 || u + v > 1.0) {
        return renderState;
    }

    system_matrix = mat3x3<f32>(
        tri.corner_a - ray.origin,
        tri.corner_a - tri.corner_b,
        tri.corner_a - tri.corner_c
    );
    let t: f32 = determinant(system_matrix) / denominator;

    if (t > tMin && t < tMax) {

        renderState.position = ray.origin + t * ray.direction;
        renderState.normal = n;
        renderState.color = tri.color;
        renderState.t = t;
        renderState.hit = true;
        return renderState;
    }

    return renderState;
}

fn hit_aabb(ray: Ray, node: Node) -> f32 {
    var inverseDir: vec3<f32> = vec3(1.0) / ray.direction;
    var t1: vec3<f32> = (node.minCorner - ray.origin) * inverseDir;
    var t2: vec3<f32> = (node.maxCorner - ray.origin) * inverseDir;
    var tMin: vec3<f32> = min(t1, t2);
    var tMax: vec3<f32> = max(t1, t2);

    var t_min: f32 = max(max(tMin.x, tMin.y), tMin.z);
    var t_max: f32 = min(min(tMax.x, tMax.y), tMax.z);

    if (t_min > t_max || t_max < 0) {
        return 99999.0;
    }
    else {
        return t_min;
    }
}