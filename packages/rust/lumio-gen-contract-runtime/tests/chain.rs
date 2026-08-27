use lumio_gen_contract_runtime::*;
#[test]
fn chain_round_trip() {
    let genesis = Hash256(sha256(b""));
    let next = hash_chain_append(&genesis, b"rec-1");
    assert!(hash_chain_verify(&genesis, b"rec-1", &next).is_ok());
    assert!(hash_chain_verify(&genesis, b"rec-2", &next).is_err());
}
#[test]
fn truncated_buffer() {
    let mut buf = BoundedBuffer::new(2);
    assert!(buf.push(1).is_ok());
    assert!(buf.push(2).is_ok());
    assert!(buf.push(3).is_err());
}
