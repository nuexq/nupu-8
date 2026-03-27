use cpu::Port;

pub type Framebuffer = [[u8; 128]; 8];

pub struct NupuDisplay {
    pub vram: Framebuffer, // 128 columns x 8 pages
    pub x_ptr: u8,
    pub page_ptr: u8,
}

impl NupuDisplay {
    pub fn new() -> Self {
        Self {
            vram: [[0; 128]; 8],
            x_ptr: 0,
            page_ptr: 0,
        }
    }

    pub fn update(&mut self, cpu_ports: &mut [Port; 256]) {
        // Port for X-Coord
        if cpu_ports[0x10].dirty {
            self.x_ptr = cpu_ports[0x10].data % 128;
            cpu_ports[0x10].dirty = false;
        }
        // Port for Page-Coord
        if cpu_ports[0x11].dirty {
            self.page_ptr = cpu_ports[0x11].data % 8;
            cpu_ports[0x11].dirty = false;
        }
        // Port for Data
        if cpu_ports[0x12].dirty {
            self.vram[self.page_ptr as usize][self.x_ptr as usize] = cpu_ports[0x12].data;

            if self.x_ptr < 127 {
                self.x_ptr += 1;
            } else {
                self.x_ptr = 0;
            }

            cpu_ports[0x12].dirty = false;
        }
    }
}
