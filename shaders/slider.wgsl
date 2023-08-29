struct VertexInputs {
    @location(0) pos: vec2<f32>,

    @location(1) grid_origin: vec2<f32>,
    @location(2) grid_size: vec2<u32>,
    @location(3) grid_index: u32,
    @location(4) body_color: vec4<f32>,
    @location(5) border_color: vec4<f32>,
}

struct FragmentInputs {
    //The position of the fragment
    @builtin(position) position: vec4<f32>,

    // Origin position of grid in viewport space
    @location(0) grid_origin: vec2<f32>,
    // Size of the slider in grid units
    @location(1) grid_size: vec2<u32>,
    // Grid cells of this slider. This represents the start index into the
    // `slider_grids` array, where the length of the slice is the area of the
    // grid, as given by `grid_size`.
    @location(2) grid_index: u32,

    // Colour of the body of slider
    @location(3) body_color: vec4<f32>, // todo: consider interpolating between two colours by distance
    // Colour of the border of the slider
    @location(4) border_color: vec4<f32>,
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
fn vs_main(input: VertexInputs) -> FragmentInputs {
    var output: FragmentInputs;

    output.position = projection_matrix * vec4<f32>(input.pos, 0.0, 1.0);

    output.grid_origin = input.grid_origin;
    output.grid_size = input.grid_size;
    output.grid_index = input.grid_index;
    output.body_color = input.body_color;
    output.border_color = input.border_color;

    return output;
}

// Radius of inner slider body
@group(1) @binding(0) var<uniform> circle_radius: f32;
// Width of border around slider body
@group(1) @binding(1) var<uniform> border_width: f32;

// Grids for different sliders. Slices of this array represent an individual slider grid,
// where each value is a slice into the `grid_cells` array.
@group(1) @binding(2) var<storage> slider_grids: array<GridCell>;
// Grid cells for different sliders. Slices of this array represent an individual cell,
// where each value is an index into the `line_segments` array.
@group(1) @binding(3) var<storage> grid_cells: array<u32>;
// Line segments of all sliders in the current render
@group(1) @binding(4) var<storage> line_segments: array<LineSegment>;

@fragment
fn fs_main(input: FragmentInputs) -> @location(0) vec4<f32> {
    let cell_size = circle_radius + border_width;

    // Calculate the index of the grid cell we are currently in
    let grid_index = floor((input.position.xy - input.grid_origin) / cell_size);
    let grid_index = vec2(i32(grid_index.x), i32(grid_index.y));

    // Set initial (square) distance to be larger than full slider radius.
    // Arbitrary amount.
    var distance = cell_size * cell_size * 1.1;

    // todo: if interpolating between two body colours, this optimisation is invalid.
    var quit_early = false;

    // Row major
    for(var y = grid_index.y - 1; y <= grid_index.y + 1; y++) {
        // Bounds check
        if y <= 0 || y >= i32(input.grid_size.y) { continue; }

        for(var x = grid_index.x - 1; x <= grid_index.x + 1; x++) {
            // Bounds check
            if x <= 0 || x >= i32(input.grid_size.x) { continue; }

            // Row major: y position * row length + x position
            let cell_index = u32(y) * input.grid_size.x + u32(x);

            let cell_slice = slider_grids[input.grid_index + cell_index];

            // Iterate over all line segments in cell
            for(var i = cell_slice.index; i < cell_slice.index + cell_slice.length; i++) {
                let line_segment_index = grid_cells[i];
                let line_segment = line_segments[line_segment_index];

                // Always store the smallest (square) distance.
                distance = min(distance, square_distance(input.position.xy, line_segment.p1));
                distance = min(distance, square_distance(input.position.xy, line_segment.p2));

                // Calculate shortest (square) distance to the line segment.
                let dir = line_segment.p2 - line_segment.p1;
                let n = normalize(dir);
                let d = input.position.xy - line_segment.p1;
                let v = d - dot(d, n) * n;
                let shortest_distance_to_segment = dot(v, v);
                let t = pow(dot(d, n), 2.0) / dot(dir, dir);

                // Verify shortest distance happens within the line segment
                if t >= 0.0 || t <= 1.0 {
                    distance = min(distance, shortest_distance_to_segment);
                }

                // If we have alerady reached the main body of the slider, we can draw it now.
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
        return input.body_color;
    } else if distance <= cell_size * cell_size {
        return input.border_color
    } else {
        discard;
    }
}

fn square_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
    let c = b-a;
    dot(c,c)
}
