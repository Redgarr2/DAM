Add-Type -AssemblyName System.Drawing

# Create a 1024x1024 transparent bitmap
$bitmap = New-Object System.Drawing.Bitmap(1024, 1024)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)

# Clear with transparent background
$graphics.Clear([System.Drawing.Color]::Transparent)

# Save as PNG
$bitmap.Save("crates\ui\src-tauri\icons\icon.png", [System.Drawing.Imaging.ImageFormat]::Png)

# Clean up
$graphics.Dispose()
$bitmap.Dispose()

Write-Host "Created transparent 1024x1024 icon.png"
