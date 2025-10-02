//! Test du système de commande GPU avec buffer

use pixel_model2_rust::memory::{GpuCommand, Model2Memory, RenderStateType};

#[test]
fn test_gpu_command_buffer_batching() {
    println!("=== Test du buffer de commandes GPU ===");

    // Créer la mémoire
    let mut memory = Model2Memory::new();

    // Créer quelques commandes GPU de test
    let commands = vec![
        GpuCommand::ClearScreen {
            color: [0.1, 0.2, 0.3, 1.0],
            depth: 1.0,
            stencil: 0,
        },
        GpuCommand::SetModelMatrix([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]),
        GpuCommand::SetViewMatrix([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]),
        GpuCommand::LoadTexture {
            id: 1,
            data: vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
            width: 2,
            height: 2,
        },
        GpuCommand::SetRenderState {
            state: RenderStateType::Texturing,
            enabled: true,
        },
        GpuCommand::DrawTriangle {
            vertices: [
                pixel_model2_rust::memory::GpuVertex {
                    x: -0.5,
                    y: -0.5,
                    z: 0.0,
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                    u: 0.0,
                    v: 0.0,
                },
                pixel_model2_rust::memory::GpuVertex {
                    x: 0.5,
                    y: -0.5,
                    z: 0.0,
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    a: 1.0,
                    u: 1.0,
                    v: 0.0,
                },
                pixel_model2_rust::memory::GpuVertex {
                    x: 0.0,
                    y: 0.5,
                    z: 0.0,
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    a: 1.0,
                    u: 0.5,
                    v: 1.0,
                },
            ],
            texture_id: Some(1),
        },
    ];

    // Injecter les commandes dans le buffer via les I/O registers
    for command in &commands {
        memory.enqueue_gpu_command(command.clone());
    }

    // Vérifier que les commandes sont dans le buffer
    println!(
        "Commandes dans le buffer: {}",
        memory.gpu_command_buffer.len()
    );
    assert_eq!(memory.gpu_command_buffer.len(), commands.len());

    // Vérifier les statistiques initiales (avant traitement)
    let initial_stats = memory.gpu_command_buffer.stats();
    println!(
        "Commandes traitées initialement: {}",
        initial_stats.total_commands_processed
    );
    assert_eq!(initial_stats.total_commands_processed, 0); // Aucune commande traitée encore

    // Traiter les commandes par lots
    let command_batches = memory.process_gpu_commands();
    println!("Commandes traitées: {}", command_batches.len());

    // Vérifier que nous avons des commandes
    assert!(!command_batches.is_empty());
    assert_eq!(command_batches.len(), commands.len());

    // Vérifier l'optimisation des commandes (elles devraient être groupées par type)
    let mut state_commands = 0;
    let mut texture_commands = 0;
    let mut draw_commands = 0;

    for command in &command_batches {
        match command {
            GpuCommand::SetModelMatrix(_)
            | GpuCommand::SetViewMatrix(_)
            | GpuCommand::SetRenderState { .. } => {
                state_commands += 1;
            }
            GpuCommand::LoadTexture { .. } => {
                texture_commands += 1;
            }
            GpuCommand::DrawTriangle { .. } => {
                draw_commands += 1;
            }
            _ => {}
        }
    }

    println!("Commandes d'état: {}", state_commands);
    println!("Commandes de texture: {}", texture_commands);
    println!("Commandes de dessin: {}", draw_commands);

    // Vérifier que les commandes sont correctement réparties
    assert_eq!(state_commands, 3); // Clear + 2 matrices + 1 render state
    assert_eq!(texture_commands, 1);
    assert_eq!(draw_commands, 1);

    // Tester le vidage du buffer à la fin du frame
    let remaining_commands = memory.flush_gpu_command_buffer();
    println!(
        "Commandes restantes après flush: {}",
        remaining_commands.len()
    );

    // Vérifier les statistiques finales
    let final_stats = memory.gpu_command_buffer.stats();
    println!("Statistiques finales du buffer:");
    println!("  Lots traités: {}", final_stats.batches_processed);
    println!(
        "  Taille moyenne des lots: {:.1}",
        final_stats.average_batch_size
    );
    println!("  Taille max des lots: {}", final_stats.max_batch_size);

    assert!(final_stats.batches_processed > 0);
    assert!(final_stats.average_batch_size > 0.0);

    println!("✅ Test du buffer de commandes GPU réussi!");
}
