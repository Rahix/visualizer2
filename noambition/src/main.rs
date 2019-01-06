#[macro_use]
extern crate log;
#[macro_use]
extern crate glium;
extern crate nalgebra as na;

use glium::glutin;
use vis_core::analyzer;

macro_rules! shader_program {
    // {{{
    ($display:expr, $vert_file:expr, $frag_file:expr) => {{
        // Use this for debug
        #[cfg(debug_assertions)]
        {
            let vert_src = {
                use std::io::Read;
                let mut buf = String::new();
                let mut f = std::fs::File::open(format!("src/{}", $vert_file)).unwrap();
                f.read_to_string(&mut buf).unwrap();

                buf
            };

            let frag_src = {
                use ::std::io::Read;
                let mut buf = String::new();
                let mut f = std::fs::File::open(format!("src/{}", $frag_file)).unwrap();
                f.read_to_string(&mut buf).unwrap();

                buf
            };

            glium::Program::from_source($display, &vert_src, &frag_src, None).unwrap()
        }

        // Use this for release
        #[cfg(not(debug_assertions))]
        glium::Program::from_source(
            $display,
            include_str!($vert_file),
            include_str!($frag_file),
            None,
        )
        .unwrap()
    }};
} // }}}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 4],
    color_id: u16,
}

glium::implement_vertex!(Vertex, position, color_id);

#[derive(Debug, Clone)]
pub struct VisInfo {
    beat: u64,
    beat_volume: f32,
    volume: f32,
    analyzer: analyzer::FourierAnalyzer,
    spectrum: analyzer::Spectrum<Vec<f32>>,
}

