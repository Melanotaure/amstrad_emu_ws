pub struct Sector {
    pub cylinder: u8,
    pub head: u8,
    pub sector_id: u8, // On CPC, generally from 0xC1 to 0xC9
    pub size_code: u8, // N (2 = 512 octets)
    pub data: Vec<u8>,
}

pub struct Track {
    pub cylinder: u8,
    pub head: u8,
    pub sectors: Vec<Sector>,
}

pub struct DiskImage {
    pub tracks: Vec<Track>,
}

impl DiskImage {
    /// Loads a standard file .DSK in memory
    pub fn load(path: &str) -> std::io::Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut buffer = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut buffer)?;

        if buffer.len() < 0x100 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Fichier trop petit",
            ));
        }

        // Detection of the extended format (Extended DSK)
        let signature = String::from_utf8_lossy(&buffer[0..16]);
        let is_extended = signature.contains("EXTENDED");

        let num_tracks = buffer[0x30] as usize;
        let num_heads = buffer[0x31] as usize;

        let global_track_size = u16::from_le_bytes([buffer[0x32], buffer[0x33]]) as usize;

        let mut tracks = Vec::new();
        let mut offset = 0x100; // Skip the global header

        for i in 0..(num_tracks * num_heads) {
            // Track information block (0x18 bytes before the sector table)
            if offset + 0x18 > buffer.len() {
                break;
            }

            // In EDSK, the track size table starts at 0x34 (1 byte = 256 bytes)
            let current_track_size = if is_extended {
                buffer.get(0x34 + i).copied().unwrap_or(0) as usize * 256
            } else {
                global_track_size
            };

            if current_track_size == 0 {
                continue; // Non formatted track or empty
            }

            let track_cylinder = buffer[offset + 0x10];
            let track_head = buffer[offset + 0x11];
            let num_sectors = buffer[offset + 0x15] as usize;

            let mut sectors = Vec::new();
            let mut sector_data_offset = offset + 0x100; // The data follow the track header

            for s in 0..num_sectors {
                let info_offset = offset + 0x18 + (s * 8);
                if info_offset + 8 > buffer.len() {
                    break;
                }

                let c = buffer[info_offset];
                let h = buffer[info_offset + 1];
                let r = buffer[info_offset + 2];
                let n = buffer[info_offset + 3];

                // For protected games (EDSK), the real sector size is more important than the code 'N'
                // It is stored in the bytes 6 and 7 of the sector information block.
                let actual_size = if is_extended {
                    u16::from_le_bytes([buffer[info_offset + 6], buffer[info_offset + 7]]) as usize
                } else {
                    128usize << n.min(8) // Default hardware formula (N=2 -> 512 bytes)
                };

                let data_end = sector_data_offset + actual_size;
                let data = if data_end <= buffer.len() {
                    buffer[sector_data_offset..data_end].to_vec()
                } else {
                    vec![0xE5; actual_size]
                };

                sectors.push(Sector {
                    cylinder: c,
                    head: h,
                    sector_id: r,
                    size_code: n,
                    data,
                });
                sector_data_offset += actual_size;
            }

            tracks.push(Track {
                cylinder: track_cylinder,
                head: track_head,
                sectors,
            });
            offset += current_track_size; // Next track
        }

        println!(
            "[DSK] Image loaded : '{}' ({} tracks, extended={})",
            path,
            tracks.len(),
            is_extended
        );
        Ok(Self { tracks })
    }

    /// Get one specific sector data (C, H, R)
    pub fn get_sector_data(&self, cylinder: u8, head: u8, sector_id: u8) -> Option<&[u8]> {
        for track in &self.tracks {
            // On the Amstrad, the head is generally 0
            if track.cylinder == cylinder && track.head == head {
                for sector in &track.sectors {
                    if sector.sector_id == sector_id {
                        return Some(&sector.data);
                    }
                }
            }
        }
        None // Sector not found
    }
}
