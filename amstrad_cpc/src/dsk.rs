pub struct Sector {
    pub cylinder: u8,
    pub head: u8,
    pub sector_id: u8, // Sur CPC, généralement de 0xC1 à 0xC9
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
    /// Charge un fichier .DSK standard en mémoire
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

        // Détection du format étendu (Extended DSK)
        let signature = String::from_utf8_lossy(&buffer[0..16]);
        let is_extended = signature.contains("EXTENDED");

        let num_tracks = buffer[0x30] as usize;
        let num_heads = buffer[0x31] as usize;

        let global_track_size = u16::from_le_bytes([buffer[0x32], buffer[0x33]]) as usize;

        let mut tracks = Vec::new();
        let mut offset = 0x100; // On saute l'en-tête global

        for i in 0..(num_tracks * num_heads) {
            if offset + 0x100 > buffer.len() {
                break;
            }

            // En EDSK, le tableau des tailles des pistes commence à 0x34 (1 octet = 256 octets)
            let current_track_size = if is_extended {
                (buffer[0x34 + i] as usize) * 256
            } else {
                global_track_size
            };

            if current_track_size == 0 {
                continue; // Piste non formatée ou vide
            }

            let track_cylinder = buffer[offset + 0x10];
            let track_head = buffer[offset + 0x11];
            let num_sectors = buffer[offset + 0x15] as usize;

            let mut sectors = Vec::new();
            let mut sector_data_offset = offset + 0x100; // Les données suivent l'en-tête de piste

            for s in 0..num_sectors {
                let info_offset = offset + 0x18 + (s * 8);

                let c = buffer[info_offset + 0];
                let h = buffer[info_offset + 1];
                let r = buffer[info_offset + 2];
                let n = buffer[info_offset + 3];

                // Sur les jeux protégés (EDSK), la vraie taille du secteur prime sur le code 'N'
                // Elle est stockée dans les octets 6 et 7 du bloc d'information du secteur.
                let actual_size = if is_extended {
                    u16::from_le_bytes([buffer[info_offset + 6], buffer[info_offset + 7]]) as usize
                } else {
                    128 << n // Formule matérielle par défaut (N=2 -> 512 octets)
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
            offset += current_track_size; // On passe à la piste suivante
        }

        // println!(
        //     "[DSK] Image chargée : '{}' ({} pistes, étendu={})",
        //     path,
        //     tracks.len(),
        //     is_extended
        // );
        Ok(Self { tracks })
    }

    /// Récupère les données d'un secteur spécifique (C, H, R)
    pub fn get_sector_data(&self, cylinder: u8, head: u8, sector_id: u8) -> Option<&[u8]> {
        for track in &self.tracks {
            // Sur l'Amstrad, la tête est généralement 0
            if track.cylinder == cylinder && track.head == head {
                for sector in &track.sectors {
                    if sector.sector_id == sector_id {
                        return Some(&sector.data);
                    }
                }
            }
        }
        None // Secteur introuvable
    }
}
