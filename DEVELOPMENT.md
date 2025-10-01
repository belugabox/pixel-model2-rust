# Guide de Développement - Pixel Model 2 Rust

## � Analyse du Projet (Octobre 2025)

### État Actuel du Projet

**Points positifs :**

- Architecture bien structurée avec séparation claire des modules (CPU, GPU, mémoire, ROM, audio, GUI)
- Utilise des crates modernes (wgpu, egui, etc.)
- Build réussi en release avec optimisations
- Tests unitaires fonctionnels (execution_tests.rs passe)
- Documentation de base présente

**Problèmes identifiés :**

- Tests d'intégration cassés (fichier corrompu `tests/integration_tests.rs`)
- 25+ warnings de compilation (imports inutiles, variables mortes, code unreachable)
- Dépendances obsolètes (egui 0.26→0.32, wgpu 0.19→26, etc.)
- Implémentation très basique (pas de vraie émulation CPU/GPU)
- GUI désactivée (commentée dans lib.rs)
- Pas de CI/CD
- Documentation incomplète
- Ambiguous glob re-exports dans lib.rs

### Plan d'Action Priorisé

**Phase 1 - Stabilisation (1-2 semaines, priorité haute)**

1. ✅ Corriger les tests d'intégration (refaire le fichier corrompu)
2. ✅ Nettoyer les warnings (cargo fix + corrections manuelles, réduit de 25 à 8)
3. ✅ Mettre à jour les dépendances critiques (egui, cpal, nalgebra, etc.)
4. ✅ Ajouter CI GitHub Actions basique

**Phase 2 - Fonctionnalités Core (2-4 semaines, priorité haute)**

1. Implémenter le décodage d'instructions V60
2. Ajouter rendu GPU basique (triangles sans textures)
3. Charger et valider de vraies ROMs Model 2
4. Réactiver et améliorer la GUI

**Phase 3 - Optimisation et Qualité (2-3 semaines, priorité moyenne)**

1. Ajouter profilage et benchmarks
2. Optimiser les performances CPU/GPU
3. Améliorer la documentation API
4. Ajouter tests d'intégration complets

**Phase 4 - Fonctionnalités Avancées (4+ semaines, priorité basse)**

1. Audio SCSP complet
2. Support réseau (link play)
3. Sauvegarde/chargement d'états
4. Interface de débogage avancée

### Métriques de Qualité

- **Build**: ✅ Compilable en release
- **Tests**: ✅ Tests unitaires et d'intégration OK (65 unit tests + 7 intégration + 9 execution + 7 decoder + 8 texture)
- **Warnings**: ✅ Réduit de 25 à 8 warnings (principalement des re-exports ambigus et code mort)
- **Dépendances**: ✅ Mises à jour vers versions compatibles (egui 0.26, cpal 0.16, nalgebra 0.34, etc.)
- **CI/CD**: ✅ GitHub Actions workflow ajouté (test sur Windows/Linux/macOS)
- **Coverage**: ❓ À mesurer
- **Performance**: ❓ À profiler

## �🚀 Démarrage Rapide

### Prérequis

