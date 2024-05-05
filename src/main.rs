use std::env;

use macroquad::{
    file, prelude::*, telemetry::frame, ui::{root_ui, widgets}
};

#[macroquad::main("mandelbrot-set")]
async fn main() {
    let texture = render_target(16, 9).texture;
    let material = load_material(
        ShaderSource::Glsl {
            vertex: DOUBLE_VERTEX_SHADER,
            fragment: DOUBLE_FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                ("screen_size".to_owned(), UniformType::Float2),
                ("position".to_owned(), UniformType::Float2),
                ("zoom".to_owned(), UniformType::Float1),
                ("max_iterations".to_owned(), UniformType::Int1),
                ("time".to_owned(), UniformType::Float1),
            ],
            ..Default::default()
        },
    )
    .unwrap();

    let args: Vec<String> = env::args().collect();
    let arg_iter = args.get(1);

    let mut pos_x: f32 = 0.;
    let mut pos_y: f32 = 0.;
    let mut zoom: f32 = 1.;

    // as far as i can go currently
    // let mut pos_x: f32 = -0.57732594;
    // let mut pos_y: f32 = 0.5470669;
    //let mut zoom: f32 = 0.0000036658892;

    // let mut pos_x: f32 = -0.83442765;
    // let mut pos_y: f32 = 0.2046405;
    // let mut zoom:f32 = 77624730000.0;

    let mut max_iterations = match arg_iter {
        Some(s) => s.parse::<i32>().unwrap_or(1000),
        None => 1000,
    };

    let speed: f32 = 0.55;

    // let mut mandel_inc = 1;
    let mut recording = false;
    let mut frames:Vec<Image> = Vec::new();

    set_default_camera();
    loop {
        // move this stuff somewhere else
        let dt = get_frame_time();
        // speed for this frame (factoring in delta time and zoom)
        let frame_speed = (speed / zoom) * dt;
        if is_key_down(KeyCode::A) {
            pos_x -= frame_speed;
        }
        if is_key_down(KeyCode::D) {
            pos_x += frame_speed;
        }
        if is_key_down(KeyCode::W) {
            pos_y += frame_speed;
        }
        if is_key_down(KeyCode::S) {
            pos_y -= frame_speed;
        }
        if is_key_down(KeyCode::Q) || recording { // TEMP
            zoom -= 1.0 * zoom * 0.05 ;//* dt;
        }
        if is_key_down(KeyCode::E) {
            zoom += 1.0 * zoom * dt;
        }
        clear_background(WHITE);
        let iter_zoom_mod: f64 = 10.0;

        println!("Zoom:{}", zoom);
        max_iterations = clamp((1000 as f32 * zoom.log10() * 1.0) as i32, 400, 60000);
        // max_iterations = 10000;
        println!("Iter_max: {}", max_iterations);
        println!("X: {}, Y: {}", pos_x, pos_y);
        gl_use_material(&material);
        material.set_uniform("screen_size", (screen_width(), screen_height()));
        material.set_uniform("position", (pos_x, pos_y));
        material.set_uniform("zoom", zoom);
        material.set_uniform("max_iterations", max_iterations);
        material.set_uniform("time", get_time() as f32);

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

        // // this is very temporary
        // // and extermly jank
        // if is_key_pressed(KeyCode::Space) {
        //     let filename = format!("images/mandelbrot_{}.png", mandel_inc);
        //     get_screen_data().export_png(&filename);
        //     mandel_inc += 1;
        // }
        if recording {
            frames.push(get_screen_data());
        }

        if is_key_pressed(KeyCode::R){
            recording = !recording;
        }
        if is_key_pressed(KeyCode::Space) {
            frames.push(get_screen_data());
        }
        if is_key_pressed(KeyCode::Escape) || zoom < 0.9 && recording{
            break;
        }
        next_frame().await;
    }

    // Might need to change zooming to not factor in delta time to make it smooth
    for (i, img) in frames.iter().enumerate(){
        println!("\nSAVING IMAGE: {}\n This will cause a fair amount of lag, please wait.\n",i);
        let filename = format!("images/mandelbrot_{}.png", i);
        img.export_png(&filename);
    }
    println!("END ");
    // export images;
}

const DOUBLE_FRAGMENT_SHADER: &str = r#"
#version 430 core
precision highp float;

out vec4 fragColor;
// in highp vec4 gl_FragCoord;

uniform sampler2D Texture;
uniform vec2 screen_size;
uniform vec2 position;
uniform float zoom;
uniform int max_iterations;
uniform float time;

void main() {
    fragColor = vec4(0.0, 0.0, 0.0, 1.0);

    // vec2 scaledf = (2.0*gl_FragCoord.xy - screen_size.xy) / screen_size.y;

    // scaledf *= zoom;
    // scaledf.x += position.x;
    // scaledf.y += position.y;

    double scaledx = double((2.0*gl_FragCoord.x - screen_size.x) / screen_size.y);
    double scaledy = double((2.0*gl_FragCoord.y - screen_size.y) / screen_size.y);
    // I dont care about perf right now
    scaledx /= double(zoom);
    scaledy /= double(zoom);
    scaledx += double(position.x);
    scaledy += double(position.y);

    double df = double(1.0);
    double x = double(0.0);
    double y = double(0.0);
    double tmp = double(0.0);
    
    int i = 0;
    double limit = double(4.0);
    double two = double(2.0); // lf suffix doesn't work
    for(; i < max_iterations; i++){
        if(x*x + y*y > limit ){
            break;
        }
        
        tmp = x*x - y*y + scaledx;
        y = two * x * y + scaledy;
        x = tmp;
    }
 
    float fi = float(i);
    float i_norm = (float(i)/float(max_iterations)); 
 
    // need to figure out an actual good method of coloring 
    if(i < max_iterations){
        float zn = float(sqrt(x*x + y*y));
        float nu = log(log(zn) / log(2.0)) / log(2.0);
        float iteration_float = float(i) + 1.0 - nu;
        
        // Color interpolation
        float smooth_color = iteration_float / float(max_iterations);
        vec3 color = smooth_color * vec3(1.0, 1.0, 1.0); 
        fragColor = vec4(color*1.5, 1.0);
    }
}
"#;

const DOUBLE_VERTEX_SHADER: &str = "
#version 430 core
in vec3 position;
in vec2 texcoord;
in vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
}
";

const FLOAT_FRAGMENT_SHADER: &str = r#"
#version 100
precision highp float;

uniform sampler2D Texture;
uniform vec2 screen_size;
uniform vec2 position;
uniform float zoom;
uniform int max_iterations;
uniform float time;

void main() {
    gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);

    vec2 scaled = (2.0*gl_FragCoord.xy - screen_size.xy) / screen_size.y;
    
    scaled *= zoom;
    scaled.x += position.x;
    scaled.y += position.y;

    float x = 0.0;
    float y = 0.0;
    float tmp = 0.0;
    
    int i = 0;

    for(; i < max_iterations; i++){
        if(x*x + y*y > 4.0 ){
            break;
        }
        
        tmp = x*x - y*y + scaled.x;
        y = 2.0 * x * y + scaled.y;
        x = tmp;
    }

    float fi = float(i);
    float i_norm = (float(i)/float(max_iterations)); 

    // need to figure out an actual good method of coloring 
    if(i < max_iterations){
        gl_FragColor.r = i_norm * fi;
        gl_FragColor.g = i_norm * fi/2.0;
        gl_FragColor.b = i_norm * fi/2.0;
    }
    
}

"#;

const FLOAT_VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
}
";
