#version 450

layout(location = 0) out uint out_color;

layout(set = 1, binding = 0) uniform NodeId {
    uint id;
};

void main() {
    out_color = id;
}