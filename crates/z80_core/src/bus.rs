pub trait SystemBus {
    fn read_memory(&mut self, addr: u16) -> u8;

    fn write_memory(&mut self, addr: u16, data: u8);

    fn read_io(&mut self, port: u16) -> u8;

    fn write_io(&mut self, port: u16, data: u8);
}
