# Generate Windows icons from SVG files using ImageMagick
# Following Microsoft's Windows 11 design specifications
# 
# This script generates:
# - Main app icons (ICO format with multiple sizes)
# - System tray icons for light/dark themes
# - PNG icons for various Windows contexts
# 
# Requirements: ImageMagick must be installed and available in PATH

param(
    [switch]$Force,
    [switch]$Verbose
)

# Enable verbose output if requested
if ($Verbose) {
    $VerbosePreference = "Continue"
}

# Check if ImageMagick is available
function Test-ImageMagick {
    try {
        $null = & magick -version 2>$null
        return $true
    }
    catch {
        return $false
    }
}

# Create directory if it doesn't exist
function Ensure-Directory {
    param([string]$Path)
    if (-not (Test-Path $Path)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
        Write-Verbose "Created directory: $Path"
    }
}

# Convert SVG to PNG at specified size with high quality
function Convert-SvgToPng {
    param(
        [string]$InputPath,
        [string]$OutputPath,
        [int]$Size,
        [string]$Background = "transparent"
    )
    
    $arguments = @(
        "-background", $Background,
        "-density", "300",
        $InputPath,
        "-resize", "${Size}x${Size}",
        "-extent", "${Size}x${Size}",
        "-gravity", "center",
        $OutputPath
    )
    
    Write-Verbose "Converting $InputPath to $OutputPath (${Size}x${Size})"
    & magick @arguments
    
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to convert $InputPath to $OutputPath"
    }
}

# Create ICO file from multiple PNG files with proper ordering for Tauri
function Create-IcoFile {
    param(
        [string[]]$InputPngs,
        [string]$OutputPath
    )
    
    Write-Verbose "Creating ICO file: $OutputPath"
    
    # Sort PNGs to ensure 32px comes first (Tauri requirement)
    # Then order by size: 32, 16, 24, 48, 64, 256
    $orderedPngs = @()
    $sizeOrder = @(32, 16, 24, 48, 64, 256)
    
    foreach ($size in $sizeOrder) {
        $matchingPng = $InputPngs | Where-Object { $_ -match "_${size}\.png$" -or $_ -match "app_${size}\.png$" }
        if ($matchingPng) {
            $orderedPngs += $matchingPng
        }
    }
    
    # Add any remaining PNGs that don't match the standard sizes
    foreach ($png in $InputPngs) {
        if ($png -notin $orderedPngs) {
            $orderedPngs += $png
        }
    }
    
    Write-Verbose "ICO layer order: $($orderedPngs -join ', ')"
    $arguments = @($orderedPngs) + @($OutputPath)
    & magick @arguments
    
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to create ICO file: $OutputPath"
    }
}

