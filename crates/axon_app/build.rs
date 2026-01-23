//! Build script for Axon Terminal
//!
//! Handles Windows-specific resource embedding (application icon).
//! Automatically converts SVG logo to ICO format for Windows.

fn main() {
    // Only run Windows-specific logic on Windows
    #[cfg(target_os = "windows")]
    windows_build();
}

#[cfg(target_os = "windows")]
fn windows_build() {
    use std::path::Path;

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    // Source SVG and target ICO paths
    let svg_path = Path::new("../../assets/icons/logo.svg");
    let ico_path = out_dir.join("app-icon.ico");

    println!("cargo:rerun-if-changed={}", svg_path.display());
    println!("cargo:rerun-if-env-changed=RELEASE_CHANNEL");

    // Check if SVG exists
    if !svg_path.exists() {
        println!(
            "cargo::warning=SVG logo not found at '{}'. Window icon will not be set.",
            svg_path.display()
        );
        return;
    }

    // Convert SVG to ICO
    if let Err(e) = svg_to_ico(svg_path, &ico_path) {
        println!("cargo::warning=Failed to convert SVG to ICO: {}. Window icon will not be set.", e);
        return;
    }

    // Set up Windows resource
    let mut res = winresource::WindowsResource::new();

    // Allow specifying RC toolkit path for environments with restricted security
    if let Ok(toolkit_path) = std::env::var("AXON_RC_TOOLKIT_PATH") {
        res.set_toolkit_path(&toolkit_path);
    }

    // Set the application icon
    res.set_icon(ico_path.to_str().unwrap());

    // Set application metadata
    res.set("FileDescription", "zTerm - Modern Terminal");
    res.set("ProductName", "zTerm");
    res.set("CompanyName", "zTerm");

    // Compile the resource
    if let Err(e) = res.compile() {
        println!("cargo::warning=Failed to compile Windows resources: {}", e);
    } else {
        println!("cargo::warning=Successfully embedded application icon");
    }
}

#[cfg(target_os = "windows")]
fn svg_to_ico(svg_path: &std::path::Path, ico_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::BufWriter;

    // Read SVG content
    let svg_data = std::fs::read(svg_path)?;

    // Parse SVG
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&svg_data, &options)?;

    // ICO supports multiple sizes; we'll generate common sizes
    let sizes = [16, 32, 48, 64, 128, 256];

    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for size in sizes {
        // Create pixmap for this size
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)
            .ok_or("Failed to create pixmap")?;

        // Calculate scale to fit SVG into the target size
        let svg_size = tree.size();
        let scale = (size as f32 / svg_size.width()).min(size as f32 / svg_size.height());

        // Center the SVG in the pixmap
        let offset_x = (size as f32 - svg_size.width() * scale) / 2.0;
        let offset_y = (size as f32 - svg_size.height() * scale) / 2.0;

        let transform = resvg::tiny_skia::Transform::from_scale(scale, scale)
            .post_translate(offset_x, offset_y);

        // Render SVG to pixmap
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // Convert RGBA to BGRA (ICO format requirement)
        let mut rgba_data = pixmap.take();
        for chunk in rgba_data.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Swap R and B
        }

        // Create ICO image entry
        let image = ico::IconImage::from_rgba_data(size, size, rgba_data);
        icon_dir.add_entry(ico::IconDirEntry::encode(&image)?);
    }

    // Write ICO file
    let file = File::create(ico_path)?;
    let writer = BufWriter::new(file);
    icon_dir.write(writer)?;

    Ok(())
}
