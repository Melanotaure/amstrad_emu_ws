// src/amstrad/fdc_765.rs

use crate::dsk::DiskImage;

#[derive(PartialEq)]
enum FdcState {
    Idle,
    ReadingCommand,
    ExecutionRead,   // Le Z80 lit les données du secteur
    _ExecutionWrite, // Le Z80 écrit des données vers le secteur
    Result,          // Le Z80 lit les octets de statut (ST0, ST1...)
}

pub struct Fdc765 {
    state: FdcState,

    // Le registre de statut principal (MSR)
    msr: u8,

    // Buffers pour stocker la commande en cours et ses paramètres
    command_buffer: [u8; 10],
    command_index: usize,
    command_length: usize,

    // Buffer pour la donnée du secteur ou les résultats
    data_buffer: Vec<u8>,
    data_index: usize,

    // État du moteur (Port 0xFA7E)
    pub motor_on: bool,

    pub disk: Option<DiskImage>,

    interrupt_pending: bool,

    pub current_cylinder: u8,
}

impl Fdc765 {
    pub fn new() -> Self {
        Self {
            state: FdcState::Idle,
            msr: 0x80, // Bit 7 (RQM) à 1 : Prêt à recevoir une commande
            command_buffer: [0; 10],
            command_index: 0,
            command_length: 0,
            data_buffer: Vec::new(),
            data_index: 0,
            motor_on: false,
            disk: None,
            interrupt_pending: false,
            current_cylinder: 0,
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

            // CORRECTION : Si la commande ne fait qu'un octet (ex: 0x08),
            // on l'exécute IMMÉDIATEMENT !
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
        if self.state == FdcState::ExecutionRead || self.state == FdcState::Result {
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
            0xFF // Erreur si le Z80 lit au mauvais moment
        }
    }

    fn update_msr(&mut self) {
        // RQM (Bit 7) : 1 = Prêt à échanger, 0 = Occupé
        // DIO (Bit 6) : 1 = Donnée du FDC vers Z80 (Lecture), 0 = Z80 vers FDC (Écriture)
        // CB  (Bit 4) : 1 = Commande en cours d'exécution
        match self.state {
            FdcState::Idle => self.msr = 0x80,           // RQM=1, DIO=0, CB=0
            FdcState::ReadingCommand => self.msr = 0x90, // RQM=1, DIO=0, CB=1
            FdcState::ExecutionRead => self.msr = 0xF0,  // RQM=1, DIO=1, CB=1 (Donnée prête)
            FdcState::Result => self.msr = 0xD0,         // RQM=1, DIO=1, CB=1 (Résultat prêt)
            _ => self.msr = 0x10,                        // Occupé
        }
    }

    fn execute_command(&mut self) {
        let cmd = self.command_buffer[0] & 0x1F;
        // println!("[FDC] Exécution de la commande : 0x{:02X}", cmd);

        match cmd {
            0x03 => {
                // Specify (Configure timings) -> Pas de résultat
                self.state = FdcState::Idle;
                self.update_msr();
            }
            0x07 | 0x0F => {
                // Recalibrate (0x07) ou Seek (0x0F) -> Déclenche une "interruption" logicielle
                self.interrupt_pending = true;
                self.state = FdcState::Idle;
                self.update_msr();
            }
            0x08 => {
                // Sense Interrupt Status (Vérifie si un Seek/Recalibrate est terminé)
                if self.interrupt_pending {
                    self.interrupt_pending = false;
                    // ST0 = 0x20 (Seek End), PCN = 0x00 (Piste 0, on simule pour l'instant)
                    self.data_buffer = vec![0x20, 0x00];
                } else {
                    // Très important : si pas d'interruption en attente, on renvoie une erreur
                    self.data_buffer = vec![0x80]; // ST0 = Invalid Command
                }
                self.transition_to_result();
            }
            0x04 => {
                // Sense Drive Status (L'AMSDOS vérifie si le disque tourne et est prêt)
                let drive = self.command_buffer[1] & 0x03;
                let ready_bit = 0x20; // 1 = Disque inséré et prêt
                let track0_bit = 0x10; // 1 = La tête est sur la piste 0
                self.data_buffer = vec![ready_bit | track0_bit | drive]; // Renvoie ST3
                self.transition_to_result();
            }
            0x06 => {
                let cylinder = self.command_buffer[2];
                let head = self.command_buffer[3];
                let start_sector = self.command_buffer[4];
                let size_code = self.command_buffer[5];
                let eot = self.command_buffer[6]; // End Of Track (Le dernier secteur à lire)

                // println!(
                //     "[FDC] Read Data : Piste={}, Tête={}, Secteurs de {:02X} à {:02X}",
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
                        // On boucle pour récupérer TOUS les secteurs demandés
                        while current_sector <= eot {
                            if let Some(sector) =
                                track.sectors.iter().find(|s| s.sector_id == current_sector)
                            {
                                data_to_send.extend_from_slice(&sector.data);
                                current_sector += 1;
                            } else {
                                error = true; // Secteur manquant sur la disquette
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

                    // On sauvegarde le secteur d'arrêt pour la phase de résultat
                    self.command_buffer[4] = current_sector;
                } else {
                    // println!(
                    //     "[FDC] Erreur matérielle : Lecture échouée sur le secteur {:02X}",
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
                    // On force la phase de résultat avec l'erreur
                    self.transition_to_result();
                }
            }
            0x0A => {
                // Read ID : L'AMSDOS veut savoir comment est formatée cette piste
                // println!("[FDC] Read ID (Cylindre actuel: {})", self.current_cylinder);

                let mut found = false;

                if let Some(disk) = &self.disk {
                    // On cherche la piste correspondante dans l'image .DSK
                    if let Some(track) = disk
                        .tracks
                        .iter()
                        .find(|t| t.cylinder == self.current_cylinder)
                    {
                        // On prend le tout premier secteur de cette piste
                        if let Some(sector) = track.sectors.first() {
                            // On renvoie la phase de résultat (7 octets) avec les infos de ce secteur
                            self.data_buffer = vec![
                                0x00, // ST0 : Succès
                                0x00, // ST1
                                0x00, // ST2
                                sector.cylinder,
                                sector.head,
                                sector.sector_id, // <-- C'est l'info que veut l'AMSDOS (ex: 0xC1)
                                sector.size_code,
                            ];
                            found = true;
                        }
                    }
                }

                if !found {
                    // Erreur matérielle : La piste est introuvable ou non formatée
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

                // Read ID passe directement en phase de résultat (pas d'exécution)
                self.transition_to_result();
            }
            _ => {
                // Sécurité matérielle : Commande inconnue = Erreur propre !
                // println!("[FDC] Commande inconnue ou non supportée -> ST0=0x80");
                self.data_buffer = vec![0x80]; // ST0: Invalid Command
                self.transition_to_result();
            }
        }
    }

    fn transition_to_result(&mut self) {
        let cmd = self.command_buffer[0] & 0x1F;

        // Seulement si on a lu les données avec succès
        if cmd == 0x06 && self.state == FdcState::ExecutionRead {
            self.data_buffer = vec![
                0x00, // ST0 : Succès
                0x00, // ST1
                0x00, // ST2
                self.command_buffer[2],
                self.command_buffer[3],
                self.command_buffer[4], // Contient maintenant la valeur de EOT + 1
                self.command_buffer[5],
            ];
        }

        self.data_index = 0;
        self.state = FdcState::Result;
        self.update_msr();
        // println!(
        //     "[FDC] Phase de Résultat prête ({} octets)",
        //     self.data_buffer.len()
        // );
    }

    fn get_command_length(&self, cmd: u8) -> usize {
        // On masque les bits 5, 6, 7 (MT, MFM, SK) pour isoler le code de la commande
        match cmd & 0x1F {
            0x03 => 3, // Specify : Cmd + 2 octets
            0x04 => 2, // Sense Device Status : Cmd + 1 octet
            0x07 => 2, // Recalibrate : Cmd + 1 octet
            0x08 => 1, // Sense Interrupt Status : Cmd seule
            0x0A => 2,
            0x0F => 3, // Seek : Cmd + 2 octets

            // Les commandes de Lecture/Écriture ont 8 paramètres
            0x06 | 0x0C | 0x05 | 0x09 => 9,

            _ => 1, // Inconnu, on évite de bloquer la machine à états
        }
    }
}