- Rust 1.70+ (installer depuis [rustup.rs](https://rustup.rs/))
- GPU compatible Vulkan/DirectX 12/Metal
- Carte son pour l'audio

### Installation

```bash
# Cloner le dépôt
git clone https://github.com/yourusername/pixel-model2-rust.git
cd pixel-model2-rust

# Windows
install.bat

# Linux/macOS
chmod +x install.sh
./install.sh
```

## 🏗️ Architecture du Code

### Modules Principaux

#### CPU (`src/cpu/`)

- `mod.rs` - Structure principale du NEC V60
- `registers.rs` - Registres et flags du processeur
- `instructions.rs` - Définition des instructions
- `decoder.rs` - Décodage des opcodes
- `executor.rs` - Exécution des instructions

#### Mémoire (`src/memory/`)

- `mod.rs` - Bus mémoire principal avec cache
- `interface.rs` - Trait commun pour tous les types de mémoire
- `mapping.rs` - Mapping des adresses Model 2
- `ram.rs` - Implémentation de la RAM avec statistiques
- `rom.rs` - Gestion des ROMs et vérification d'intégrité

#### GPU (`src/gpu/`)

- `mod.rs` - GPU principal avec statistiques de rendu
- `renderer.rs` - Rendu moderne avec wgpu
- `geometry.rs` - Traitement de géométrie 3D
- `texture.rs` - Gestionnaire de textures
- `framebuffer.rs` - Framebuffer virtuel pour émulation précise
- `shaders/` - Shaders WGSL pour le rendu

#### Audio (`src/audio/`)

- `mod.rs` - Émulation SCSP avec cpal

#### Autres

- `input/` - Gestion des contrôles
- `rom/` - Chargement des ROMs de jeux
- `config/` - Configuration sérialisable
- `gui/` - Interface principale avec winit

## 🔧 Développement

### Ajouter un Nouveau Jeu

1. **Ajouter les informations du jeu** dans `src/memory/rom.rs` :

```rust
pub fn mon_jeu() -> GameInfo {
    GameInfo {
        name: "Mon Jeu".to_string(),
        short_name: "mon_jeu".to_string(),
        year: 1995,
        publisher: "SEGA".to_string(),
        required_roms: vec![
            "program.rom".to_string(),
            "graphics.rom".to_string(),
        ],
        optional_roms: vec![],
        special_config: None,
    }
}
```

2. **Mettre à jour le chargeur** dans `src/rom/mod.rs` :

```rust
let game_info = match game_name {
    "mon_jeu" => GameInfo::mon_jeu(),
    // ... autres jeux
};
```

### Ajouter une Nouvelle Instruction CPU

1. **Définir l'instruction** dans `src/cpu/instructions.rs` :

```rust
pub enum Instruction {
    // Instructions existantes...
    MonNouvelleInstruction { dest: Operand, src: Operand },
}
```

2. **Ajouter le décodage** dans `src/cpu/decoder.rs`
3. **Implémenter l'exécution** dans `src/cpu/executor.rs`

### Tests et Benchmarks

```bash
# Tests unitaires
cargo test

# Tests d'intégration
cargo test --test integration_tests

# Benchmarks
cargo bench

# Coverage (avec tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Débogage

#### Mode Debug

```rust
// Dans config.toml
[emulation]
debug_mode = true
```

#### Logs

```bash
RUST_LOG=debug cargo run
```

#### Stats GPU

Les statistiques de rendu sont accessibles via `gpu.get_stats()` :

- FPS instantanés et moyens
- Nombre de triangles par frame
- Temps de rendu

## 📊 Optimisation

### Profiling

```bash
# Profiling avec perf (Linux)
cargo build --release
perf record ./target/release/pixel-model2-rust
perf report

# Profiling avec Instruments (macOS)
cargo instruments -t "Time Profiler"
```

### Cache Mémoire

Le système de cache dans `Model2Memory` peut être ajusté :

- Taille du cache : `max_entries` dans `MemoryCache`
- Stratégie d'éviction : actuellement FIFO simple

### Optimisations GPU

- Utilisation du cache de texture
- Batching des triangles similaires
- Culling des triangles hors écran

## 🧪 Tests de Régression

### ROMs de Test

Créer des ROMs de test simples pour valider :

```rust
#[test]
fn test_cpu_add_instruction() {
    let mut cpu = NecV60::new();
    let mut memory = Model2Memory::new();

    // Charger une instruction ADD
    memory.write_u32(0x00000000, encode_add_instruction()).unwrap();

    // Exécuter et vérifier le résultat
    cpu.step(&mut memory).unwrap();
    assert_eq!(cpu.registers.read_general(1), expected_value);
}
```

### Tests de Performance

```rust
#[test]
fn test_60fps_requirement() {
    let mut emulator = setup_emulator();
    let start = std::time::Instant::now();

    // Simuler 1 seconde d'émulation
    for _ in 0..60 {
        emulator.run_frame().unwrap();
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_secs_f32() < 1.1); // Tolérance de 10%
}
```

## 📚 Ressources

### Documentation Model 2

- Spécifications hardware SEGA Model 2
- Documentation du NEC V60
- Format des ROMs et checksums

### Outils Utiles

- **Hex Editor** - Pour analyser les ROMs
- **Disassembler** - Pour comprendre le code machine
- **GPU Debugger** - RenderDoc pour le débogage graphique

## 🤝 Contribution

1. **Forker** le dépôt
2. **Créer une branche** : `git checkout -b feature/ma-fonctionnalite`
3. **Commit** : `git commit -m 'Ajout de ma fonctionnalité'`
4. **Tests** : `cargo test && cargo bench`
5. **Push** : `git push origin feature/ma-fonctionnalite`
6. **Pull Request**

### Style de Code

```bash
# Formatage automatique
cargo fmt

# Linting
cargo clippy -- -D warnings
```

### Commits

Format conseillé :

```
type(scope): description

body (optionnel)

Fixes #123
```

Types : `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## 🐛 Débogage Courant

### "Instruction inconnue"

- Vérifier le décodage dans `decoder.rs`
- Ajouter des logs pour l'opcode
- Comparer avec documentation NEC V60

### "Accès mémoire hors limites"

- Vérifier le mapping mémoire
- Logs des accès avec adresses
- Valider les calculs d'offset

### Performance faible

- Profiler avec `cargo bench`
- Vérifier le cache mémoire
- Optimiser les boucles critiques

### Rendu incorrect

- Vérifier les shaders WGSL
- Debugger avec RenderDoc
- Valider les matrices de transformation
