//! Système audio SCSP (Saturn Custom Sound Processor) pour Model 2

use anyhow::Result;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream, StreamConfig,
};
use std::collections::VecDeque;

/// Registres SCSP (Saturn Custom Sound Processor)
#[derive(Debug, Clone)]
pub struct ScspRegisters {
    /// Registre de contrôle principal (0x100400)
    pub control: u32,

    /// Registre de statut (0x100404)
    pub status: u32,

    /// Volume maître (0x100408)
    pub master_volume: u16,

    /// Registre de contrôle des slots (0x10040C)
    pub slot_control: u32,

    /// Registres des slots individuels (32 slots)
    pub slot_registers: [SlotRegisters; 32],

    /// Mémoire DSP (4KB)
    pub dsp_memory: [u16; 2048],

    /// Mémoire wave (2MB)
    pub wave_memory: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct SlotRegisters {
    /// Volume du slot
    pub volume: u16,

    /// Fréquence du slot
    pub frequency: u16,

    /// Adresse de début dans la mémoire wave
    pub start_address: u32,

    /// Adresse de fin dans la mémoire wave
    pub end_address: u32,

    /// Adresse de boucle
    pub loop_address: u32,

    /// Contrôle du slot (attaque, decay, sustain, release)
    pub control: u16,

    /// Panoramique (gauche/droite)
    pub pan: u16,

    /// Type d'onde (PCM, noise, etc.)
    pub wave_type: u8,
}

/// État d'un slot audio
#[derive(Debug, Clone)]
struct SlotState {
    /// Position actuelle dans l'onde
    position: f32,

    /// Vitesse de lecture
    speed: f32,

    /// Volume actuel (avec enveloppe)
    current_volume: f32,

    /// Phase de l'enveloppe (attack, decay, sustain, release)
    envelope_phase: EnvelopePhase,

    /// Compteur pour l'enveloppe
    envelope_counter: u32,

    /// Actif ou non
    active: bool,
}

#[derive(Debug, Clone, Copy)]
enum EnvelopePhase {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

impl Default for EnvelopePhase {
    fn default() -> Self {
        EnvelopePhase::Idle
    }
}

/// Émulateur du processeur sonore SCSP
pub struct ScspAudio {
    sample_rate: u32,
    channels: u16,
    _stream: Stream,
    pub volume: f32,

    /// Registres SCSP
    pub registers: ScspRegisters,

    /// États des slots
    slot_states: [SlotState; 32],

    /// Buffer audio de sortie
    output_buffer: VecDeque<f32>,

    /// Taille du buffer
    buffer_size: usize,

    /// Horloge interne
    clock_counter: u64,
}

impl ScspAudio {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("Aucun périphérique audio disponible"))?;

        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let stream_config = StreamConfig {
            channels,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let buffer_size = (sample_rate / 60) as usize * channels as usize; // Buffer pour ~1 frame à 60Hz

        let audio = Self {
            sample_rate,
            channels,
            _stream: device.build_output_stream(
                &stream_config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Le callback sera configuré après l'initialisation
                    for sample in data.iter_mut() {
                        *sample = 0.0;
                    }
                },
                move |err| eprintln!("Erreur audio: {}", err),
                None,
            )?,
            volume: 1.0,
            registers: ScspRegisters::new(),
            slot_states: Default::default(),
            output_buffer: VecDeque::with_capacity(buffer_size * 2),
            buffer_size,
            clock_counter: 0,
        };

        // Démarrer le stream audio
        audio._stream.play()?;

        Ok(audio)
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Met à jour l'émulation audio (appelé périodiquement)
    pub fn update(&mut self, cycles: u32) {
        self.clock_counter = self.clock_counter.wrapping_add(cycles as u64);

        // Générer des échantillons audio
        self.generate_audio_samples();

        // Mettre à jour les enveloppes des slots
        self.update_envelopes();

        // Nettoyer les slots inactifs
        self.cleanup_inactive_slots();
    }

