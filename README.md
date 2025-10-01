# Pixel Model 2 Rust

Un émulateur du système d'arcade SEGA Model 2 écrit en Rust, inspiré par l'émulateur Model 2 d'ElSemi.

## 🎮 À propos

Le SEGA Model 2 était un système d'arcade révolutionnaire qui a alimenté des jeux légendaires comme :
- Virtua Fighter 2
- Daytona USA
- Virtua Cop
- Dead or Alive
- Virtua Striker
- Et bien d'autres !

Ce projet vise à émuler fidèlement le matériel Model 2 avec des performances optimales en Rust.

## 🏗️ Architecture

### Composants principaux

- **CPU** : Émulation du processeur NEC V60 à 25MHz
- **GPU** : Système de rendu 3D personnalisé avec textures et éclairage
- **Audio** : Processeur sonore SCSP (Saturn Custom Sound Processor)
- **Mémoire** : Gestion complète de la mémoire et mapping des adresses
- **Interface** : GUI moderne avec configuration des jeux

### Structure du projet

```
src/
├── main.rs              # Point d'entrée principal
├── lib.rs               # Bibliothèque principale
├── cpu/                 # Émulation du processeur NEC V60
├── memory/              # Système de gestion mémoire
├── gpu/                 # Rendu 3D et graphiques
├── audio/               # Système audio SCSP
├── input/               # Gestion des contrôles
├── rom/                 # Chargement et gestion des ROMs
├── gui/                 # Interface utilisateur
└── config/              # Configuration et paramètres
```

## 🚀 Démarrage rapide

### Prérequis

- Rust 1.70+
- GPU compatible Vulkan/DirectX 12/Metal

### Installation

```bash
git clone https://github.com/yourusername/pixel-model2-rust.git
cd pixel-model2-rust
cargo build --release
```

### Utilisation

```bash
# Lancer l'émulateur
cargo run --release

# Avec un fichier ROM spécifique
cargo run --release -- --rom "path/to/game.rom"
```

## 🎯 Fonctionnalités

- [x] Structure de base du projet
- [ ] Émulation CPU NEC V60
- [ ] Système mémoire Model 2
- [ ] Rendu 3D moderne (wgpu)
- [ ] Audio SCSP
- [ ] Interface graphique intuitive
- [ ] Support des formats ROM
- [ ] Sauvegarde d'états
- [ ] Configuration avancée

## 🔧 Développement

### Tests

```bash
# Tests unitaires
cargo test

# Benchmarks de performance
cargo bench
```

### Structure des benchmarks

- `cpu_benchmark` : Performance du processeur émulé
- `memory_benchmark` : Vitesse d'accès mémoire

## 📚 Documentation technique

### Spécifications SEGA Model 2

- **CPU Principal** : NEC V60 @ 25MHz
- **CPU Son** : Motorola 68000 @ 11.3MHz  
- **RAM Principale** : 8MB
- **RAM Vidéo** : 4MB
- **RAM Son** : 512KB
- **Résolution** : 496×384 ou 640×480

### Architecture GPU

Le Model 2 utilise un système de rendu 3D pionnier avec :
- Triangles texturés
- Z-buffering
- Éclairage Gouraud
- Transparence

## 🤝 Contribution

Les contributions sont les bienvenues ! Merci de :

1. Fork le projet
2. Créer une branche feature (`git checkout -b feature/amazing-feature`)
3. Commit vos changements (`git commit -m 'Add amazing feature'`)
4. Push sur la branche (`git push origin feature/amazing-feature`)
5. Ouvrir une Pull Request

## 📄 Licence

Ce projet est sous licence MIT OU Apache-2.0. Voir les fichiers `LICENSE-MIT` et `LICENSE-APACHE` pour plus de détails.

## 🙏 Remerciements

- ElSemi pour son travail pionnier sur l'émulation Model 2
- La communauté d'émulation pour la documentation du hardware
- SEGA pour avoir créé ce système d'arcade légendaire