use uniffi_bindgen;

pub const MY_MATH_UDL: &str = "src/gl_client.udl";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for lang in ["python", "kotlin", "swift"] {
        uniffi_bindgen::generate_bindings(
            &format!("{}/{}", env!("CARGO_MANIFEST_DIR"), MY_MATH_UDL),
            None,
            vec![lang],
            Some(&format!("{}/bindings/{}", env!("CARGO_MANIFEST_DIR"), lang)),
            false,
        )?;
    }

    Ok(())
}
