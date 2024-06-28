use std::io;
#[cfg(windows)]
use winres::WindowsResource;

#[allow(clippy::unnecessary_wraps)]
// I'm just keeping this as the example had it, no need to warn me about this since it doesn't affect the actual code
/// This code will set the windows program icon to the fbt logo
///
/// # Errors
///
/// This function will return an error if the icon file cannot be found at compile time.
fn main() -> io::Result<()> {
    #[cfg(windows)]
    {
        WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("assets/icon.ico")
            .compile()?;
    }
    Ok(())
}
