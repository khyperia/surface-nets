mod array;

use array::Array;

// Positive is "air"
// Negative is "solid"

pub fn surface_net<F: Fn((usize, usize, usize)) -> f32>(
    resolution: usize,
    func: F,
) -> (Vec<(f32, f32, f32)>, Vec<usize>) {
    let grid_values = Array::create_from(resolution + 1, func);
    let mut vertex_positions = Vec::new();
    let grid_to_index = Array::create_from(resolution, |coords| {
        match find_center(&grid_values, coords) {
            Some(x) => {
                let index = vertex_positions.len();
                vertex_positions.push(x);
                index
            }
            None => usize::max_value(),
        }
    });
    let mut indicies = Vec::new();
    make_all_triangles(&grid_values, resolution, &grid_to_index, &mut indicies);
    (vertex_positions, indicies)
}

const OFFSETS: [((usize, usize, usize), (usize, usize, usize)); 12] = [
    ((0, 0, 0), (0, 0, 1)),
    ((0, 0, 0), (0, 1, 0)),
    ((0, 0, 0), (1, 0, 0)),
    ((0, 0, 1), (0, 1, 1)),
    ((0, 0, 1), (1, 0, 1)),
    ((0, 1, 0), (0, 1, 1)),
    ((0, 1, 0), (1, 1, 0)),
    ((0, 1, 1), (1, 1, 1)),
    ((1, 0, 0), (1, 0, 1)),
    ((1, 0, 0), (1, 1, 0)),
    ((1, 0, 1), (1, 1, 1)),
    ((1, 1, 0), (1, 1, 1)),
];

fn find_center(
    grid_values: &Array<Vec<f32>>,
    coord: (usize, usize, usize),
) -> Option<(f32, f32, f32)> {
    let edges = OFFSETS
        .iter()
        .filter_map(|&(offset1, offset2)| find_edge(grid_values, coord, offset1, offset2));
    let mut count = 0;
    let mut sum = (0.0, 0.0, 0.0);
    for edge in edges {
        count += 1;
        sum = (sum.0 + edge.0, sum.1 + edge.1, sum.2 + edge.2);
    }
    if count == 0 {
        None
    } else {
        Some((
            sum.0 / count as f32,
            sum.1 / count as f32,
            sum.2 / count as f32,
        ))
    }
}

fn find_edge(
    grid_values: &Array<Vec<f32>>,
    coord: (usize, usize, usize),
    offset1: (usize, usize, usize),
    offset2: (usize, usize, usize),
) -> Option<(f32, f32, f32)> {
    let value1 = grid_values[(
        offset1.0 + coord.0,
        offset1.1 + coord.1,
        offset1.2 + coord.2,
    )];
    let value2 = grid_values[(
        offset2.0 + coord.0,
        offset2.1 + coord.1,
        offset2.2 + coord.2,
    )];
    if (value1 < 0.0) == (value2 < 0.0) {
        return None;
    }
    let interp = value1 / (value1 - value2);
    let point = (
        offset1.0 as f32 * (1.0 - interp) + offset2.0 as f32 * interp + coord.0 as f32,
        offset1.1 as f32 * (1.0 - interp) + offset2.1 as f32 * interp + coord.1 as f32,
        offset1.2 as f32 * (1.0 - interp) + offset2.2 as f32 * interp + coord.2 as f32,
    );
    Some(point)
}

fn make_all_triangles(
    grid_values: &Array<Vec<f32>>,
    resolution: usize,
    grid_to_index: &Array<Vec<usize>>,
    indicies: &mut Vec<usize>,
) {
    for x in 0..resolution {
        for y in 0..resolution {
            for z in 0..resolution {
                if y != 0 && z != 0 {
                    make_triangles(
                        grid_values,
                        grid_to_index,
                        indicies,
                        (x, y, z),
                        (1, 0, 0),
                        (0, 1, 0),
                        (0, 0, 1),
                    );
                }
                if x != 0 && z != 0 {
                    make_triangles(
                        grid_values,
                        grid_to_index,
                        indicies,
                        (x, y, z),
                        (0, 1, 0),
                        (0, 0, 1),
                        (1, 0, 0),
                    );
                }
                if x != 0 && y != 0 {
                    make_triangles(
                        grid_values,
                        grid_to_index,
                        indicies,
                        (x, y, z),
                        (0, 0, 1),
                        (1, 0, 0),
                        (0, 1, 0),
                    );
                }
            }
        }
    }
}

fn make_triangles(
    grid_values: &Array<Vec<f32>>,
    grid_to_index: &Array<Vec<usize>>,
    indicies: &mut Vec<usize>,
    coord: (usize, usize, usize),
    offset: (usize, usize, usize),
    other_axis1: (usize, usize, usize),
    other_axis2: (usize, usize, usize),
) {
    let v1 = grid_to_index[(coord.0, coord.1, coord.2)];
    let v2 = grid_to_index[(
        coord.0 - other_axis1.0,
        coord.1 - other_axis1.1,
        coord.2 - other_axis1.2,
    )];
    let v3 = grid_to_index[(
        coord.0 - other_axis2.0,
        coord.1 - other_axis2.1,
        coord.2 - other_axis2.2,
    )];
    let v4 = grid_to_index[(
        coord.0 - other_axis1.0 - other_axis2.0,
        coord.1 - other_axis1.1 - other_axis2.1,
        coord.2 - other_axis1.2 - other_axis2.2,
    )];
    if v1 == usize::max_value() || v2 == usize::max_value() || v3 == usize::max_value() {
        return;
    }
    match is_face(grid_values, coord, offset) {
        FaceResult::NoFace => (),
        FaceResult::FacePositive => {
            indicies.push(v1);
            indicies.push(v2);
            indicies.push(v4);

            indicies.push(v1);
            indicies.push(v4);
            indicies.push(v3);
        }
        FaceResult::FaceNegative => {
            indicies.push(v1);
            indicies.push(v4);
            indicies.push(v2);

            indicies.push(v1);
            indicies.push(v3);
            indicies.push(v4);
        }
    }
}

enum FaceResult {
    NoFace,
    FacePositive,
    FaceNegative,
}

fn is_face(
    grid_values: &Array<Vec<f32>>,
    coord: (usize, usize, usize),
    offset: (usize, usize, usize),
) -> FaceResult {
    let other = (coord.0 + offset.0, coord.1 + offset.1, coord.2 + offset.2);
    match (grid_values[coord] < 0.0, grid_values[other] < 0.0) {
        (true, false) => FaceResult::FacePositive,
        (false, true) => FaceResult::FaceNegative,
        _ => FaceResult::NoFace,
    }
}
