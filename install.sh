#!/bin/bash

echo "Installation de Pixel Model 2 Rust Emulator"
echo "==========================================="
echo ""

# Vérifier si Rust est installé
if ! command -v cargo &> /dev/null; then
    echo "Rust n'est pas installé. Veuillez installer Rust depuis https://rustup.rs/"
    echo "Puis relancer ce script."
    exit 1
fi

echo "Rust détecté. Version :"
cargo --version

echo ""
echo "Compilation du projet..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Erreur lors de la compilation !"
    exit 1
fi

echo ""
echo "Compilation réussie !"
echo ""
echo "Pour lancer l'émulateur :"
echo "  cargo run --release"
echo ""
echo "Pour charger un jeu spécifique :"
echo "  cargo run --release -- --rom \"chemin/vers/rom.bin\""
echo ""
echo "Pour lancer les benchmarks :"
echo "  cargo bench"
echo ""
echo "Jeux supportés :"
echo "  - Virtua Fighter 2 (vf2)"
echo "  - Daytona USA (daytona)"
echo ""