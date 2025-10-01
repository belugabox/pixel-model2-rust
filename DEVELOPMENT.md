# Guide de Développement - Pixel Model 2 Rust

## 🎯 État Actuel et Tâches Prioritaires (Octobre 2025)

### ✅ Accomplissements Réalisés

- **Architecture complète** : Tous les modules (CPU, GPU, mémoire, ROM, audio, GUI) intégrés et fonctionnels
- **Émulateur opérationnel** : L'émulateur démarre et affiche une fenêtre GUI
- **Audio SCSP intégré** : Système audio complet avec accès registre via mémoire I/O
- **GPU wgpu initialisé** : Buffer de commandes GPU opérationnel
- **Compilation réussie** : Binaire GUI compilé sans erreurs (quelques warnings normaux)

### 🚧 Tâches Prioritaires Immédiates

#### 1. Corriger erreurs swap chain GPU

- **Objectif** : Éliminer les messages d'erreur répétés "The underlying surface has changed, and therefore the swap chain must be updated"
- **Impact** : Nettoyer les logs et améliorer la stabilité du rendu
- **Complexité** : Moyenne - nécessite gestion des événements de redimensionnement fenêtre

#### 2. Tester exécution CPU basique

- **Objectif** : Valider le décodeur et exécuteur d'instructions NEC V60 avec des instructions simples
- **Tests requis** : Instructions arithmétiques, logiques, et de transfert
- **Impact** : Base pour l'émulation CPU complète
- **Complexité** : Moyenne - nécessite ROMs de test ou code machine simple

#### 3. Implémenter rendu GPU

- **Objectif** : Intégrer le système de rendu pour afficher des primitives graphiques (triangles, textures)
- **Composants** : Utiliser les structures GPU existantes et les commandes du buffer
- **Impact** : Permettre l'affichage visuel des jeux
- **Complexité** : Élevée - nécessite compréhension des spécifications Model 2 GPU

#### 4. Tester avec ROMs réelles

- **Objectif** : Charger et exécuter des ROMs SEGA Model 2 authentiques
- **Étape préalable** : Implémenter chargement ROM complet dans le système mémoire
- **Tests** : Validation checksum, mapping mémoire, exécution basique
- **Impact** : Passage de l'émulation théorique à pratique
- **Complexité** : Moyenne - dépend du système ROM existant

#### 5. Optimisations performances

- **Objectif** : Améliorer les performances CPU, GPU et audio pour une exécution fluide
- **Métriques cibles** : 60 FPS stable, latence audio < 10ms
- **Optimisations** : Cache mémoire, batching GPU, optimisations CPU
- **Impact** : Expérience utilisateur fluide
- **Complexité** : Variable selon les goulots d'étranglement identifiés

### 📊 Métriques de Qualité Actuelles

- **Build**: ✅ Compilable en release et debug
- **Tests**: ✅ Tests unitaires opérationnels (145 warnings normaux pour code non utilisé)
- **GUI**: ✅ Fenêtre d'émulation fonctionnelle avec tous modules actifs
- **Audio**: ✅ SCSP intégré avec I/O routing complet
- **GPU**: ✅ Initialisé avec buffer de commandes (erreurs swap chain à corriger)
- **CPU**: ✅ Structure prête pour exécution (à tester)
- **Mémoire**: ✅ Système Model 2 complet avec cache et statistiques
- **ROM**: ✅ Framework de chargement et validation présent

## 📋 Plan d'Action Détaillé

### Phase 1 - Corrections Immédiates (Cette semaine)

#### 1.1 Correction Swap Chain GPU

```rust
// Dans src/gpu/mod.rs ou renderer.rs
impl Model2Gpu {
    pub fn handle_surface_change(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // Recréer la swap chain avec la nouvelle taille
        // Gérer les erreurs de surface perdue
    }
}
```

**Étapes** :

- Intercepter les événements de redimensionnement dans la GUI
- Appeler `handle_surface_change()` lors des changements de taille
- Ajouter gestion d'erreur pour surface perdue

#### 1.2 Tests CPU Basiques

```rust
#[test]
fn test_basic_cpu_execution() {
    let mut cpu = NecV60::new();
    let mut memory = Model2Memory::new();

    // Charger une instruction simple (ex: MOV R1, #42)
    let instruction_bytes = encode_mov_instruction(1, 42);
    memory.write_bytes(0x00000000, &instruction_bytes);

    // Exécuter et vérifier
    cpu.run_cycles(&mut memory, 1).unwrap();
    assert_eq!(cpu.registers.read_general(1), 42);
}
```

**Étapes** :

- Créer des ROMs de test simples
- Tester instructions de base (MOV, ADD, SUB)
- Valider les flags du processeur

### Phase 2 - Rendu et ROMs (1-2 semaines)

#### 2.1 Implémentation Rendu GPU

```rust
// Intégration dans gui/mod.rs
impl AppState {
    fn render_frame(&mut self) {
        // Traiter les commandes GPU du buffer
        let commands = self.memory.process_gpu_commands();
        for command in commands {
            self.gpu.execute_command(command);
        }
        // Rendre la frame
        self.gpu.render_frame();
    }
}
```

**Étapes** :

- Connecter le buffer de commandes GPU à l'exécuteur
- Implémenter rendu de triangles de base
- Ajouter support des textures

#### 2.2 Chargement ROMs Réelles

```rust
// Dans main_gui.rs
fn load_game_rom(rom_path: &str) -> Result<()> {
    let rom_data = std::fs::read(rom_path)?;
    let mut memory = Model2Memory::new();
    memory.load_rom("program".to_string(), rom_data)?;

    // Valider checksum et mapper en mémoire
    memory.validate_rom_integrity()?;
    Ok(())
}
```

