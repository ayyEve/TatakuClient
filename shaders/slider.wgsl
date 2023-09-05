struct VertexInputs {
    @location(0) pos: vec2<f32>,

    @location(1) slider_index: u32,
}

struct FragmentInputs {
    //The position of the fragment
    @builtin(position) position: vec4<f32>,

    // Index into the `slider_data` array.
    @location(0) slider_index: u32,
}

struct SliderData {
    // Origin position of grid in viewport space
    grid_origin: array<f32, 2>,
    // Size of the slider in grid units
    grid_size: array<u32, 2>,
    // Grid cells of this slider. This represents the start index into the
    // `slider_grids` array, where the length of the slice is the area of the
    // grid, as given by `grid_size`.
    grid_index: u32,

    // Colour of the body of slider
    body_color: array<f32, 4>, // todo: consider interpolating between two colours by distance
    // Colour of the border of the slider
    border_color: array<f32, 4>,
}

// Slice into the index buffer representing the grid cell
struct GridCell {
    // Starting index for slice in `grid_cells` array
    index: u32,
    // Length of slice in `grid_cells` array
    length: u32,
}

// An individual line segment for a slider.
struct LineSegment {
    p1: vec2<f32>,
    p2: vec2<f32>,
}

@group(0) @binding(0) var<uniform> projection_matrix: mat4x4<f32>;

@vertex
fn slider_vs_main(input: VertexInputs) -> FragmentInputs {
    var output: FragmentInputs;

    output.position = projection_matrix * vec4<f32>(input.pos, 0.0, 1.0);

    output.slider_index = input.slider_index;

    return output;
}

// Radius of inner slider body
@group(1) @binding(0) var<uniform> circle_radius: f32;
// Width of border around slider body
@group(1) @binding(1) var<uniform> border_width: f32;

// Per slider data
@group(1) @binding(2) var<storage> slider_data: array<SliderData>;

// Grids for different sliders. Slices of this array represent an individual slider grid,
// where each value is a slice into the `grid_cells` array.
@group(1) @binding(3) var<storage> slider_grids: array<GridCell>;
// Grid cells for different sliders. Slices of this array represent an individual cell,
// where each value is an index into the `line_segments` array.
@group(1) @binding(4) var<storage> grid_cells: array<u32>;
// Line segments of all sliders in the current render
@group(1) @binding(5) var<storage> line_segments: array<LineSegment>;

@fragment
fn slider_fs_main(input: FragmentInputs) -> @location(0) vec4<f32> {
    let cell_size = circle_radius + border_width;

    let slider = slider_data[input.slider_index];

    let slider_grid_origin = cast_vec2_f32(slider.grid_origin);
    let position = input.position.xy - slider_grid_origin;

    // Calculate the index of the grid cell we are currently in
    let grid_index_bad = floor(position / cell_size);
    let grid_index = vec2(i32(grid_index_bad.x), i32(grid_index_bad.y));

    // Set initial (square) distance to be larger than full slider radius.
    // Arbitrary amount.
    var distance = cell_size * cell_size * 1.1;

    // todo: if interpolating between two body colours, this optimisation is invalid.
    var quit_early = false;

    // Row major
    for(var y = grid_index.y - 1; y <= grid_index.y + 1; y++) {
        // Bounds check
        if y < 0 || y >= i32(slider.grid_size[1]) { continue; }

        for(var x = grid_index.x - 1; x <= grid_index.x + 1; x++) {
            // Bounds check
            if x < 0 || x >= i32(slider.grid_size[0]) { continue; }

            // Row major: y position * row length + x position
            let cell_index = u32(y) * slider.grid_size[0] + u32(x);

            let cell_slice = slider_grids[slider.grid_index + cell_index];

            // Iterate over all line segments in cell
            for(var i = cell_slice.index; i < cell_slice.index + cell_slice.length; i++) {
                let line_segment_index = grid_cells[i];
                let line_segment = line_segments[line_segment_index];

                // Always store the smallest (square) distance.
                distance = min(distance, square_distance(position, line_segment.p1));
                distance = min(distance, square_distance(position, line_segment.p2));

                // Calculate shortest (square) distance to the line segment.
                let dir = line_segment.p2 - line_segment.p1;
                let n = normalize(dir);
                let d = position - line_segment.p1;
                let v = d - dot(d, n) * n;
                let shortest_distance_to_segment = dot(v, v);
                let t = dot(d, dir) / dot(dir, dir);

                // Verify shortest distance happens within the line segment
                if t >= 0.0 && t <= 1.0 {
                    distance = min(distance, shortest_distance_to_segment);
                }

                // If we have already reached the main body of the slider, we can draw it now.
                quit_early = distance <= circle_radius * circle_radius;
                if quit_early {
                    break;
                }
            }

            if quit_early {
                break;
            }
        }

        if quit_early {
            break;
        }
    }

    if quit_early {
        return cast_vec4_f32(slider.body_color);
    } else if distance <= cell_size * cell_size {
        return cast_vec4_f32(slider.border_color);
    } else {
        // return vec4(0.0, 0.0, 0.0, 0.4);
        discard;
    }
}

fn square_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
    let c = b-a;
    return dot(c,c);
}

fn cast_vec2_f32(a: array<f32, 2>) -> vec2<f32> {
    return vec2(a[0], a[1]);
}
fn cast_vec4_f32(a: array<f32, 4>) -> vec4<f32> {
    return vec4(a[0], a[1], a[2], a[3]);
}