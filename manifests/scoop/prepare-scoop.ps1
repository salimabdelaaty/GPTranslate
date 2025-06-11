# GPTranslate Scoop Manifest Helper
# This script helps validate and prepare your Scoop manifest for submission

param(
    [switch]$Validate = $false,
    [switch]$Test = $false,
    [switch]$Help = $false
)

if ($Help) {
    Write-Host "GPTranslate Scoop Manifest Helper" -ForegroundColor Green
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\prepare-scoop.ps1 [-Validate] [-Test]"
    Write-Host ""
    Write-Host "Parameters:"
    Write-Host "  -Validate    Validate the Scoop manifest"
    Write-Host "  -Test        Test local installation with Scoop"
    Write-Host "  -Help        Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\prepare-scoop.ps1 -Validate"
    Write-Host "  .\prepare-scoop.ps1 -Test"
    exit 0
}

$ManifestPath = ".\scoop\gptranslate.json"

function Test-ScoopAvailable {
    try {
        $scoopVersion = scoop --version
        Write-Host "Using Scoop version: $scoopVersion" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Error "Scoop is not available. Please install Scoop first: https://scoop.sh"
        return $false
    }
}

function Validate-ScoopManifest {
    Write-Host "Validating Scoop manifest..." -ForegroundColor Yellow
    
    if (-not (Test-Path $ManifestPath)) {
        Write-Error "Manifest file not found: $ManifestPath"
        return $false
    }
    
    try {
        # Test JSON parsing
        $manifest = Get-Content $ManifestPath -Raw | ConvertFrom-Json
        Write-Host "✅ JSON syntax is valid" -ForegroundColor Green
        
        # Check required fields
        $requiredFields = @('version', 'description', 'url', 'hash')
        foreach ($field in $requiredFields) {
            if (-not $manifest.$field) {
                Write-Error "Missing required field: $field"
                return $false
            }
        }
        Write-Host "✅ All required fields present" -ForegroundColor Green
        
        # Validate hash format
        if ($manifest.hash -notmatch '^[a-fA-F0-9]{64}$') {
            Write-Error "Invalid SHA256 hash format"
            return $false
        }
        Write-Host "✅ Hash format is valid" -ForegroundColor Green
        
        Write-Host "Manifest validation successful!" -ForegroundColor Green
        return $true
    }
    catch {
        Write-Error "Manifest validation failed: $_"
        return $false
    }
}

function Test-ScoopInstall {
    Write-Host "Testing Scoop installation..." -ForegroundColor Yellow
    
    if (-not (Test-ScoopAvailable)) {
        return $false
    }
    
    try {
        # Install from local manifest
        Write-Host "Installing GPTranslate from local manifest..." -ForegroundColor Yellow
        scoop install $ManifestPath
        
        Write-Host "Installation completed!" -ForegroundColor Green
        Write-Host ""
        Write-Host "To uninstall: scoop uninstall gptranslate" -ForegroundColor Cyan
        return $true
    }
    catch {
        Write-Error "Installation test failed: $_"
        return $false
    }
}

function Show-ScoopSubmissionSteps {
    Write-Host ""
    Write-Host "Next Steps for Scoop Submission:" -ForegroundColor Cyan
    Write-Host "================================"
    Write-Host ""
    Write-Host "Option 1: Submit to Official Scoop Extras Bucket:" -ForegroundColor Yellow
    Write-Host "1. Fork: https://github.com/ScoopInstaller/Extras"
    Write-Host "2. Add your manifest: bucket/gptranslate.json"
    Write-Host "3. Create pull request"
    Write-Host ""
    Write-Host "Option 2: Create Your Own Bucket:" -ForegroundColor Yellow
    Write-Host "1. Create repository: scoop-bucket"
    Write-Host "2. Add manifest as: gptranslate.json"
    Write-Host "3. Users add bucket: scoop bucket add your-bucket https://github.com/username/scoop-bucket"
    Write-Host "4. Users install: scoop install your-bucket/gptranslate"
    Write-Host ""
    Write-Host "Option 3: Direct Installation:" -ForegroundColor Yellow
    Write-Host "Users can install directly from URL:"
    Write-Host "scoop install https://raw.githubusercontent.com/philberndt/gptranslate/main/scoop/gptranslate.json"
    Write-Host ""
    Write-Host "Recommended Buckets for Submission:" -ForegroundColor Green
    Write-Host "• Extras: https://github.com/ScoopInstaller/Extras (for GUI apps)"
    Write-Host "• Main: https://github.com/ScoopInstaller/Main (for CLI tools)"
    Write-Host ""
}

# Main execution
Write-Host "GPTranslate Scoop Manifest Preparation" -ForegroundColor Green
Write-Host "======================================" -ForegroundColor Green

if ($Validate) {
    if (Validate-ScoopManifest) {
        Write-Host "✅ Manifest is ready for submission!" -ForegroundColor Green
    }
}

if ($Test) {
    if (Test-ScoopAvailable) {
        Test-ScoopInstall
    }
}

if (-not $Validate -and -not $Test) {
    Show-ScoopSubmissionSteps
}

Write-Host ""
Write-Host "Scoop manifest created: $ManifestPath" -ForegroundColor Green
