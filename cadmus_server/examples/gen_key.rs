
use rand::RngCore;


fn main() -> Result<(), ()> {
        let mut key = [0u8; 64];
        rand::rng().fill_bytes(&mut key);

        let b64u = base64_url::encode(&key);
        println!("\nKey b64u encoded:\n{b64u}");

        Ok(())
}
