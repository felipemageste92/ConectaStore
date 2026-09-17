//! Programa de demonstração da biblioteca ConectaStore.

use megastore::{
    NodeId, RecommendationConfig, RecommendationEngine, RelationType, Result, StoreRepository,
};

fn main() {
    // Executa o fluxo principal e encerra com erro caso alguma validação falhe.
    if let Err(error) = run_demo() {
        eprintln!("Erro ao executar a demonstracao: {error}");
        std::process::exit(1);
    }
}

fn run_demo() -> Result<()> {
    // Etapa 1: prepara os dados e cria o motor que apenas consulta o grafo.
    let repository = build_demo_repository()?;
    let engine = RecommendationEngine::new(repository.graph());
    let config = RecommendationConfig {
        max_depth: 3,
        limit: 5,
        distance_decay: 0.85,
    };

    // Etapa 2: gera recomendações usando um cliente como ponto inicial.
    println!("ConectaStore - demonstracao reproduzivel\n");
    println!("Recomendacoes para Ana (cliente 1):");
    print_recommendations(&engine.for_client(1, config)?);
    // Etapa 3: repete a busca usando um produto como ponto inicial.
    println!("\nRecomendacoes a partir de Notebook (produto 101):");
    print_recommendations(&engine.for_product(101, config)?);
    Ok(())
}

/// Monta um pequeno cenário fixo para que a saída possa ser reproduzida.
fn build_demo_repository() -> Result<StoreRepository> {
    // Primeiro são cadastrados todos os vértices: clientes, categorias e produtos.
    let mut repository = StoreRepository::new();
    repository.add_client(1, "Ana")?;
    repository.add_client(2, "Bruno")?;
    repository.add_category(10, "Informatica")?;
    repository.add_category(20, "Acessorios")?;
    repository.add_product(101, "Notebook", 3500.0, true)?;
    repository.add_product(102, "Mouse sem fio", 120.0, true)?;
    repository.add_product(103, "Teclado mecanico", 280.0, true)?;
    repository.add_product(104, "Monitor", 900.0, true)?;

    // Em seguida, cada produto é associado à sua categoria.
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
    // Registra compras e interesses que representam o comportamento dos clientes.
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
    // Por fim, cria uma relação direta de similaridade entre dois produtos.
    connect(
        &mut repository,
        NodeId::Product(101),
        NodeId::Product(104),
        RelationType::Similarity,
        0.9,
    )?;
    Ok(repository)
}

/// Pequeno auxiliar que deixa a construção do cenário mais legível.
fn connect(
    repository: &mut StoreRepository,
    from: NodeId,
    to: NodeId,
    relation: RelationType,
    weight: f64,
) -> Result<()> {
    repository.connect(from, to, relation, weight)
}

/// Exibe a lista final ou uma mensagem apropriada quando ela estiver vazia.
fn print_recommendations(recommendations: &[megastore::Recommendation]) {
    // Trata o caso vazio antes de percorrer os itens.
    if recommendations.is_empty() {
        println!("Nenhuma recomendacao encontrada.");
        return;
    }
    // Cada linha apresenta os dados comerciais e a justificativa do algoritmo.
    for item in recommendations {
        println!(
            "ID: {} | {} | Categoria: {} | Pontuacao: {:.4} | Motivo: {}",
            item.product_id, item.name, item.category, item.score, item.reason
        );
    }
}


