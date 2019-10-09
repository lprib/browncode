include!(concat!(env!("OUT_DIR"), "/grammar.rs"));

#[allow(unused_imports)]
mod test {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_ident() {
        assert_eq!("Hello123", ident("Hello123").unwrap())
    }

    #[test]
    fn test_deref() {
        println!("{:?}", expression("32").unwrap());
        println!("{:?}", expression("[32]").unwrap());
    }
}
