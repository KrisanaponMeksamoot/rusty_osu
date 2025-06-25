extern crate bytemuck;
extern crate gl;
extern crate glfw;

pub mod graphics;
pub mod resource;

use std::{collections::VecDeque, path::Path};

use glfw::{Action, Context, Key};

use crate::{graphics::circle, resource::osufile::HitObjectType};

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    let (mut window, events) = glfw
        .create_window(640, 480, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.make_current();

    gl::load_with(|s| glfw.get_proc_address_raw(s));

    let vertex_shader =
        graphics::Shader::new(gl::VERTEX_SHADER).expect("Couldn't make a vertex shader");
    vertex_shader
        .init(
            r#"#version 330 core
  layout (location = 0) in vec2 pos;
  layout (location = 1) in vec4 vcol;
  layout (location = 2) in float vrat;
  layout (location = 3) in float vca;
  uniform mat4 u_mat;

  out vec4 col;
  out float rat;
  out float ca;
  void main() {
    col = vcol;
    rat = vrat;
    ca = vca;
    gl_Position = u_mat * vec4(pos, 0.0, 1.0);
  }
"#,
        )
        .unwrap();

    let fragment_shader =
        graphics::Shader::new(gl::FRAGMENT_SHADER).expect("Couldn't make a fragment shader");
    fragment_shader
        .init(
            r#"#version 330 core
  in vec4 col;
  in float rat;
  in float ca;
  uniform vec3 u_color;
  out vec4 final_color;

  void main() {
    final_color = vec4(u_color, ca) * rat + col * (1 - rat);
  }
"#,
        )
        .unwrap();

    let shader_program = graphics::ShaderProgram::new(&vertex_shader, &fragment_shader).unwrap();

    let vao = graphics::VertexArray::new().expect("Couldn't make a VAO");
    vao.bind();
    let vbo = graphics::Buffer::new().expect("Couldn't make a VBO");
    vbo.bind(graphics::BufferType::Array);

    let buf_cir = graphics::circle::CircleBuffer::new();

    buf_cir.vertices_buffer_data();

    let ebo = graphics::Buffer::new().expect("Couldn't make the element buffer.");
    ebo.bind(graphics::BufferType::ElementArray);
    buf_cir.indeces_buffer_data();

    window.glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    shader_program.use_program();
    let u_mvp = shader_program.get_uniform_location(b"u_mat\0".as_ptr() as *const _);
    let u_col = shader_program.get_uniform_location(b"u_color\0".as_ptr() as *const _);
    
    let p = Path::new("test_res/UPLIFT SPICE - Omega Rhythm/UPLIFT SPICE - Omega Rhythm (Jemmmmy) [lightr's Insane].osu");
    let bm: resource::osufile::OsuFile = resource::osufile::parse_osu(&p);
    let bmn = bm.metadata.title;
    let bma = bm.metadata.artist;
    println!("Title: {bma} - {bmn}");
    let ar = bm.difficulty.approach_rate;
    let cs = bm.difficulty.circle_size;
    let hr = bm.difficulty.hp_drain_rate;
    let od = bm.difficulty.overall_difficulty;
    println!("AR: {ar}, CS: {cs}, HR: {hr}, OD: {od}");
    
    let scale: f32 = 54.4 - 4.48 * cs;

    let p_aud = p.parent().unwrap().join(bm.general.audio_filename).as_path().to_owned();
    let p_aud_s = p_aud.to_str().unwrap();
    println!("Audio file: {p_aud_s}");

    let (mut player, _handle) = resource::audio::AudioPlayer::new_async(&p_aud);

    player.start();
    player.play();

    let mut queue = VecDeque::new();
    let mut i = 0;
    let mut cbi = 0;

    unsafe {
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    while !window.should_close() {
        let elapsed_ms = player.get_time_ms() as i32 - bm.general.audio_lead_in;
        let preempt = 500;
        let fado = 100;

        while i < bm.hit_objects.len() && bm.hit_objects[i].time <= elapsed_ms + preempt {
            let ho = &bm.hit_objects[i];
            if ho.obj_type.contains(HitObjectType::NEW_COMBO) {
                cbi += 1;
                if cbi >= bm.colours.combos.len() {
                    cbi = 0
                }
            }
            // if ho.obj_type.contains(HitObjectType::CIRCLE) {
                queue.push_front((ho, cbi));
            // }
            i += 1;
        }

        while match queue.back() {Some(ho) => ho.0.time + fado < elapsed_ms, None => false} {
            queue.pop_back();
        }

        if queue.len() == 0 && i >= bm.hit_objects.len() {
            println!("Song ended!");
            break;
        }

        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            
            for ho in queue.iter() {
                let x = ho.0.x as f32 / 320.0 - 1.0;
                let y = 1.0 - ho.0.y as f32 / 240.0;
                circle::calc_mat(u_mvp, x, y, scale);
                let col = if ho.0.time < elapsed_ms {
                    (255, 255, 255)
                } else {
                    bm.colours.combos[ho.1]
                };
                let col = [col.0 as f32 / 255.0, col.1 as f32 / 255.0, col.2 as f32 / 255.0];
                gl::Uniform3fv(u_col, 1, col.as_ptr());
                buf_cir.draw();
            }
        }

        window.swap_buffers();
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
                _ => {}
            }
        }
    }
}
