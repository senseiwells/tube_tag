pub fn convert_relative_path(path: &str) -> String
{
    return format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path);
}