fn main() {
    vis_core::default_config();
    vis_core::default_log();

    let mut frames = {
        // Analyzer {{{
        let mut beat = analyzer::BeatBuilder::new().build();
        let mut beat_num = 0;

        let analyzer = analyzer::FourierBuilder::new().plan();

        vis_core::Visualizer::new(
            VisInfo {
                beat: 0,
                beat_volume: 0.0,
                volume: 0.0,
                spectrum: analyzer::Spectrum::new(vec![0.0; analyzer.buckets()], 0.0, 1.0),
                analyzer,
            },
            move |info, samples| {
                if beat.detect(&samples) {
                    beat_num += 1;
                }
                info.beat = beat_num;
                info.beat_volume = beat.last_volume();
                info.volume = samples.volume(0.3);

                info.analyzer.analyze(&samples);
                info.spectrum.fill_from(&info.analyzer.average());

                info
            },
        )
        .async_analyzer(300)
        .frames()
        // }}}
    };

    // Config {{{
    // Window
    let window_width = vis_core::CONFIG.get_or("window.width", 1280);
    let window_height = vis_core::CONFIG.get_or("window.height", 720);
    let aspect = window_width as f32 / window_height as f32;

    // Columns
    let rows = vis_core::CONFIG.get_or("noa.cols.rows", 50);
    let cols = vis_core::CONFIG.get_or("noa.cols.num", 30);
    let nrow = cols * 4;
    let cols_per_note = vis_core::CONFIG.get_or("noa.cols.note_width", 6);
    let notes_num = cols * 2 / cols_per_note;
    let width = vis_core::CONFIG.get_or("noa.cols.width", 10.0);
    let depth = vis_core::CONFIG.get_or("noa.cols.depth", 30.0);
    let rowsize = depth / rows as f32;
    let mid_dist = vis_core::CONFIG.get_or("noa.cols.mid_dist", 0.1);
    let base_height = vis_core::CONFIG.get_or("noa.cols.base_height", 0.2);
    let base_speed = vis_core::CONFIG.get_or("noa.cols.speed", 0.1);
    let slowdown = vis_core::CONFIG.get_or("noa.cols.slowdown", 0.995);
    let speed_deviation = vis_core::CONFIG.get_or("noa.cols.speed_deviation", 50.0);
    let ampli_top = vis_core::CONFIG.get_or("noa.cols.amp_top", 0.7);
    let ampli_bottom = vis_core::CONFIG.get_or("noa.cols.amp_bottom", 0.2);

    let frame_time =
        std::time::Duration::from_micros(1000000 / vis_core::CONFIG.get_or("noa.fps", 30));

    // Colors
    let colors: Vec<[f32; 4]> = vis_core::CONFIG.get_or(
        "noa.cols.colors",
        vec![
            [1.0, 0.007443, 0.318893, 1.0],
            [0.915586, 0.704283, 0.214133, 1.0],
            [0.044844, 0.64629, 0.590788, 1.0],
            [0.130165, 0.022207, 0.27614, 1.0],
            [1.0, 0.007443, 0.318893, 1.0],
            [0.915586, 0.704283, 0.214133, 1.0],
            [0.044844, 0.64629, 0.590788, 1.0],
            [0.130165, 0.022207, 0.27614, 1.0],
            [1.0, 0.007443, 0.318893, 1.0],
            [0.915586, 0.704283, 0.214133, 1.0],
            [0.044844, 0.64629, 0.590788, 1.0],
            [0.130165, 0.022207, 0.27614, 1.0],
        ],
    );
    let note_roll_size = vis_core::CONFIG.get_or("noa.cols.note_roll", 20) as f32;

    // Camera
    let cam_height = vis_core::CONFIG.get_or("noa.camera.height", 1.0);
    let cam_look = vis_core::CONFIG.get_or("noa.camera.look_height", 0.8);

    // }}}

    // Window Initialization {{{
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(
            window_width as f64,
            window_height as f64,
        ))
        .with_maximized(true)
        .with_decorations(false)
        .with_fullscreen(Some(events_loop.get_primary_monitor()))
        .with_title("Visualizer2 - NoAmbition");

    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 1)))
        .with_gl_profile(glutin::GlProfile::Core)
        .with_multisampling(0);

    let display = glium::Display::new(window, context, &events_loop).unwrap();
    // }}}

    // Framebuffer Initialization {{{
    let tex1 = glium::texture::Texture2d::empty_with_format(
        &display,
        glium::texture::UncompressedFloatFormat::F32F32F32F32,
        glium::texture::MipmapsOption::NoMipmap,
        window_width,
        window_height,
    )
    .unwrap();
    let depth1 = glium::texture::DepthTexture2d::empty_with_format(
        &display,
        glium::texture::DepthFormat::F32,
        glium::texture::MipmapsOption::NoMipmap,
        window_width,
        window_height,
    )
    .unwrap();
    let mut framebuffer1 =
        glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&display, &tex1, &depth1).unwrap();

    let tex2 = glium::texture::Texture2d::empty_with_format(
        &display,
        glium::texture::UncompressedFloatFormat::F32F32F32F32,
        glium::texture::MipmapsOption::NoMipmap,
        window_width,
        window_height,
    )
    .unwrap();
    let depth2 = glium::texture::DepthTexture2d::empty_with_format(
        &display,
        glium::texture::DepthFormat::F32,
        glium::texture::MipmapsOption::NoMipmap,
        window_width,
        window_height,
    )
    .unwrap();
    let mut framebuffer2 =
        glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(&display, &tex2, &depth2).unwrap();
    // }}}

    // Shader Initialization {{{
    let prepass_program = shader_program!(&display, "shaders/prepass.vert", "shaders/prepass.frag");
    let background_program =
        shader_program!(&display, "shaders/pp.vert", "shaders/background.frag");
    // let fxaa_program = shader_program!(&display, "shaders/pp.vert", "shaders/fxaa.frag");
    // let bokeh_program = shader_program!(&display, "shaders/pp.vert", "shaders/bokeh.frag");
    // let color_program = shader_program!(&display, "shaders/pp.vert", "shaders/color.frag");
    // }}}

    // Buffers {{{

    // Quad {{{
    let quad_verts = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 4],
            texcoord: [f32; 2],
        }

        glium::implement_vertex!(Vertex, position, texcoord);

        glium::VertexBuffer::new(
            &display,
            &[
                Vertex {
                    position: [-1.0, -1.0, 0.0, 1.0],
                    texcoord: [0.0, 0.0],
                },
                Vertex {
                    position: [1.0, -1.0, 0.0, 1.0],
                    texcoord: [1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0, 1.0],
                    texcoord: [1.0, 1.0],
                },
                Vertex {
                    position: [-1.0, 1.0, 0.0, 1.0],
                    texcoord: [0.0, 1.0],
                },
            ],
        )
        .unwrap()
    };
    let quad_inds = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &[0u16, 1, 2, 0, 2, 3],
    )
    .unwrap();
    // }}}

    // Lines {{{
    let (mut lines_verts, mut lines_colors) = {
        let colsmax = (cols - 1) as f32 / width * 2.0;
        let rowsmax = (rows - 1) as f32 / depth;
        let h = base_height / 2.0;
        let mut v_buf = Vec::with_capacity(rows * cols * 4);

        for row in 0..rows {
            let y = row as f32 / rowsmax;
            // Left
            for col in 0..cols {
                let x = -(col as f32 / colsmax) - mid_dist;
                let cid = (cols - col - 1) / cols_per_note;
                v_buf.push(Vertex {
                    position: [x, y, -h, 1.0],
                    color_id: cid as u16,
                });
                v_buf.push(Vertex {
                    position: [x, y, h, 1.0],
                    color_id: cid as u16,
                });
            }

            // Right
            for col in 0..cols {
                let x = (col as f32 / colsmax) + mid_dist;
                let cid = col / cols_per_note + notes_num / 2;
                v_buf.push(Vertex {
                    position: [x, y, -h, 1.0],
                    color_id: cid as u16,
                });
                v_buf.push(Vertex {
                    position: [x, y, h, 1.0],
                    color_id: cid as u16,
                });
            }
        }

        let mut colors_buf = [[1.0, 0.0, 0.0, 1.0]; 32];
        for (buf, color) in colors_buf.iter_mut().zip(colors.iter()) {
            *buf = *color;
        }
        let lines_colors =
            glium::uniforms::UniformBuffer::persistent(&display, colors_buf).unwrap();

        (
            glium::VertexBuffer::persistent(&display, &v_buf).unwrap(),
            lines_colors,
        )
    };
    // }}}

    // Points {{{
    let points_colors = {
        let mut colors_buf = [[1.0, 0.0, 0.0, 1.0]; 32];
        for (buf, color) in colors_buf.iter_mut().zip(colors.iter()) {
            *buf = *color;
        }
        glium::uniforms::UniformBuffer::persistent(&display, colors_buf).unwrap()
    };
    // }}}

    // Lightning {{{
    // }}}

    // }}}

    let mut previous_time = 0.0;
    let mut previous_offset = 0.0;
    let mut rolling_volume = 0.0;
    let mut write_row = rows * 3 / 4;
    let mut last_beat = -100.0;

    let mut notes_spectrum = analyzer::Spectrum::new(vec![0.0; notes_num], 220.0, 660.0);
    let mut notes_rolling_buf = vec![0.0; notes_num];
    let mut row_buf = Vec::with_capacity(nrow);
    let mut row_spectrum = vec![0.0; cols];

    let mut beat_rolling = 0.0;
    let mut last_beat_num = 0;

    let mut maxima_buf = [(0.0, 0.0); 8];

    'main: for frame in frames.iter() {
        use glium::Surface;

        let start = std::time::Instant::now();
        let delta = frame.time - previous_time;
        trace!("Delta: {}s", delta);

        // Audio Info Retrieval {{{
        let (volume, maxima, notes_rolling_spectrum, base_volume) = frame.info(|info| {
            rolling_volume = info.volume.max(rolling_volume * slowdown);

            if info.beat != last_beat_num {
                last_beat = frame.time;
                last_beat_num = info.beat;
            }

            let notes_spectrum = info.spectrum.fill_spectrum(&mut notes_spectrum);

            for (n, s) in notes_rolling_buf.iter_mut().zip(notes_spectrum.iter()) {
                *n = (*n * (note_roll_size - 1.0) + s) / note_roll_size;
            }
            let notes_rolling_spectrum = vis_core::analyzer::Spectrum::new(
                &mut *notes_rolling_buf,
                notes_spectrum.lowest(),
                notes_spectrum.highest(),
            );

            let maxima = notes_rolling_spectrum.find_maxima(&mut maxima_buf);

            (
                info.volume,
                maxima,
                notes_rolling_spectrum,
                info.beat_volume,
            )
        });
        // }}}

        // GL Matrices {{{
        let view = na::Matrix4::look_at_rh(
            &na::Point3::new(0.0, -1.0, cam_height),
            &na::Point3::new(0.0, 10.0, cam_look),
            &na::Vector3::new(0.0, 0.0, 1.0),
        );

        let perspective =
            na::Matrix4::new_perspective(aspect, std::f32::consts::FRAC_PI_4, 0.001, 100.0);
        // }}}

        // Grid {{{
        let speed = base_speed + rolling_volume * speed_deviation;
        let offset = (previous_offset + delta * speed) % rowsize;
        let model_grid =
            na::Translation3::from(na::Vector3::new(0.0, -offset, 0.0)).to_homogeneous();

        // Color Notes {{{
        {
            let mut color_buf = lines_colors.map();
            for color in color_buf.iter_mut() {
                color[3] = 0.05;
            }
            for (f, _) in maxima.iter().take(4) {
                let note = notes_rolling_spectrum.freq_to_id(*f);
                color_buf[note][3] = 1.0;
            }
        }
        // }}}

        // Spectral Grid {{{
        if previous_offset > offset {
            let mut lines_buf = lines_verts.map();
            // We jumped right here
            // Save last rows y coordinate
            row_buf.clear();
            let last_row_offset = lines_buf.len() - nrow;
            for i in 0..nrow {
                row_buf.push(lines_buf[last_row_offset + i].position[1]);
            }
            // Copy y coordinate of each line to the next
            for i in (nrow..(lines_buf.len())).rev() {
                lines_buf[i].position[1] = lines_buf[i - nrow].position[1];
            }
            // Write saved info to first row (cycle rows)
            for i in 0..nrow {
                lines_buf[i].position[1] = row_buf[i];
            }

            // Write spectral information
            frame.info(|info| {
                let left = info
                    .analyzer
                    .left()
                    .slice(100.0, 800.0)
                    .fill_buckets(&mut row_spectrum[..]);
                let row_offset = nrow * write_row;
                let max = left.max() + 0.0001;
                for (i, v) in left.iter().enumerate() {
                    lines_buf[row_offset + i * 2 + 1].position[2] =
                        base_height / 2.0 + v / max * ampli_top;
                    lines_buf[row_offset + i * 2].position[2] =
                        -base_height / 2.0 - v / max * ampli_bottom;
                }

                let right = info
                    .analyzer
                    .right()
                    .slice(100.0, 800.0)
                    .fill_buckets(&mut row_spectrum[..]);
                let row_offset = nrow * write_row + cols * 2;
                let max = right.max() + 0.0001;
                for (i, v) in right.iter().enumerate() {
                    lines_buf[row_offset + i * 2 + 1].position[2] =
                        base_height / 2.0 + v / max * ampli_top;
                    lines_buf[row_offset + i * 2].position[2] =
                        -base_height / 2.0 - v / max * ampli_bottom;
                }
            });

            write_row = (write_row + 1) % rows;
        }
        // }}}
        // }}}

        // Drawing {{{
        let draw_params = glium::DrawParameters {
            line_width: Some(1.0),
            point_size: Some(2.0),
            blend: glium::Blend {
                color: glium::BlendingFunction::Addition {
                    source: glium::LinearBlendingFactor::SourceAlpha,
                    destination: glium::LinearBlendingFactor::OneMinusSourceAlpha,
                },
                alpha: glium::BlendingFunction::Addition {
                    source: glium::LinearBlendingFactor::One,
                    destination: glium::LinearBlendingFactor::One,
                },
                constant_value: (1.0, 1.0, 1.0, 1.0),
            },
            ..Default::default()
        };

        framebuffer1.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        framebuffer2.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);

        let (ref mut fa, ref mut fb) = (&mut framebuffer1, &mut framebuffer2);

        // Lines {{{
        let uniforms = uniform! {
            perspective_matrix: Into::<[[f32; 4]; 4]>::into(perspective),
            view_matrix: Into::<[[f32; 4]; 4]>::into(view),
            model_matrix: Into::<[[f32; 4]; 4]>::into(model_grid),
            Colors: &lines_colors,
            volume: rolling_volume,
        };
        fa.draw(
            &lines_verts,
            &glium::index::NoIndices(glium::index::PrimitiveType::LinesList),
            &prepass_program,
            &uniforms,
            &draw_params,
        )
        .unwrap();
        // }}}

        // Points {{{
        let uniforms = uniform! {
            perspective_matrix: Into::<[[f32; 4]; 4]>::into(perspective),
            view_matrix: Into::<[[f32; 4]; 4]>::into(view),
            model_matrix: Into::<[[f32; 4]; 4]>::into(model_grid),
            Colors: &points_colors,
            volume: rolling_volume,
        };
        fa.draw(
            &lines_verts,
            &glium::index::NoIndices(glium::index::PrimitiveType::Points),
            &prepass_program,
            &uniforms,
            &draw_params,
        )
        .unwrap();
        // }}}

        // Post-Processing {{{
        beat_rolling = (beat_rolling * 0.95f32).max(base_volume);

        let (fa, fb) = (fb, fa);
        let ua = uniform! {
            previous: tex1.sampled().wrap_function(glium::uniforms::SamplerWrapFunction::Mirror),
            aspect: aspect,
            time: frame.time,
            volume: volume,
            last_beat: frame.time - last_beat,
            beat: beat_rolling,
        };
        let ub = uniform! {
            previous: tex2.sampled().wrap_function(glium::uniforms::SamplerWrapFunction::Mirror),
            aspect: aspect,
            time: frame.time,
            volume: volume,
            last_beat: frame.time - last_beat,
            beat: beat_rolling,
        };

        fa.draw(
            &quad_verts,
            &quad_inds,
            &background_program,
            &ua,
            &draw_params,
        )
        .unwrap();
        #[allow(unused_variables)]
        let (fa, ua, fb, ub) = (fb, ub, fa, ua);
        // }}}

        // Finalizing / Draw to screen {{{
        let target = display.draw();
        let dims = target.get_dimensions();
        target.blit_from_simple_framebuffer(
            &fb,
            &glium::Rect {
                left: 0,
                bottom: 0,
                width: window_width,
                height: window_height,
            },
            &glium::BlitTarget {
                left: 0,
                bottom: 0,
                width: dims.0 as i32,
                height: dims.1 as i32,
            },
            glium::uniforms::MagnifySamplerFilter::Linear,
        );
        target.finish().unwrap();
        // }}}
        // }}}

        // Events {{{
        let mut closed = false;
        events_loop.poll_events(|ev| match ev {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::CloseRequested => closed = true,
                glutin::WindowEvent::KeyboardInput {
                    input:
                        glutin::KeyboardInput {
                            virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => closed = true,
                _ => (),
            },
            _ => (),
        });
        if closed {
            break 'main;
        }
        // }}}

        previous_time = frame.time;
        previous_offset = offset;

        let end = std::time::Instant::now();
        let dur = end - start;
        if dur < frame_time {
            let sleep = frame_time - dur;
            std::thread::sleep(sleep);
        }
    }
}
