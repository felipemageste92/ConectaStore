//! Testes da API pública de cadastro, consulta e conexão do grafo.

use megastore::{ConectaStoreError, NodeId, RelationType, StoreRepository};

#[test]
fn cadastra_consulta_e_atualiza_conexao() -> Result<(), ConectaStoreError> {
    // Etapa 1: cadastra as duas extremidades necessárias para a relação.
    let mut repository = StoreRepository::new();
    repository.add_client(1, "Ana")?;
    repository.add_product(10, "Notebook", 3000.0, true)?;
    // Etapa 2: cria uma relação e depois atualiza a mesma relação com outro peso.
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

    // Etapa 3: confirma o cadastro e verifica que não surgiu uma aresta duplicada.
    assert_eq!(repository.product(10)?.name, "Notebook");
    let edges = repository.graph().outgoing(NodeId::Client(1))?;
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].weight, 0.9);
    Ok(())
}

#[test]
fn rejeita_peso_e_relacionamento_invalidos() -> Result<(), ConectaStoreError> {
    // Prepara vértices válidos para isolar apenas os erros das conexões.
    let mut repository = StoreRepository::new();
    repository.add_client(1, "Ana")?;
    repository.add_product(10, "Notebook", 3000.0, true)?;

    // Um peso acima de 1,0 deve ser rejeitado.
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

    // Uma compra deve ir de cliente para produto, nunca no sentido contrário.
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
