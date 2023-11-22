use specs::prelude::*;
use crate::renderer::QuadProps;
use crate::renderer::Renderer;
use crate::shader::ShaderProgram;

use super::components::*;
use super::resources::*;

pub struct Physics;

impl<'a> System<'a> for Physics{

    // System은 계속 돌아감.
    // 
    type SystemData = (
        Read<'a, DeltaTime>,            // 
        WriteStorage<'a, Position>,     // 공간에 쓴다.
        ReadStorage<'a, Velocity>       // 공간 전체(ex: 배열)를 읽는다.
    );

    fn run(&mut self, (dt, mut pos, vel): Self::SystemData){
        let dt = dt.delta.as_micros() as f32 / 1_000_000.0;
        for (pos, vel) in (&mut pos, &vel).join(){ // Generate Iterator 
            pos.0 += vel.0 * dt;
            pos.1 += vel.1 * dt;
        }
    }
}

pub struct Render;

impl<'a> System <'a> for Render{
    type SystemData = (
        Write<'a, Renderer>,
        WriteExpect<'a, ShaderProgram>,     // Expect for existence of its Resources, if not exist then PANIC!
        ReadStorage<'a, Position>);         // Since we draw something in RIGHT Position

        fn run(&mut self, (mut renderer, mut shader, pos): Self::SystemData) {
            renderer.begin_batch();
            for position in (&pos, ).join(){
                let pos = &*position.0;
                let tuple = (pos.0, pos.1, pos.2);

                renderer.submit_quad(QuadProps{
                    position: tuple,
                    size: (0.5, 0.5),
                    texture_id: 1,
                    texture_coords: (0.0, 0.0, 1.0, 1.0),
                });
            }
            shader.use_program();
            renderer.end_batch(&mut shader);
        }
}

pub struct ComputeDeltaTime;

impl<'a> System<'a> for ComputeDeltaTime{
    type SystemData = Write<'a, DeltaTime>;

    fn run(&mut self, mut dt: Self::SystemData){
        let now = now();

        dt.delta = now - dt.prev;
        dt.prev = now;
    }
}

pub struct Bounce;

impl<'a> System<'a> for Bounce {
    type SystemData = (
        ReadStorage<'a, Position>,
        WriteStorage<'a, Velocity>,
        // ReadStorage<'a, Bounce>
    );

    fn run(&mut self, (pos, mut vel): Self::SystemData){
        for (pos, vel) in (&pos, &mut vel).join(){
            if pos.0 < -1.0 || pos.0 > 1.0 {
                vel.0 *= -1.0;
            }
            if pos.1 < -1.0 || pos.1 > 1.0 {
                vel.1 *= -1.0;
            }
        }
    }
}