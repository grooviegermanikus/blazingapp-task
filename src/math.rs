#[cfg(test)]
mod tests {
    use crate::utils::sqrt_price_x64_to_price;

    #[test]
    fn test_sqrt_price() {
        let sqrt_price = sqrt_price_x64_to_price(901697932954476299104, 0, 0);
        assert!((sqrt_price - 2389.3661304033488).abs() < 0.0001);
    }
}
