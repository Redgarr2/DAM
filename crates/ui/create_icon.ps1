Add-Type -AssemblyName System.Drawing
$bmp = New-Object System.Drawing.Bitmap(32, 32)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.FillRectangle([System.Drawing.Brushes]::Blue, 0, 0, 32, 32)
$bmp.Save("icons\icon.ico", [System.Drawing.Imaging.ImageFormat]::Icon)
$g.Dispose()
$bmp.Dispose()
