use macroquad::{
    prelude::*,
    ui::{root_ui, widgets},
};

#[macroquad::main("mandelbrot-set")]
async fn main() {
    let texture = render_target(1000, 1000).texture;
    let material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                ("screen_size".to_owned(), UniformType::Float2),
                ("position".to_owned(), UniformType::Float2),
                ("zoom".to_owned(), UniformType::Float1),
            ],
            ..Default::default()
        },
    )
    .unwrap();

    let mut pos_x: f32 = 0.0;
    let mut pos_y: f32 = 0.0;
    let mut speed: f32 = 0.55;
    let mut zoom: f32 = 1.0;

    let mut mandel_inc = 1;

    set_default_camera();
    loop {
        // move this stuff somewhere else
        let dt = get_frame_time();
        if is_key_down(KeyCode::A) {
            pos_x -= speed * dt;
        }
        if is_key_down(KeyCode::D) {
            pos_x += speed * dt;
        }
        if is_key_down(KeyCode::W) {
            pos_y += speed * dt;
        }
        if is_key_down(KeyCode::S) {
            pos_y -= speed * dt;
        }
        if is_key_pressed(KeyCode::Q) {
            zoom -= 0.05;
        }
        if is_key_pressed(KeyCode::E) {
            zoom += 0.05;
        }

        clear_background(WHITE);

        gl_use_material(&material);
        material.set_uniform("screen_size", (screen_width(), screen_height()));
        material.set_uniform("position", (pos_x, pos_y));
        material.set_uniform("zoom", zoom);

        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();

        // this is very temporary
        // and extermly jank
        if is_key_pressed(KeyCode::Space) {
            let filename = format!("images/mandelbrot_{}.png", mandel_inc);
            get_screen_data().export_png(&filename);
            mandel_inc += 1;
        }

        next_frame().await;
    }
}

const FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;

uniform vec2 screen_size;
uniform vec2 position;
uniform float zoom;

void main() {
    gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);

// vec2 coord = gl_FragCoord.xy / screen_size.xy;
//     vec2 scaled = vec2(
//         coord.x * (0.47 - -2.00) + -2.00,
//         coord.y * (1.12 - -1.12) + -1.12
//     );
//
    vec2 scaled = (2.0*gl_FragCoord.xy - screen_size.xy) / screen_size.y;

    scaled.x += position.x;
    scaled.y += position.y;
    scaled *= zoom;

    float x = 0.0;
    float y = 0.0;
    float tmp = 0.0;
    
    // make this a uniform
    int iter_max = 1000;
    int i = 0;



    for(; i < iter_max; i++){
        if(x*x + y*y > 4.0){
            break;
        }
        
        tmp = x*x - y*y + scaled.x;
        y = 2.0 * x * y + scaled.y;
        x = tmp;
    }

    float fi = float(i);
    float i_norm = (float(i)/float(iter_max)); 

    // need to figure out an actual good method of coloring 
    if(i < iter_max){
        gl_FragColor.r = i_norm * fi;
        gl_FragColor.g = i_norm * fi/2.0;
        gl_FragColor.b = i_norm * fi/2.0;
    }
}

"#;

const VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
}
";