# Main execution
try {
    Write-Host "Generating Windows icons for GPTranslate..." -ForegroundColor Green
    
    # Check prerequisites
    if (-not (Test-ImageMagick)) {
        throw "ImageMagick is not installed or not available in PATH. Please install ImageMagick first."
    }
    
    # Define paths
    $projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
    $imgDir = Join-Path $projectRoot "img"
    $iconsDir = Join-Path $projectRoot "src-tauri" "icons"
    $tempDir = Join-Path $projectRoot "temp_icons"
    
    # Source SVG files
    $appSvg = Join-Path $imgDir "logo_app.svg"
    $trayDarkSvg = Join-Path $imgDir "logo_tray_dark.svg"
    $trayLightSvg = Join-Path $imgDir "logo_tray_light.svg"
    
    # Verify source files exist
    $requiredFiles = @($appSvg, $trayDarkSvg, $trayLightSvg)
    foreach ($file in $requiredFiles) {
        if (-not (Test-Path $file)) {
            throw "Required file not found: $file"
        }
    }
    
    # Create directories
    Ensure-Directory $iconsDir
    Ensure-Directory $tempDir
    
    Write-Host "Converting SVG files to PNG..." -ForegroundColor Yellow
    
    # Windows icon sizes following Microsoft specifications
    # Core sizes: 16, 24, 32, 48, 256 (minimum required)
    # Additional sizes for better scaling: 20, 30, 36, 40, 60, 64, 72, 80, 96, 128
    $iconSizes = @(16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 128, 256)
    
    # System tray specific sizes (typically 16, 20, 24, 32)
    $traySizes = @(16, 20, 24, 32)
    
    # Generate main app icon PNGs
    $appPngs = @()
    foreach ($size in $iconSizes) {
        $pngPath = Join-Path $tempDir "app_${size}.png"
        Convert-SvgToPng -InputPath $appSvg -OutputPath $pngPath -Size $size
        $appPngs += $pngPath
        
        # Copy some standard sizes to icons directory for Tauri
        if ($size -in @(32, 64, 128)) {
            $destPath = Join-Path $iconsDir "${size}x${size}.png"
            Copy-Item $pngPath $destPath -Force
            Write-Verbose "Copied ${size}x${size}.png to icons directory"
        }
    }
    
    # Generate tray icons (dark theme)
    foreach ($size in $traySizes) {
        $pngPath = Join-Path $iconsDir "tray_dark_${size}.png"
        Convert-SvgToPng -InputPath $trayDarkSvg -OutputPath $pngPath -Size $size
    }
    
    # Generate tray icons (light theme)
    foreach ($size in $traySizes) {
        $pngPath = Join-Path $iconsDir "tray_light_${size}.png"
        Convert-SvgToPng -InputPath $trayLightSvg -OutputPath $pngPath -Size $size
    }
    
    Write-Host "Creating ICO files..." -ForegroundColor Yellow
      # Create main app ICO file with required sizes for Tauri (32px first)
    $mainIcoPath = Join-Path $iconsDir "icon.ico"
    $requiredSizes = @(32, 16, 24, 48, 64, 256)
    $mainIcoPngs = @()
    foreach ($size in $requiredSizes) {
        $pngPath = Join-Path $tempDir "app_${size}.png"
        if (Test-Path $pngPath) {
            $mainIcoPngs += $pngPath
        }
    }
    Create-IcoFile -InputPngs $mainIcoPngs -OutputPath $mainIcoPath
    
    # Create smaller ICO files for specific contexts
    $smallSizes = @(16, 20, 24, 32)
    $smallPngs = $appPngs | Where-Object { 
        $size = [regex]::Match($_, "app_(\d+)\.png").Groups[1].Value
        $size -in $smallSizes
    }
    $smallIcoPath = Join-Path $iconsDir "icon_small.ico"
    Create-IcoFile -InputPngs $smallPngs -OutputPath $smallIcoPath
    
    # Create tray ICO files
    $trayDarkPngs = @()
    $trayLightPngs = @()
    foreach ($size in $traySizes) {
        $trayDarkPngs += Join-Path $iconsDir "tray_dark_${size}.png"
        $trayLightPngs += Join-Path $iconsDir "tray_light_${size}.png"
    }
    
    $trayDarkIcoPath = Join-Path $iconsDir "tray_dark.ico"
    $trayLightIcoPath = Join-Path $iconsDir "tray_light.ico"
    Create-IcoFile -InputPngs $trayDarkPngs -OutputPath $trayDarkIcoPath
    Create-IcoFile -InputPngs $trayLightPngs -OutputPath $trayLightIcoPath
    
    # Generate special Windows icon formats following Microsoft naming conventions
    Write-Host "Creating Windows-specific icon formats..." -ForegroundColor Yellow
    
    # AppList target sizes (required for Windows Store and modern apps)
    $targetSizes = @(16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 256)
    foreach ($size in $targetSizes) {
        # Default theme
        $targetPath = Join-Path $iconsDir "AppList.targetsize-${size}.png"
        Convert-SvgToPng -InputPath $appSvg -OutputPath $targetPath -Size $size
        
        # Dark theme (unplated)
        $darkPath = Join-Path $iconsDir "AppList.targetsize-${size}_altform-unplated.png"
        Convert-SvgToPng -InputPath $appSvg -OutputPath $darkPath -Size $size
        
        # Light theme (unplated)
        $lightPath = Join-Path $iconsDir "AppList.targetsize-${size}_altform-lightunplated.png"
        Convert-SvgToPng -InputPath $appSvg -OutputPath $lightPath -Size $size
    }
    
    # Scale-based icons for Windows 10 compatibility
    $scales = @(100, 125, 150, 200, 400)
    foreach ($scale in $scales) {
        $size = [math]::Round(44 * ($scale / 100))  # Base size 44px
        $scalePath = Join-Path $iconsDir "AppList.scale-${scale}.png"
        Convert-SvgToPng -InputPath $appSvg -OutputPath $scalePath -Size $size
    }
    
    # Clean up temp directory
    if (Test-Path $tempDir) {
        Remove-Item $tempDir -Recurse -Force
        Write-Verbose "Cleaned up temporary directory"
    }
    
    Write-Host "✅ Icon generation completed successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Generated files:" -ForegroundColor Cyan
    Write-Host "  📁 Main app icons: icon.ico, icon_small.ico" -ForegroundColor White
    Write-Host "  📁 Tray icons: tray_dark.ico, tray_light.ico" -ForegroundColor White
    Write-Host "  📁 PNG variants: Various sizes for different contexts" -ForegroundColor White
    Write-Host "  📁 Windows Store: AppList.targetsize-* variants" -ForegroundColor White
    Write-Host ""
    Write-Host "Files are located in: $iconsDir" -ForegroundColor Gray
    
} catch {
    Write-Error "Failed to generate icons: $_"
    exit 1
}