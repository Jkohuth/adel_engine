
pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            layout(location = 0) in vec2 position;
            layout(location = 1) in vec3 color;
            layout(location = 0) out vec3 fragColor; 
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                fragColor = color;
            }
        "
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec3 fragColor;
            layout(location = 0) out vec4 outColor;
            void main() {
                outColor = vec4(fragColor, 1.0);
            }
        "
    }
}


pub mod fs_texture {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec2 tex_coords;
            layout(location = 0) out vec4 f_color;

            layout(set = 0, binding = 0) uniform sampler2D tex;

            void main() {
                f_color = texture(tex, tex_coords);
            }
        "
    }
}
pub mod vs_texture {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450

            layout(location = 0) in vec2 position;
            layout(location = 0) out vec2 tex_coords;

            void main() {
                gl_Position = vec4(position.x * 2, position.y *2, 0.0, 1.0);
                tex_coords = position + vec2(0.5);
            }
        "
    }
}
pub mod vs_2d_push {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            layout(location = 0) in vec2 position;
            layout(location = 1) in vec3 color;
            layout(location = 0) out vec3 outColor;
            layout(push_constant) uniform PushConstantData2d {
                mat2 transform;
                vec2 offset;
                vec3 color;
            } push;
            void main() {
                gl_Position = vec4(push.transform * position + push.offset, 0.0, 1.0);
                outColor = color;
            }
        "
    }
}

pub mod fs_2d_push {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec3 inColor;
            layout(location = 0) out vec4 outColor;
            layout(push_constant) uniform PushConstantData2d {
                mat2 transform;
                vec2 offset;
                vec3 color;
            } push;
            void main() {
                outColor = vec4(inColor + push.color, 1.0);
            }
        "
    }
}

pub mod vs_push {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 450
            layout(location = 0) in vec3 position;
            layout(location = 1) in vec3 color;
            layout(location = 0) out vec3 outColor;
            layout(push_constant) uniform PushConstantData {
                mat4 transform;
                vec3 color;
            } push;
            void main() {
                //gl_Position = vec4(push.transform * position + push.offset, 0.0, 1.0);
                gl_Position = push.transform * vec4(position, 1.0); 
                outColor = color;
            }
        "
    }
}

pub mod fs_push {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 450

            layout(location = 0) in vec3 inColor;
            layout(location = 0) out vec4 outColor;
            layout(push_constant) uniform PushConstantData {
                mat4 transform;
                vec3 color;
            } push;
            void main() {
                outColor = vec4(inColor + push.color, 1.0);
            }
        "
    }
}