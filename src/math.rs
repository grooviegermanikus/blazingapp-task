



#[cfg(test)]
mod tests {
    use crate::utils::{price_to_sqrt_price_x64, price_to_x64, sqrt_price_x64_to_price};

    #[test]
    fn test() {
        // let sqrt_price = sqrt_price_x64_to_price(901697932954476299104, 4, 0);
        let sqrt_price = sqrt_price_x64_to_price(901697932954476299104, 4, 0);
        println!("sqrt_price: {}", sqrt_price);
    }
}