    /// Génère des échantillons audio
    fn generate_audio_samples(&mut self) {
        let samples_needed = (self.sample_rate as f32 / 44100.0 * 128.0) as usize; // ~128 échantillons à 44.1kHz

        for _ in 0..samples_needed {
            let mut left_sample = 0.0f32;
            let mut right_sample = 0.0f32;

            // Collecter les données nécessaires pour éviter les conflits d'emprunt
            let mut active_slots = Vec::new();
            for slot_id in 0..32 {
                if self.slot_states[slot_id].active {
                    let slot_regs = self.registers.slot_registers[slot_id].clone();
                    let slot_state_pos = self.slot_states[slot_id].position;
                    let slot_state_speed = self.slot_states[slot_id].speed;
                    let current_volume = self.slot_states[slot_id].current_volume;
                    active_slots.push((
                        slot_id,
                        slot_regs,
                        slot_state_pos,
                        slot_state_speed,
                        current_volume,
                    ));
                }
            }

            // Générer les échantillons pour chaque slot actif
            for (slot_id, slot_regs, mut position, speed, current_volume) in active_slots {
                // Générer l'échantillon pour ce slot
                let sample = self.generate_slot_sample_from_data(&slot_regs, &mut position, speed);

                // Mettre à jour la position dans le slot state
                self.slot_states[slot_id].position = position;

                // Appliquer le volume et le panoramique
                let volume = (slot_regs.volume as f32 / 0xFFF as f32) * current_volume;
                let pan = slot_regs.pan as f32 / 0x1F as f32; // 0-31 -> 0.0-1.0

                left_sample += sample * volume * (1.0 - pan);
                right_sample += sample * volume * pan;
            }

            // Appliquer le volume maître
            let master_volume = self.registers.master_volume as f32 / 0xFFF as f32;
            left_sample *= master_volume * self.volume;
            right_sample *= master_volume * self.volume;

            // Ajouter au buffer de sortie
            self.output_buffer.push_back(left_sample);
            if self.channels == 2 {
                self.output_buffer.push_back(right_sample);
            }

            // Limiter la taille du buffer
            while self.output_buffer.len() > self.buffer_size * 2 {
                self.output_buffer.pop_front();
            }
        }
    }

    /// Génère un échantillon pour un slot avec données locales (évite les conflits d'emprunt)
    fn generate_slot_sample_from_data(
        &self,
        slot_regs: &SlotRegisters,
        position: &mut f32,
        speed: f32,
    ) -> f32 {
        let sample = match slot_regs.wave_type {
            0 => self.generate_pcm_sample_from_data(slot_regs, *position), // PCM
            1 => self.generate_square_wave_from_data(*position),           // Carré
            2 => self.generate_triangle_wave_from_data(*position),         // Triangle
            3 => self.generate_noise_from_data(position),                  // Bruit
            _ => 0.0,
        };

        // Avancer la position
        *position += speed;

        // Gestion de la boucle
        if *position >= slot_regs.end_address as f32 {
            if slot_regs.loop_address < slot_regs.end_address {
                *position = slot_regs.loop_address as f32;
            } else {
                *position = slot_regs.start_address as f32;
            }
        }

        sample
    }

    /// Génère un échantillon PCM avec données locales
    fn generate_pcm_sample_from_data(&self, _slot_regs: &SlotRegisters, position: f32) -> f32 {
        let addr = position as usize;
        if addr < self.registers.wave_memory.len() {
            // Convertir u8 en f32 (-1.0 à 1.0)
            (self.registers.wave_memory[addr] as f32 - 128.0) / 128.0
        } else {
            0.0
        }
    }

    /// Génère une onde carrée avec données locales
    fn generate_square_wave_from_data(&self, position: f32) -> f32 {
        if position.fract() < 0.5 {
            0.5
        } else {
            -0.5
        }
    }

