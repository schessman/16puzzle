use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement, MouseEvent};
use wgpu::util::DeviceExt;
use rand::seq::SliceRandom;
use std::cell::RefCell;
use std::rc::Rc;
use bytemuck;

const GRID_SIZE: usize = 4;
const NUM_TILES: usize = GRID_SIZE * GRID_SIZE;
const EMPTY_TILE: u8 = 0;
const SHUFFLE_MOVES: usize = 100;

#[wasm_bindgen]
pub struct Puzzle {
    state: Rc<RefCell<PuzzleState>>,
}

struct RenderState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    _config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
}

struct PuzzleState {
    render_state: Option<RenderState>,
    tiles: [u8; NUM_TILES],
    empty_pos: (usize, usize),
    moves: u32,
    canvas: HtmlCanvasElement,
    animation_frame_closure: Option<Closure<dyn FnMut()>>,
}

#[wasm_bindgen]
impl Puzzle {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<Puzzle, JsValue> {
        console_error_panic_hook::set_once();

        let document = window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("Canvas element not found"))?
            .dyn_into::<HtmlCanvasElement>()?;

        let state = Rc::new(RefCell::new(PuzzleState {
            render_state: None,
            tiles: [0; NUM_TILES],
            empty_pos: (0, 0),
            moves: 0,
            canvas,
            animation_frame_closure: None,
        }));

        let puzzle = Puzzle { state: state.clone() };

        wasm_bindgen_futures::spawn_local(async move {
            state.borrow_mut().init(state.clone()).await.unwrap();
        });

        Ok(puzzle)
    }

    pub fn restart(&self) {
        self.state.borrow_mut().restart();
    }
}

impl PuzzleState {
    async fn init(self: &mut Self, state_rc: Rc<RefCell<PuzzleState>>) -> Result<(), JsValue> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(self.canvas.clone())).map_err(|e| JsValue::from_str(&format!("Failed to create surface: {}", e)))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| JsValue::from_str("Failed to find an appropriate adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to create device: {}", e)))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self.canvas.width(),
            height: self.canvas.height(),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 2 * 4, // 2 floats * 4 bytes/float
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        self.render_state = Some(RenderState {
            surface,
            device,
            queue,
            _config: config,
            render_pipeline,
        });

        self.restart();

        // Add click listener
        let state_clone = state_rc.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            state_clone.borrow_mut().click(event.offset_x() as u32, event.offset_y() as u32);
        }) as Box<dyn FnMut(_)>);
        self.canvas.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();

        // Start render loop
        let state_clone_anim = state_rc.clone();
        let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            state_clone_anim.borrow().render();
            let window = web_sys::window().unwrap();
            window.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
        }) as Box<dyn FnMut()>));

        self.animation_frame_closure = Some(g.borrow_mut().take().unwrap());
        let window = web_sys::window().unwrap();
        window.request_animation_frame(self.animation_frame_closure.as_ref().unwrap().as_ref().unchecked_ref())?;

        Ok(())
    }

    fn restart(&mut self) {
        self.moves = 0;
        self.update_move_count();
        for i in 0..NUM_TILES - 1 {
            self.tiles[i] = (i + 1) as u8;
        }
        self.tiles[NUM_TILES - 1] = EMPTY_TILE;
        self.empty_pos = (GRID_SIZE - 1, GRID_SIZE - 1);
        self.shuffle();
        self.render();
    }

    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        for _ in 0..SHUFFLE_MOVES {
            let (ex, ey) = self.empty_pos;
            let mut neighbors = Vec::new();
            if ex > 0 { neighbors.push((ex - 1, ey)); }
            if ex < GRID_SIZE - 1 { neighbors.push((ex + 1, ey)); }
            if ey > 0 { neighbors.push((ex, ey - 1)); }
            if ey < GRID_SIZE - 1 { neighbors.push((ex, ey + 1)); }

            if let Some(&(nx, ny)) = neighbors.choose(&mut rng) {
                let empty_idx = ey * GRID_SIZE + ex;
                let neighbor_idx = ny * GRID_SIZE + nx;
                self.tiles.swap(empty_idx, neighbor_idx);
                self.empty_pos = (nx, ny);
            }
        }
    }

    fn click(&mut self, x: u32, y: u32) {
        let (tile_w, tile_h) = (self.canvas.width() / GRID_SIZE as u32, self.canvas.height() / GRID_SIZE as u32);
        let (cx, cy) = ( (x / tile_w) as usize, (y / tile_h) as usize );

        if cx >= GRID_SIZE || cy >= GRID_SIZE { return; }

        let (ex, ey) = self.empty_pos;
        let is_adjacent = (cx == ex && (cy as i32 - ey as i32).abs() == 1) || (cy == ey && (cx as i32 - ex as i32).abs() == 1);

        if is_adjacent {
            let click_idx = cy * GRID_SIZE + cx;
            let empty_idx = ey * GRID_SIZE + ex;
            self.tiles.swap(click_idx, empty_idx);
            self.empty_pos = (cx, cy);
            self.moves += 1;
            self.update_move_count();
            self.render();
        }
    }

    fn update_move_count(&self) {
        let document = window().unwrap().document().unwrap();
        if let Some(el) = document.get_element_by_id("move-count") {
            el.set_text_content(Some(&self.moves.to_string()));
        }
    }

    fn render(&self) {
        let rs = if let Some(rs) = &self.render_state { rs } else { return };

        let output = match rs.surface.get_current_texture() {
            Ok(output) => output,
            Err(wgpu::SurfaceError::OutOfMemory) => {
                // Skip rendering this frame.
                return;
            }
            Err(e) => {
                web_sys::console::error_1(&JsValue::from_str(&format!("Surface error: {:?}", e)));
                return;
            }
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = rs.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut vertices = Vec::new();
        let tile_size_ndc = 2.0 / GRID_SIZE as f32; // Size of a tile in normalized device coordinates

        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                let tile_index = y * GRID_SIZE + x;
                if self.tiles[tile_index] != EMPTY_TILE {
                    // Top-left corner of the grid in NDC is (-1.0, 1.0)
                    let top_left_x = -1.0 + x as f32 * tile_size_ndc;
                    let top_left_y = 1.0 - y as f32 * tile_size_ndc;
                    let bottom_right_x = top_left_x + tile_size_ndc;
                    let bottom_right_y = top_left_y - tile_size_ndc;

                    // Triangle 1
                    vertices.extend_from_slice(&[top_left_x, top_left_y]);
                    vertices.extend_from_slice(&[bottom_right_x, top_left_y]);
                    vertices.extend_from_slice(&[bottom_right_x, bottom_right_y]);

                    // Triangle 2
                    vertices.extend_from_slice(&[top_left_x, top_left_y]);
                    vertices.extend_from_slice(&[bottom_right_x, bottom_right_y]);
                    vertices.extend_from_slice(&[top_left_x, bottom_right_y]);
                }
            }
        }

        let vertex_buffer = rs.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.9, g: 0.9, b: 0.9, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&rs.render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..vertices.len() as u32 / 2, 0..1);
        }

        rs.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}