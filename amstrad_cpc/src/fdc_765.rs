// src/amstrad/fdc_765.rs

use crate::dsk::DiskImage;

#[derive(PartialEq)]
enum FdcState {
    Idle,
    ReadingCommand,
    ExecutionRead,   // The Z80 reads the sector data
    _ExecutionWrite, // The Z80 writes data to the sector
    Result,          // The Z80 reads the status bytes (ST0, ST1...)
}

pub struct Fdc765 {
    state: FdcState,

    // Main Status Register (MSR)
    msr: u8,

    // Buffers to store the current command and its paramenters
    command_buffer: [u8; 10],
    command_index: usize,
    command_length: usize,

    // Buffer for the sector data or the results
    data_buffer: Vec<u8>,
    data_index: usize,

    // Motor state (Port 0xFA7E)
    pub motor_on: bool,

    pub disk: Option<DiskImage>,

    interrupt_pending: bool,

    pub current_cylinder: u8,

    // Result of the last Seek/Recalibrate, reported by Sense Interrupt Status
    seek_drive: u8,
}

impl Default for Fdc765 {
    fn default() -> Self {
        Self::new()
    }
}

impl Fdc765 {
    pub fn new() -> Self {
        Self {
            state: FdcState::Idle,
            msr: 0x80, // Bit 7 (RQM) set : Ready to receive a command
            command_buffer: [0; 10],
            command_index: 0,
            command_length: 0,
            data_buffer: Vec::new(),
            data_index: 0,
            motor_on: false,
            disk: None,
            interrupt_pending: false,
            current_cylinder: 0,
            seek_drive: 0,
        }
    }

    pub fn read_msr(&self) -> u8 {
        self.msr
    }

    pub fn write_data(&mut self, value: u8) {
        if self.state == FdcState::Idle {
            self.command_buffer[0] = value;
            self.command_index = 1;
            self.command_length = self.get_command_length(value);

            if self.command_length == 1 {
                self.execute_command();
            } else {
                self.state = FdcState::ReadingCommand;
                self.update_msr();
            }
        } else if self.state == FdcState::ReadingCommand {
            self.command_buffer[self.command_index] = value;
            self.command_index += 1;

            if self.command_index >= self.command_length {
                self.execute_command();
            }
        }
    }

    pub fn read_data(&mut self) -> u8 {
        if (self.state == FdcState::ExecutionRead || self.state == FdcState::Result)
            && self.data_index < self.data_buffer.len()
        {
            let val = self.data_buffer[self.data_index];
            self.data_index += 1;

            if self.data_index >= self.data_buffer.len() {
                if self.state == FdcState::ExecutionRead {
                    self.transition_to_result();
                } else {
                    self.state = FdcState::Idle;
                    self.update_msr();
                }
            }
            val
        } else {
            0xFF // Error if the Z80 reads at the wrong moment
        }
    }

    fn update_msr(&mut self) {
        // RQM (Bit 7) : 1 = Ready, 0 = Busy
        // DIO (Bit 6) : 1 = FDC data to Z80 (reading), 0 = Z80 to FDC (Writing)
        // CB  (Bit 4) : 1 = Current command execution
        match self.state {
            FdcState::Idle => self.msr = 0x80,           // RQM=1, DIO=0, CB=0
            FdcState::ReadingCommand => self.msr = 0x90, // RQM=1, DIO=0, CB=1
            FdcState::ExecutionRead => self.msr = 0xF0,  // RQM=1, DIO=1, CB=1 (Data ready)
            FdcState::Result => self.msr = 0xD0,         // RQM=1, DIO=1, CB=1 (Result ready)
            _ => self.msr = 0x10,                        // Occupé
        }
    }

