
fn get_offset(
    pos_x: i32, 
    pos_y: i32, 
    size_w: usize, 
    size_h: usize) 
    -> usize
{
    let pos_x = pos_x.clamp(0, (size_w - 1) as i32);
    let pos_y = pos_y.clamp(0, (size_h - 1) as i32);
    (pos_y * (size_w as i32) + pos_x) as usize
}

pub fn rgba_to_grayscale(
    src: &[u8],
    dst: &mut [u8],
    stride: usize)
{
    for offset in 0..dst.len() {
        dst[offset] = src[offset * stride];
    }
}

pub fn edge_detect(
    src: &[u8],
    dst: &mut [u8],
    width: usize,
    height: usize) 
{
    let size_x = width;
    let size_y = height;

    for pos_y in 0..height {
        for pos_x in 0..width {
            let pos_x = pos_x as i32;
            let pos_y = pos_y as i32;
            let offset = 
                get_offset(pos_x, pos_y, size_x, size_y);

            if offset == 268800 {
                ;
            }

            if src[offset] >= 128 {
                let mut is_edge = false;
                
                for pos1_y in pos_y - 1 ..= pos_y + 1 {
                    for pos1_x in pos_x - 1 ..= pos_x + 1 {
                        if pos1_x != pos_x || pos1_y != pos_y {
                            let offset =
                                get_offset(pos1_x, pos1_y, size_x, size_y);
                            if src[offset] < 128 {
                                is_edge = true;
                                break;
                            }
                        }
                    }
                }

                dst[offset] = if is_edge { 255 } else { 127 };
            }
            else {
                dst[offset] = 0;
            }
        }
    }
}

pub fn sdf_generate(
    edge: &[u8],
    sdf: &mut [u8],
    edge_width: usize,
    edge_height: usize,
    sdf_width: usize,
    sdf_height: usize,
    stride: usize,
    search_radius: usize) 
{
    let sdf_size_x = sdf_width;
    let sdf_size_y = sdf_height;
    let edge_size_x = edge_width;
    let edge_size_y = edge_height;

    for sdf_pos_y in 0..sdf_height {
        for sdf_pos_x in 0..sdf_width {
            let sdf_pos_x = sdf_pos_x as i32;
            let sdf_pos_y = sdf_pos_y as i32;

            let sdf_offset = 
                get_offset(sdf_pos_x, sdf_pos_y, sdf_size_x, sdf_size_y);
    
            let edge_pos_x = (sdf_pos_x * stride as i32 + stride as i32 / 2) as i32;
            let edge_pos_y = (sdf_pos_y * stride as i32 + stride as i32 / 2) as i32;
            let edge_offset = 
                get_offset(
                    edge_pos_x, 
                    edge_pos_y, 
                    edge_size_x, 
                    edge_size_y);

            let is_inner = 
                edge[edge_offset] > 96;

            let mut min_distance = if is_inner { 127 } else { -127 };

            'outer: for distance in 1..=(search_radius as i32) {
                for t in -distance..=distance {
                    let top = 
                        get_offset(
                            edge_pos_x + t,
                            edge_pos_y - distance,
                            edge_size_x,
                            edge_size_y);

                    let bottom = 
                        get_offset(
                            edge_pos_x + t,
                            edge_pos_y + distance,
                            edge_size_x,
                            edge_size_y);

                    let left = 
                        get_offset(
                            edge_pos_x - distance,
                            edge_pos_y + t, 
                            edge_size_x,
                            edge_size_y);

                    let right = 
                        get_offset(
                            edge_pos_x + distance, 
                            edge_pos_y + t, 
                            edge_size_x,
                            edge_size_y);


                    if edge[top] > 192 || edge[bottom] > 192 || edge[left] > 192 || edge[right] > 192 {
                        let min_distancef = 
                            ((t * t + distance * distance) as f32).sqrt() / (search_radius as f32);

                        let min_distancef = min_distancef.clamp(0.0, 1.0);
                        min_distance = (min_distancef * 127.0) as i32 * (if is_inner { 1 } else { -1 });
                        break 'outer;
                    }
                }
            }
            sdf[sdf_offset] = (min_distance + 127) as u8;
        }
    }
}



    
