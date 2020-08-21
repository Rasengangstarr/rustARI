extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::env;

use std::time::Instant;

use log::error;
use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;


mod rom_read;
mod mem_load;
mod atari;

const TARGET_FPS: u64 = 30;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
   box_x: i16,
   box_y: i16,
   velocity_x: i16,
   velocity_y: i16,
}

impl World {
   /// Create a new `World` instance that can draw a moving box.
   fn new() -> Self {
       Self {
           box_x: 24,
           box_y: 16,
           velocity_x: 1,
           velocity_y: 1,
       }
   }

   /// Update the `World` internal state; bounce the box around the screen.
   fn update(&mut self) {
       if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH as i16 {
           self.velocity_x *= -1;
       }
       if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT as i16 {
           self.velocity_y *= -1;
       }

       self.box_x += self.velocity_x;
       self.box_y += self.velocity_y;
   }

   /// Draw the `World` state to the frame buffer.
   ///
   /// Assumes the default texture format: [`wgpu::TextureFormat::Rgba8UnormSrgb`]
   fn draw(&self, frame: &mut [u8], atari: &mut atari::Atari, timer: &mut usize) {
      
       for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
           
           let x = (i % WIDTH as usize) as i16;
           let y = (i / WIDTH as usize) as i16;

           let mut rgba = if atari.read_mem(0x09) == 0x30 {
               [0xff, 0x00, 0x00, 0xff]
           } else {
               [0x00, 0x00, 0x00, 0xff]
           };

           if x == 68 || y == 37 || y == 229 {
              rgba = [0x00, 0xff, 0x00, 0xff];
           }
           
           if *timer > atari.cycles * 3 {
               atari.execute_step();
           }
           *timer = *timer + 1;

           pixel.copy_from_slice(&rgba);
       }
   }
}


fn main() {

   let args: Vec<String> = env::args().collect();

   assert_eq!(args.len(), 2, "wrong number of arguments provided! provide a filename only");
   let filename = &args[1];

   //println!("reading file: {}", filename);

   let rom = rom_read::get_file_as_byte_vec(filename);
   let atari : atari::Atari = atari::Atari::new(mem_load::write_rom_to_mem(rom));
   main_loop(atari).unwrap();
}

const WIDTH: u32 = 228;
const HEIGHT: u32 = 262;
const BOX_SIZE: i16 = 64;

fn main_loop(mut atari : atari::Atari) -> Result<(), Error> {
   env_logger::init();
   let event_loop = EventLoop::new();
   let mut input = WinitInputHelper::new();
   let mut timer = 0;
   let window = {
       let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
       WindowBuilder::new()
           .with_title("RUSTARI")
           .with_inner_size(size)
           .with_min_inner_size(size)
           .build(&event_loop)
           .unwrap()
   };

   let mut pixels = {
       let window_size = window.inner_size();
       let surface = Surface::create(&window);
       let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, surface);
       Pixels::new(WIDTH, HEIGHT, surface_texture)?
   };
   let mut world = World::new();


   event_loop.run(move |event, _, control_flow| {
       let start_time = Instant::now();
       // Draw the current frame
       if let Event::RedrawRequested(_) = event {
         
           world.draw(pixels.get_frame(), &mut atari, &mut timer);
           if pixels
               .render()
               .map_err(|e| error!("pixels.render() failed: {}", e))
               .is_err()
           {
               *control_flow = ControlFlow::Exit;
               return;
           }
           let elapsed_time = Instant::now().duration_since(start_time).as_millis() as u64;
 
           let wait_millis = match 1000 / TARGET_FPS >= elapsed_time {
               true => 1000 / TARGET_FPS - elapsed_time,
               false => 0
           };
           let new_inst = start_time + std::time::Duration::from_millis(wait_millis);
           *control_flow = ControlFlow::WaitUntil(new_inst);
       }

       // Handle input events
       if input.update(event) {
           // Close events
           if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
               *control_flow = ControlFlow::Exit;
               return;
           }

           // Resize the window
           if let Some(size) = input.window_resized() {
               pixels.resize(size.width, size.height);
           }

           // Update internal state and request a redraw
           world.update();
           window.request_redraw();
       }

       
   });
}