    fn execute_command(&mut self) {
        let cmd = self.command_buffer[0] & 0x1F;
        // println!("[FDC] Executing the command: 0x{:02X}", cmd);

        match cmd {
            0x03 => {
                // Specify (Configure timings) -> No result
                self.state = FdcState::Idle;
                self.update_msr();
            }
            0x07 | 0x0F => {
                // Recalibrate (0x07) or Seek (0x0F) -> Triggers a software "interrupt"
                self.seek_drive = self.command_buffer[1] & 0x03;
                if cmd == 0x07 {
                    self.current_cylinder = 0;
                } else {
                    // Seek: the new cylinder number is the 2nd parameter
                    self.current_cylinder = self.command_buffer[2];
                }
                self.interrupt_pending = true;
                self.state = FdcState::Idle;
                self.update_msr();
            }
            0x08 => {
                // Sense Interrupt Status (Check if a Seek/Recalibrate is over)
                if self.interrupt_pending {
                    self.interrupt_pending = false;
                    // ST0 = Seek End + drive number, PCN = present cylinder
                    self.data_buffer = vec![0x20 | self.seek_drive, self.current_cylinder];
                } else {
                    // Warning : if no pending interrupt, returns an error
                    self.data_buffer = vec![0x80]; // ST0 = Invalid Command
                }
                self.transition_to_result();
            }
            0x04 => {
                // Sense Drive Status (L'AMSDOS checks if the disk is ready)
                let drive = self.command_buffer[1] & 0x03;
                let ready_bit = 0x20; // 1 = Disque inserted and ready
                let track0_bit = if self.current_cylinder == 0 { 0x10 } else { 0 }; // 1 = The head is on the track 0
                self.data_buffer = vec![ready_bit | track0_bit | drive]; // Returns ST3
                self.transition_to_result();
            }
            0x06 => {
                let cylinder = self.command_buffer[2];
                let head = self.command_buffer[3];
                let start_sector = self.command_buffer[4];
                let size_code = self.command_buffer[5];
                let eot = self.command_buffer[6]; // End Of Track

                // println!(
                //     "[FDC] Read Data : Track={}, Head={}, Sectors from {:02X} to {:02X}",
                //     cylinder, head, start_sector, eot
                // );

                let mut data_to_send = Vec::new();
                let mut current_sector = start_sector;
                let mut error = false;

                if let Some(disk) = &self.disk {
                    if let Some(track) = disk
                        .tracks
                        .iter()
                        .find(|t| t.cylinder == cylinder && t.head == head)
                    {
                        // Get ALL the required sectors
                        while current_sector <= eot {
                            if let Some(sector) =
                                track.sectors.iter().find(|s| s.sector_id == current_sector)
                            {
                                data_to_send.extend_from_slice(&sector.data);
                                current_sector += 1;
                            } else {
                                error = true; // Missing sector on the disk
                                break;
                            }
                        }
                    } else {
                        error = true;
                    }
                } else {
                    error = true;
                }

                if !error && !data_to_send.is_empty() {
                    self.data_buffer = data_to_send;
                    self.state = FdcState::ExecutionRead;
                    self.data_index = 0;
                    self.update_msr();

                    self.command_buffer[4] = current_sector;
                } else {
                    // println!(
                    //     "[FDC] Hardware Error : Reading failure for sector {:02X}",
                    //     current_sector
                    // );
                    self.data_buffer = vec![
                        0x40, // ST0 : Abnormal Termination
                        0x04, // ST1 : No Data
                        0x00, // ST2
                        cylinder,
                        head,
                        current_sector,
                        size_code,
                    ];
                    self.transition_to_result();
                }
            }
            0x0A => {
                // Read ID : L'AMSDOS wants to know how is formatted this track
                // println!("[FDC] Read ID (Current cylinder: {})", self.current_cylinder);

                let mut found = false;

                if let Some(disk) = &self.disk {
                    // Look for the matching track in the image .DSK
                    if let Some(track) = disk
                        .tracks
                        .iter()
                        .find(|t| t.cylinder == self.current_cylinder)
                    {
                        // Take the full sector of this track
                        if let Some(sector) = track.sectors.first() {
                            self.data_buffer = vec![
                                0x00, // ST0 : Success
                                0x00, // ST1
                                0x00, // ST2
                                sector.cylinder,
                                sector.head,
                                sector.sector_id, // <-- That's the info the AMSDOS wants (ex: 0xC1)
                                sector.size_code,
                            ];
                            found = true;
                        }
                    }
                }

                if !found {
                    self.data_buffer = vec![
                        0x40, // ST0 : Abnormal Termination
                        0x01, // ST1 : Missing Address Mark (Secteur introuvable)
                        0x00, // ST2
                        self.current_cylinder,
                        0,
                        0,
                        0,
                    ];
                }

                self.transition_to_result();
            }
            _ => {
                // println!("[FDC] Command unknown or not supportes -> ST0=0x80");
                self.data_buffer = vec![0x80]; // ST0: Invalid Command
                self.transition_to_result();
            }
        }
    }

    fn transition_to_result(&mut self) {
        let cmd = self.command_buffer[0] & 0x1F;

        if cmd == 0x06 && self.state == FdcState::ExecutionRead {
            self.data_buffer = vec![
                0x00, // ST0 : Success
                0x00, // ST1
                0x00, // ST2
                self.command_buffer[2],
                self.command_buffer[3],
                self.command_buffer[4], // Now holds EOT + 1
                self.command_buffer[5],
            ];
        }

        self.data_index = 0;
        self.state = FdcState::Result;
        self.update_msr();
        // println!(
        //     "[FDC] Result phase ready ({} octets)",
        //     self.data_buffer.len()
        // );
    }

    fn get_command_length(&self, cmd: u8) -> usize {
        match cmd & 0x1F {
            0x03 => 3, // Specify : Cmd + 2 octets
            0x04 => 2, // Sense Device Status : Cmd + 1 octet
            0x07 => 2, // Recalibrate : Cmd + 1 octet
            0x08 => 1, // Sense Interrupt Status : Cmd seule
            0x0A => 2,
            0x0F => 3, // Seek : Cmd + 2 octets

            // Read/Write commands have 8 parameters
            0x06 | 0x0C | 0x05 | 0x09 => 9,

            _ => 1, // Non blocking for the state machine
        }
    }
}
