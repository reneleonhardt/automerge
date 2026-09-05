use automerge::{
    transaction::Transactable, AutoCommit, Automerge, ReadDoc, SaveOptions, Value, ROOT,
};

#[test]
fn automerge_save_is_stable_until_mutation() -> Result<(), automerge::AutomergeError> {
    let mut doc = Automerge::new();
    {
        let mut tx = doc.transaction();
        tx.put(ROOT, "key", "value")?;
        tx.commit();
    }

    let saved = doc.save_with_options(SaveOptions::default());
    assert_eq!(saved, doc.save());

    let mut tx = doc.transaction();
    tx.put(ROOT, "key", "updated")?;
    tx.commit();
    let updated = doc.save();
    assert_ne!(saved, updated);

    let loaded = Automerge::load(&updated)?;
    assert_eq!(loaded.get(ROOT, "key")?.unwrap().0, Value::from("updated"));
    Ok(())
}

#[test]
fn autocommit_save_is_stable_until_mutation() -> Result<(), automerge::AutomergeError> {
    let mut doc = AutoCommit::new();
    doc.put(ROOT, "key", "value")?;

    let saved = doc.save();
    assert_eq!(saved, doc.save());

    doc.put(ROOT, "key", "updated")?;
    let updated = doc.save();
    assert_ne!(saved, updated);

    let loaded = AutoCommit::load(&updated)?;
    assert_eq!(loaded.get(ROOT, "key")?.unwrap().0, Value::from("updated"));
    Ok(())
}
