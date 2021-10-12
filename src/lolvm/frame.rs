#[derive(Default, Debug, Clone, Copy)]
pub struct CallFrame {
    pub ret_ip: usize,
    pub ip: usize,
    pub st_offset: usize,
}