**Étapes** :

- Implémenter chargement de fichiers ROM
- Validation des checksums
- Mapping correct en mémoire selon le type de ROM

### Phase 3 - Optimisation et Polissage (2-3 semaines)

#### 3.1 Optimisations Performance

```rust
// Benchmarks pour mesurer les performances
#[bench]
fn bench_cpu_execution(b: &mut Bencher) {
    let mut cpu = NecV60::new();
    let mut memory = Model2Memory::new();
    // Setup ROM de test

    b.iter(|| {
        cpu.run_cycles(&mut memory, 1000);
    });
}
```

**Étapes** :

- Ajouter benchmarks pour mesurer FPS
- Optimiser les goulots d'étranglement identifiés
- Améliorer la latence audio

#### 3.2 Interface Utilisateur

- Ajout menu de chargement de ROMs
- Affichage des statistiques de performance
- Contrôles de débogage (pause, step-by-step)

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

**Phase 1 - Corrections Immédiates (Cette semaine, priorité haute)**

1. ✅ **Corriger erreurs swap chain GPU** - Éliminer les messages d'erreur répétés lors du redimensionnement
2. ✅ **Tester exécution CPU basique** - Valider décodeur et exécuteur avec instructions simples
3. ✅ **Implémenter rendu GPU** - Connecter buffer de commandes au système de rendu

**Phase 2 - Fonctionnalités Core (1-2 semaines, priorité haute)**

1. **Charger et valider ROMs réelles** - Support complet des ROMs SEGA Model 2
2. **Améliorer interface utilisateur** - Menus de chargement, statistiques, contrôles de débogage
3. **Optimiser performances** - Atteindre 60 FPS stable avec faible latence audio

**Phase 3 - Polissage et Tests (2-3 semaines, priorité moyenne)**

1. **Tests d'intégration complets** - Scénarios d'émulation réalistes
2. **Documentation API complète** - Guides pour développeurs contributeurs
3. **Outils de débogage avancés** - Interface de débogage intégrée

**Phase 4 - Fonctionnalités Avancées (4+ semaines, priorité basse)**

1. **Audio SCSP étendu** - Effets audio avancés et DSP
2. **Support réseau** - Jeu en réseau (link play)
3. **Sauvegarde/chargement d'états** - Save states
4. **Emulation haute précision** - Timing cycle-exact, sous-systèmes complets

### Métriques de Qualité

- **Build**: ✅ Compilable en release et debug sans erreurs
- **Tests**: ✅ Tests unitaires opérationnels (quelques warnings normaux pour code non utilisé)
- **GUI**: ✅ Fenêtre d'émulation fonctionnelle avec tous modules actifs
- **Audio**: ✅ SCSP intégré avec I/O routing complet et accès registre
- **GPU**: ✅ Initialisé avec buffer de commandes (erreurs swap chain à corriger)
- **CPU**: ✅ Structure prête pour exécution (à tester avec instructions réelles)
- **Mémoire**: ✅ Système Model 2 complet avec cache, statistiques et mapping I/O
- **ROM**: ✅ Framework de chargement et validation présent
- **Performance**: ❓ À mesurer (cible: 60 FPS, latence audio < 10ms)
- **Coverage**: ❓ À mesurer avec tarpaulin
- **CI/CD**: ❓ GitHub Actions à configurer

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

## 🎯 Tâches de Développement Actives

### ✅ Priorité 1 - Corrections Immédiates

- [x] **Corriger erreurs swap chain GPU** : Gérer les événements de redimensionnement fenêtre pour éviter les erreurs répétées
  - Ajout de gestion des erreurs `SurfaceError::Lost` et `SurfaceError::Outdated`
  - Reconfiguration automatique de la surface lors de ces erreurs
  - Méthodes `render()`, `render_simple_triangles()` et `render_textured_triangles()` mises à jour
- [x] **Tester exécution CPU basique** : Créer des tests pour valider le décodeur et exécuteur d'instructions simples
  - Création de 8 tests d'exécution CPU (tous passent ✅)
  - Tests d'initialisation, reset, registres, mémoire, et cycles d'exécution
  - Tests de validation de l'intégration CPU-mémoire
- [ ] **Implémenter rendu GPU** : Connecter le buffer de commandes GPU au système de rendu pour afficher des primitives

### 📋 Priorité 2 - Fonctionnalités Core

- [ ] **Charger ROMs réelles** : Implémenter chargement et validation de ROMs SEGA Model 2 authentiques
- [ ] **Interface utilisateur** : Ajouter menus de chargement ROM, statistiques performance, contrôles débogage
- [ ] **Optimisations performance** : Atteindre 60 FPS stable avec latence audio optimisée

### 🔧 Priorité 3 - Polissage

- [ ] **Tests d'intégration** : Scénarios d'émulation réalistes avec ROMs de test
- [ ] **Documentation API** : Guides complets pour les développeurs contributeurs
- [ ] **Outils débogage** : Interface de débogage intégrée avec inspection mémoire/CPU

### 🚀 Priorité 4 - Fonctionnalités Avancées

- [ ] **Audio SCSP étendu** : Effets DSP, mixing avancé, filtres
- [ ] **Support réseau** : Jeu en réseau pour titres Model 2 compatibles
- [ ] **Save states** : Sauvegarde et chargement de l'état d'émulation
- [ ] **Emulation haute précision** : Timing cycle-exact, sous-systèmes complets

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
