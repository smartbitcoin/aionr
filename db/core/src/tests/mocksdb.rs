use super::*;

#[test]
fn crud_test() {
    let mut db = Mockkvdb::new_default();

    let key1: Vec<u8> = vec![1];
    let value1: Vec<u8> = vec![1];
    let key2: Vec<u8> = vec![2];
    let value2: Vec<u8> = vec![2];
    let value3: Vec<u8> = vec![3];

    db.put(&key1, &DBValue::from_vec(value1.clone()));
    assert_eq!(db.get(&key1).unwrap(), value1);

    db.put(&key2, &DBValue::from_vec(value2.clone()));
    assert_eq!(db.get(&key2).unwrap(), value2);

    db.put(&key1, &DBValue::from_vec(value3.clone()));
    assert_eq!(db.get(&key1).unwrap(), value3);

    db.delete(&key1);
    db.delete(&key2);

    assert_eq!(db.get(&key1), None);
}
