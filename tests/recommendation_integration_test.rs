use megastore::{
    ConectaStoreError, NodeId, RecommendationConfig, RecommendationEngine, RelationType,
    StoreRepository,
};

fn fixture() -> Result<StoreRepository, ConectaStoreError> {
    let mut repository = StoreRepository::new();
    repository.add_client(1, "Ana")?;
    repository.add_category(1, "Tecnologia")?;
    repository.add_product(10, "Notebook", 3000.0, true)?;
    repository.add_product(20, "Monitor", 900.0, true)?;
    repository.add_product(30, "Indisponivel", 100.0, false)?;
    for product in [10, 20, 30] {
        repository.connect(
            NodeId::Product(product),
            NodeId::Category(1),
            RelationType::BelongsToCategory,
            1.0,
        )?;
    }
    repository.connect(
        NodeId::Client(1),
        NodeId::Product(10),
        RelationType::Purchase,
        1.0,
    )?;
    repository.connect(
        NodeId::Product(10),
        NodeId::Product(20),
        RelationType::Similarity,
        0.9,
    )?;
    Ok(repository)
}

#[test]
fn cliente_nao_recebe_produto_comprado_nem_indisponivel() -> Result<(), ConectaStoreError> {
    let repository = fixture()?;
    let engine = RecommendationEngine::new(repository.graph());
    let results = engine.for_client(1, RecommendationConfig::default())?;
    assert!(results.iter().any(|item| item.product_id == 20));
    assert!(!results.iter().any(|item| item.product_id == 10));
    assert!(!results.iter().any(|item| item.product_id == 30));
    Ok(())
}

#[test]
fn produto_origem_nao_e_recomendado() -> Result<(), ConectaStoreError> {
    let repository = fixture()?;
    let engine = RecommendationEngine::new(repository.graph());
    let results = engine.for_product(10, RecommendationConfig::default())?;
    assert_eq!(results.first().map(|item| item.product_id), Some(20));
    assert!(!results.iter().any(|item| item.product_id == 10));
    Ok(())
}

#[test]
fn grafo_completo_produz_ranking_esperado_para_cliente_e_produto() -> Result<(), ConectaStoreError>
{
    let mut repository = StoreRepository::new();
    repository.add_client(1, "Ana")?;
    repository.add_category(1, "Computadores")?;
    repository.add_category(2, "Acessorios")?;
    for (id, name) in [
        (10, "Notebook"),
        (20, "Mouse"),
        (30, "Teclado"),
        (40, "Monitor"),
    ] {
        repository.add_product(id, name, 100.0, true)?;
    }
    repository.connect(
        NodeId::Product(10),
        NodeId::Category(1),
        RelationType::BelongsToCategory,
        1.0,
    )?;
    for product in [20, 30, 40] {
        repository.connect(
            NodeId::Product(product),
            NodeId::Category(2),
            RelationType::BelongsToCategory,
            1.0,
        )?;
    }
    repository.connect(
        NodeId::Client(1),
        NodeId::Product(10),
        RelationType::Purchase,
        1.0,
    )?;
    repository.connect(
        NodeId::Client(1),
        NodeId::Product(20),
        RelationType::Interest,
        0.7,
    )?;
    repository.connect(
        NodeId::Client(1),
        NodeId::Product(30),
        RelationType::Rating,
        0.9,
    )?;
    repository.connect(
        NodeId::Product(10),
        NodeId::Product(40),
        RelationType::Similarity,
        0.95,
    )?;

    let engine = RecommendationEngine::new(repository.graph());
    let config = RecommendationConfig {
        max_depth: 2,
        limit: 10,
        distance_decay: 0.85,
    };
    let client_ids: Vec<u64> = engine
        .for_client(1, config)?
        .into_iter()
        .map(|item| item.product_id)
        .collect();
    let product_ids: Vec<u64> = engine
        .for_product(10, config)?
        .into_iter()
        .map(|item| item.product_id)
        .collect();

    assert_eq!(client_ids, vec![30, 40, 20]);
    assert_eq!(product_ids, vec![40, 30, 20]);
    Ok(())
}
