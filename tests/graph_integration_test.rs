use megastore::{ConectaStoreError, NodeId, RelationType, StoreRepository};

#[test]
fn cadastra_consulta_e_atualiza_conexao() -> Result<(), ConectaStoreError> {
    let mut repository = StoreRepository::new();
    repository.add_client(1, "Ana")?;
    repository.add_product(10, "Notebook", 3000.0, true)?;
    repository.connect(
        NodeId::Client(1),
        NodeId::Product(10),
        RelationType::Interest,
        0.5,
    )?;
    repository.connect(
        NodeId::Client(1),
        NodeId::Product(10),
        RelationType::Interest,
        0.9,
    )?;

    assert_eq!(repository.product(10)?.name, "Notebook");
    let edges = repository.graph().outgoing(NodeId::Client(1))?;
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].weight, 0.9);
    Ok(())
}

#[test]
fn rejeita_peso_e_relacionamento_invalidos() -> Result<(), ConectaStoreError> {
    let mut repository = StoreRepository::new();
    repository.add_client(1, "Ana")?;
    repository.add_product(10, "Notebook", 3000.0, true)?;

    let weight_error = repository.connect(
        NodeId::Client(1),
        NodeId::Product(10),
        RelationType::Purchase,
        1.5,
    );
    assert!(matches!(
        weight_error,
        Err(ConectaStoreError::InvalidWeight(_))
    ));

    let relation_error = repository.connect(
        NodeId::Product(10),
        NodeId::Client(1),
        RelationType::Purchase,
        1.0,
    );
    assert!(matches!(
        relation_error,
        Err(ConectaStoreError::InvalidRelationship { .. })
    ));
    Ok(())
}
