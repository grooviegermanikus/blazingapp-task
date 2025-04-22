fn main() {

    let b64 = include_str!("tickarrayaccount.dat");
    base64::decode(&b64).unwrap();

}

