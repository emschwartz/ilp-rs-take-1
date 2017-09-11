use ilp_packet;
use base64;
use rand::{Rng, OsRng};
use ring::{hmac, digest};

const PSK_CONDITION_STRING: &'static [u8]= b"ilp_psk_condition";

fn get_psk_token() -> String {
    // TODO use ring SecureRandom instead
    let mut rng = OsRng::new().unwrap();
    let bytes: [u8; 16] = rng.gen();
    base64::encode_config(&bytes, base64::URL_SAFE_NO_PAD)
}

fn hmac(key: &[u8], message: &[u8]) -> Vec<u8> {
    let s_key = hmac::SigningKey::new(&digest::SHA256, key);
    hmac::sign(&s_key, message).as_ref().to_vec()
}

fn packet_to_preimage(shared_secret: &[u8], packet: &[u8]) -> Vec<u8> {
    let psk_condition_key = hmac(shared_secret, PSK_CONDITION_STRING);
    hmac(&psk_condition_key, packet)
}

pub fn create_packet_and_condition(shared_secret: &[u8], destination_account: &str, destination_amount: u64) -> (Vec<u8>, Vec<u8>) {
    let nonce = get_psk_token();
    // TODO support encryption and memos
    let data = format!("PSK/1.0\nNonce: {}\nEncryption: none\n\n\n\n", nonce).into_bytes();
    let packet = ilp_packet::packet::IlpPayment {
        account: destination_account.to_string(),
        amount: destination_amount,
        data,
        // TODO don't use unwrap
    }.to_bytes().unwrap();
    let preimage = packet_to_preimage(shared_secret, &packet);
    let condition = digest::digest(&digest::SHA256, &preimage).as_ref().to_vec();
    (packet, condition)
}
