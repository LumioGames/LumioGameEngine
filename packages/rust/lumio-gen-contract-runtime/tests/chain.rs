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
#[test]
fn sha256_known_answer_vectors() {
    // FIPS 180-4 reference digest of the empty message.
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
    // FIPS 180-4 B.1: single-block message.
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
    // FIPS 180-4 B.2: 448-bit message, 56 bytes, spans two compression blocks.
    assert_eq!(
        sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
    );
}
