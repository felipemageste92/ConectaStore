use megastore::{
    NodeId, RecommendationConfig, RecommendationEngine, RelationType, Result, StoreRepository,
};

fn main() {
    if let Err(error) = run_demo() {
        eprintln!("Erro ao executar a demonstracao: {error}");
        std::process::exit(1);
    }
}

fn run_demo() -> Result<()> {
    let repository = build_demo_repository()?;
    let engine = RecommendationEngine::new(repository.graph());
    let config = RecommendationConfig {
        max_depth: 3,
        limit: 5,
        distance_decay: 0.85,
    };

    println!("ConectaStore - demonstracao reproduzivel\n");
    println!("Recomendacoes para Ana (cliente 1):");
    print_recommendations(&engine.for_client(1, config)?);
    println!("\nRecomendacoes a partir de Notebook (produto 101):");
    print_recommendations(&engine.for_product(101, config)?);
    Ok(())
}

fn build_demo_repository() -> Result<StoreRepository> {
    let mut repository = StoreRepository::new();
    repository.add_client(1, "Ana")?;
    repository.add_client(2, "Bruno")?;
    repository.add_category(10, "Informatica")?;
    repository.add_category(20, "Acessorios")?;
    repository.add_product(101, "Notebook", 3500.0, true)?;
    repository.add_product(102, "Mouse sem fio", 120.0, true)?;
    repository.add_product(103, "Teclado mecanico", 280.0, true)?;
    repository.add_product(104, "Monitor", 900.0, true)?;

    connect(
        &mut repository,
        NodeId::Product(101),
        NodeId::Category(10),
        RelationType::BelongsToCategory,
        1.0,
    )?;
    connect(
        &mut repository,
        NodeId::Product(102),
        NodeId::Category(20),
        RelationType::BelongsToCategory,
        1.0,
    )?;
    connect(
        &mut repository,
        NodeId::Product(103),
        NodeId::Category(20),
        RelationType::BelongsToCategory,
        1.0,
    )?;
    connect(
        &mut repository,
        NodeId::Product(104),
        NodeId::Category(10),
        RelationType::BelongsToCategory,
        1.0,
    )?;
    connect(
        &mut repository,
        NodeId::Client(1),
        NodeId::Product(101),
        RelationType::Purchase,
        1.0,
    )?;
    connect(
        &mut repository,
        NodeId::Client(1),
        NodeId::Product(102),
        RelationType::Interest,
        0.8,
    )?;
    connect(
        &mut repository,
        NodeId::Client(2),
        NodeId::Product(101),
        RelationType::Purchase,
        1.0,
    )?;
    connect(
        &mut repository,
        NodeId::Client(2),
        NodeId::Product(103),
        RelationType::Purchase,
        0.9,
    )?;
    connect(
        &mut repository,
        NodeId::Product(101),
        NodeId::Product(104),
        RelationType::Similarity,
        0.9,
    )?;
    Ok(repository)
}

fn connect(
    repository: &mut StoreRepository,
    from: NodeId,
    to: NodeId,
    relation: RelationType,
    weight: f64,
) -> Result<()> {
    repository.connect(from, to, relation, weight)
}

fn print_recommendations(recommendations: &[megastore::Recommendation]) {
    if recommendations.is_empty() {
        println!("Nenhuma recomendacao encontrada.");
        return;
    }
    for item in recommendations {
        println!(
            "ID: {} | {} | Categoria: {} | Pontuacao: {:.4} | Motivo: {}",
            item.product_id, item.name, item.category, item.score, item.reason
        );
    }
}
