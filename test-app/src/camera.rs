use cgmath::{Matrix3, Rad, Vector3, Point3};
use glutin::VirtualKeyCode;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::Instant;

pub struct CameraControl {
    pressed_keys: HashMap<VirtualKeyCode, Instant>,
    pub pos: Point3<f32>,
    pub look: Vector3<f32>,
    pub up: Vector3<f32>,
}

impl CameraControl {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashMap::new(),
            pos: Point3::new(0.0, 0.0, 0.0),
            look: Vector3::unit_x(),
            up: Vector3::unit_y(),
        }
    }

    pub fn key_down(&mut self, scancode: VirtualKeyCode) {
        let time = Instant::now();
        let new_key = if let Entry::Vacant(entry) = self.pressed_keys.entry(scancode) {
            entry.insert(time);
            true
        } else {
            false
        };
        if new_key {
            self.run(time);
        }
    }

    pub fn key_up(&mut self, scancode: VirtualKeyCode) {
        let time = Instant::now();
        if self.pressed_keys.contains_key(&scancode) {
            self.run(time);
            self.pressed_keys.remove(&scancode);
        }
    }

    pub fn step(&mut self) {
        let time = Instant::now();
        self.run(time);
    }

    pub fn right(&self) -> Vector3<f32> {
        Vector3::cross(self.look, self.up)
    }

    fn run(&mut self, now: Instant) {
        use cgmath::InnerSpace;
        let move_speed = 8.0;
        let turn_speed = 0.5;
        let roll_speed = 1.0;
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::W) {
            self.pos += self.look * (move_speed * dt);
        }
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::S) {
            self.pos -= self.look * (move_speed * dt);
        }
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::D) {
            self.pos += self.right() * (move_speed * dt);
        }
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::A) {
            self.pos -= self.right() * (move_speed * dt);
        }
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::Space) {
            self.pos += self.up * (move_speed * dt);
        }
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::Z) {
            self.pos -= self.up * (move_speed * dt);
        }
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::I) {
            self.look = Matrix3::from_axis_angle(self.right(), Rad(turn_speed * dt)) * self.look;
        }
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::K) {
            self.look = Matrix3::from_axis_angle(self.right(), Rad(-turn_speed * dt)) * self.look;
        }
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::J) {
            self.look = Matrix3::from_axis_angle(self.up, Rad(turn_speed * dt)) * self.look;
        }
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::L) {
            self.look = Matrix3::from_axis_angle(self.up, Rad(-turn_speed * dt)) * self.look;
        }
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::O) {
            self.up = Matrix3::from_axis_angle(self.look, Rad(roll_speed * dt)) * self.up;
        }
        if let Some(dt) = self.is_pressed(now, VirtualKeyCode::U) {
            self.up = Matrix3::from_axis_angle(self.look, Rad(-roll_speed * dt)) * self.up;
        }
        self.look = self.look.normalize();
        self.up = Vector3::cross(Vector3::cross(self.look, self.up), self.look).normalize();
        for value in self.pressed_keys.values_mut() {
            *value = now;
        }
    }

    fn is_pressed(&self, now: Instant, scancode: VirtualKeyCode) -> Option<f32> {
        if let Some(&old) = self.pressed_keys.get(&scancode) {
            let dt = now.duration_since(old);
            let flt = dt.as_secs() as f32 + dt.subsec_nanos() as f32 * 1e-9;
            Some(flt)
        } else {
            None
        }
    }
}
