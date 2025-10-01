@echo off
echo Installation de Pixel Model 2 Rust Emulator
echo ===========================================
echo.

REM Vérifier si Rust est installé
cargo --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Rust n'est pas installé. Veuillez installer Rust depuis https://rustup.rs/
    echo Puis relancer ce script.
    pause
    exit /b 1
)

echo Rust détecté. Version :
cargo --version

echo.
echo Compilation du projet...
cargo build --release

if %errorlevel% neq 0 (
    echo Erreur lors de la compilation !
    pause
    exit /b 1
)

echo.
echo Compilation réussie !
echo.
echo Pour lancer l'émulateur :
echo   cargo run --release
echo.
echo Pour charger un jeu spécifique :
echo   cargo run --release -- --rom "chemin/vers/rom.bin"
echo.
echo Pour lancer les benchmarks :
echo   cargo bench
echo.
echo Jeux supportés :
echo   - Virtua Fighter 2 (vf2)
echo   - Daytona USA (daytona)
echo.
pause