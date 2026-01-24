use std::collections::HashMap;
use crate::parser::WasmaConfig;


#[derive(Debug, Clone)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub z_index: u8,
    pub active: bool,
}

pub struct WindowMultitary {
    config: WasmaConfig,
    pub viewports: HashMap<u8, Viewport>, // stream_id -> Viewport
    screen_width: u32,
    screen_height: u32,
}

impl WindowMultitary {
    pub fn new(config: WasmaConfig, screen_width: u32, screen_height: u32) -> Self {
        let mut multitary = Self {
            config,
            viewports: HashMap::new(),
            screen_width,
            screen_height,
        };
        multitary.calculate_layouts();
        multitary
    }

    pub fn calculate_layouts(&mut self) {
        let proto_count = self.config.uri_handling.protocols.len();
        let is_singularity = self.config.uri_handling.singularity_instances;
        let is_multi = self.config.uri_handling.multi_instances;

        if is_singularity && !is_multi {
            // Singularity: Tek pencere, tam ekran
            self.viewports.insert(0, Viewport {
                x: 0, y: 0,
                width: self.screen_width,
                height: self.screen_height,
                z_index: 1,
                active: true,
            });
        } else if is_multi {
            // Multi-Instance: Ekranı protokol sayısına göre dikey böl (Tiling)
            let section_height = self.screen_height / (proto_count as u32);
            for i in 0..proto_count {
                self.viewports.insert(i as u8, Viewport {
                    x: 0,
                    y: (i as i32) * (section_height as i32),
                    width: self.screen_width,
                    height: section_height,
                    z_index: 1,
                    active: true,
                });
            }
        }
    }

    pub fn get_viewport_for_stream(&self, stream_id: u8) -> Option<&Viewport> {
        self.viewports.get(&stream_id)
    }

    pub fn update_resolution(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
        self.calculate_layouts();
    }

    pub fn handle_input_focus(&self, x: i32, y: i32) -> Option<u8> {
        // Fare koordinatına göre hangi protokolün (stream_id) aktif olduğunu bulur
        for (id, vp) in &self.viewports {
            if x >= vp.x && x <= (vp.x + vp.width as i32) &&
               y >= vp.y && y <= (vp.y + vp.height as i32) {
                return Some(*id);
            }
        }
        None
    }
}