    /// Génère une onde triangle avec données locales
    fn generate_triangle_wave_from_data(&self, position: f32) -> f32 {
        let phase = position.fract();
        if phase < 0.25 {
            phase * 4.0
        } else if phase < 0.75 {
            2.0 - phase * 4.0
        } else {
            phase * 4.0 - 4.0
        }
    }

    /// Génère du bruit avec données locales
    fn generate_noise_from_data(&self, position: &mut f32) -> f32 {
        // Bruit simple basé sur un LFSR
        let mut lfsr = *position as u32;
        lfsr = (lfsr >> 1) | (((lfsr >> 0) ^ (lfsr >> 1) ^ (lfsr >> 21) ^ (lfsr >> 31)) << 31);
        *position = lfsr as f32;

        (lfsr as f32 / u32::MAX as f32 - 0.5) * 2.0
    }

    /// Met à jour les enveloppes des slots
    fn update_envelopes(&mut self) {
        for (slot_id, slot_state) in self.slot_states.iter_mut().enumerate() {
            if !slot_state.active {
                continue;
            }

            let _slot_regs = &self.registers.slot_registers[slot_id];
            slot_state.envelope_counter += 1;

            match slot_state.envelope_phase {
                EnvelopePhase::Attack => {
                    // Attaque rapide (quelques ms)
                    let attack_time = 1000; // échantillons
                    slot_state.current_volume =
                        (slot_state.envelope_counter as f32 / attack_time as f32).min(1.0);

                    if slot_state.envelope_counter >= attack_time {
                        slot_state.envelope_phase = EnvelopePhase::Decay;
                        slot_state.envelope_counter = 0;
                    }
                }
                EnvelopePhase::Decay => {
                    // Decay vers le sustain level
                    let decay_time = 2000;
                    let sustain_level = 0.7;
                    let decay_amount = 1.0 - sustain_level;
                    slot_state.current_volume = 1.0
                        - decay_amount
                            * (slot_state.envelope_counter as f32 / decay_time as f32).min(1.0);

                    if slot_state.envelope_counter >= decay_time {
                        slot_state.envelope_phase = EnvelopePhase::Sustain;
                        slot_state.envelope_counter = 0;
                    }
                }
                EnvelopePhase::Sustain => {
                    // Maintenir le niveau sustain
                    slot_state.current_volume = 0.7;
                }
                EnvelopePhase::Release => {
                    // Release vers zéro
                    let release_time = 3000;
                    slot_state.current_volume = 0.7
                        * (1.0 - slot_state.envelope_counter as f32 / release_time as f32).max(0.0);

                    if slot_state.envelope_counter >= release_time {
                        slot_state.active = false;
                        slot_state.envelope_phase = EnvelopePhase::Idle;
                    }
                }
                EnvelopePhase::Idle => {
                    slot_state.current_volume = 0.0;
                }
            }
        }
    }

    /// Nettoie les slots inactifs
    fn cleanup_inactive_slots(&mut self) {
        for slot_state in &mut self.slot_states {
            if matches!(slot_state.envelope_phase, EnvelopePhase::Idle) {
                slot_state.active = false;
                slot_state.current_volume = 0.0;
            }
        }
    }

    /// Démarre un slot audio
    pub fn start_slot(&mut self, slot_id: usize) {
        if slot_id >= 32 {
            return;
        }

        let slot_regs = &self.registers.slot_registers[slot_id];
        let slot_state = &mut self.slot_states[slot_id];

        slot_state.active = true;
        slot_state.position = slot_regs.start_address as f32;
        slot_state.speed = slot_regs.frequency as f32 / 1000.0; // Ajuster selon les besoins
        slot_state.current_volume = 0.0;
        slot_state.envelope_phase = EnvelopePhase::Attack;
        slot_state.envelope_counter = 0;
    }

