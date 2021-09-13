
size_t get_offset(int2 pos, int2 size) 
{
    pos.x = clamp(pos.x, 0, size.x - 1);
    pos.y = clamp(pos.y, 0, size.y - 1);
    return pos.y * size.x + pos.x;
}

__kernel void rgba_to_grayscale(
    __global const uchar *src,
    __global uchar *dst,
    int stride)
{
    const int offset = get_global_id(0);
    dst[offset] = src[offset * stride];
}

__kernel void edge_detect(
    __global const uchar *src,
    __global uchar *dst,
    int width,
    int height) 
{
    const int2 pos = (int2)(get_global_id(0), get_global_id(1));
    const int2 size = (int2)(width, height);
    const int offset = get_offset(pos, size);

    if(src[offset] >= 128) 
    {
        bool is_edge = false;
        int2 pos1;
        for(pos1.y = pos.y - 1; pos1.y <= pos.y + 1; ++ pos1.y)
        {
            for(pos1.x = pos.x - 1; pos1.x <= pos.x + 1; ++ pos1.x) 
            {
                if(pos1.x != pos.x || pos1.y != pos.y) 
                {
                    if(src[get_offset(pos1, size)] < 128)
                    {
                        is_edge = true;
                        break;
                    }
                }
            }
        }

        dst[offset] = is_edge ? 255 : 127;
    }
    else 
    {
        dst[offset] = 0;
    }
}

__kernel void sdf_generate(
    __global const uchar *edge,
    __global uchar *sdf,
    int edge_width,
    int edge_height,
    int sdf_width,
    int sdf_height,
    int stride,
    int search_radius) 
{
    const int2 sdf_pos = (int2)(get_global_id(0), get_global_id(1));
    const int2 sdf_size = (int2)(sdf_width, sdf_height);
    const int sdf_offset = get_offset(sdf_pos, sdf_size);
    
    const int2 edge_pos = sdf_pos * stride + (int2)(stride / 2, stride / 2);
    const int2 edge_size = (int2)(edge_width, edge_height);

    bool is_inner = edge[get_offset(edge_pos, edge_size)] > 96;

    int min_distance = is_inner ? 127 : -127;

    for(int distance = 1; distance <= search_radius; ++ distance)
    {
        for(int t = -distance; t <= distance; ++t) 
        {
            const int top = get_offset(edge_pos + (int2)(t, -distance), edge_size);
            const int bottom = get_offset(edge_pos + (int2)(t, distance), edge_size);
            const int left = get_offset(edge_pos + (int2)(-distance, t), edge_size);
            const int right = get_offset(edge_pos + (int2)(distance, t), edge_size);

            if(edge[top] > 192 || edge[bottom] > 192 || edge[left] > 192 || edge[right] > 192)
            {
                float min_distancef = length((float2)(t, distance)) / (float)search_radius;
                min_distancef = clamp(min_distancef, 0.0f, 1.0f);
                min_distance = (int)(min_distancef * 127) * (is_inner ? 1 : -1);
                goto BREAK;
            }
        }
    }

BREAK:
    sdf[sdf_offset] = min_distance + 127;
}