    /// Arrête un slot audio
    pub fn stop_slot(&mut self, slot_id: usize) {
        if slot_id >= 32 {
            return;
        }

        let slot_state = &mut self.slot_states[slot_id];
        if slot_state.active {
            slot_state.envelope_phase = EnvelopePhase::Release;
            slot_state.envelope_counter = 0;
        }
    }

    /// Lit un registre SCSP
    pub fn read_register(&self, offset: u32) -> u32 {
        match offset {
            0x00 => self.registers.control,
            0x04 => self.registers.status,
            0x08 => self.registers.master_volume as u32,
            0x0C => self.registers.slot_control,
            _ => {
                // Registres de slots (0x10 - 0x1FF)
                if offset >= 0x10 && offset < 0x200 {
                    let slot_id = ((offset - 0x10) / 0x10) as usize;
                    let reg_offset = (offset - 0x10) % 0x10;

                    if slot_id < 32 {
                        match reg_offset {
                            0x00 => self.registers.slot_registers[slot_id].volume as u32,
                            0x04 => self.registers.slot_registers[slot_id].frequency as u32,
                            0x08 => self.registers.slot_registers[slot_id].start_address,
                            0x0C => self.registers.slot_registers[slot_id].control as u32,
                            _ => 0,
                        }
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
        }
    }

    /// Écrit dans un registre SCSP
    pub fn write_register(&mut self, offset: u32, value: u32) {
        match offset {
            0x00 => self.registers.control = value,
            0x04 => self.registers.status = value,
            0x08 => self.registers.master_volume = value as u16,
            0x0C => self.registers.slot_control = value,
            _ => {
                // Registres de slots (0x10 - 0x1FF)
                if offset >= 0x10 && offset < 0x200 {
                    let slot_id = ((offset - 0x10) / 0x10) as usize;
                    let reg_offset = (offset - 0x10) % 0x10;

                    if slot_id < 32 {
                        match reg_offset {
                            0x00 => self.registers.slot_registers[slot_id].volume = value as u16,
                            0x04 => self.registers.slot_registers[slot_id].frequency = value as u16,
                            0x08 => self.registers.slot_registers[slot_id].start_address = value,
                            0x0C => {
                                self.registers.slot_registers[slot_id].control = value as u16;

                                // Vérifier les bits de contrôle
                                let key_on = (value & 0x1000) != 0;
                                let key_off = (value & 0x2000) != 0;

                                if key_on {
                                    self.start_slot(slot_id);
                                } else if key_off {
                                    self.stop_slot(slot_id);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    /// Obtient des données audio pour le callback
    pub fn get_audio_data(&mut self, buffer: &mut [f32]) {
        for (_i, sample) in buffer.iter_mut().enumerate() {
            *sample = self.output_buffer.pop_front().unwrap_or(0.0) * self.volume;
        }
    }
}

impl ScspRegisters {
    pub fn new() -> Self {
        Self {
            control: 0x00000000,
            status: 0x00000001,    // SCSP prêt
            master_volume: 0x0FFF, // Volume max
            slot_control: 0x00000000,
            slot_registers: [SlotRegisters::default(); 32],
            dsp_memory: [0; 2048],
            wave_memory: vec![0; 2 * 1024 * 1024], // 2MB
        }
    }
}

impl Default for ScspRegisters {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SlotRegisters {
    fn default() -> Self {
        Self {
            volume: 0x0FFF,
            frequency: 1000,
            start_address: 0,
            end_address: 0x1000,
            loop_address: 0,
            control: 0x0000,
            pan: 0x0F,    // Centre
            wave_type: 0, // PCM
        }
    }
}

impl Default for SlotState {
    fn default() -> Self {
        Self {
            position: 0.0,
            speed: 1.0,
            current_volume: 0.0,
            envelope_phase: EnvelopePhase::Idle,
            envelope_counter: 0,
            active: false,
        }
    }
}

impl Default for ScspAudio {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| panic!("Impossible d'initialiser l'audio"))
    }
}
