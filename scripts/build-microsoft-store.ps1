$ErrorActionPreference = "Stop"

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)][string]$Label,
        [Parameter(Mandatory = $true)][scriptblock]$Command
    )
    Write-Host ""
    Write-Host "== $Label ==" -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE."
    }
}

$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot

$pfxPath = $null
$importedThumbprints = @()
$signtool = $null

try {
    if ([string]::IsNullOrWhiteSpace($env:WINDOWS_PFX_BASE64) -or
        [string]::IsNullOrWhiteSpace($env:WINDOWS_PFX_PASSWORD)) {
        throw "Set WINDOWS_PFX_BASE64 and WINDOWS_PFX_PASSWORD to a trusted code-signing PFX before building the Microsoft Store installer."
    }

    if (!(Test-Path "node_modules/@tauri-apps/cli")) {
        throw "Dependencies are missing. Run npm ci from the repository root first."
    }

    $tauriConfig = Get-Content "src-tauri/tauri.conf.json" -Raw | ConvertFrom-Json
    if ([string]::IsNullOrWhiteSpace($tauriConfig.bundle.publisher)) {
        throw "Set bundle.publisher in src-tauri/tauri.conf.json to the real publisher identity before creating the Store installer."
    }

    Invoke-Checked "Validate Store-specific project configuration" { npm run check:store-config }
    Invoke-Checked "Check Tauri command permissions" { npm run check:acl }
    Invoke-Checked "Check drag overlay behavior" { npm run check:drag-overlay }
    Invoke-Checked "Check dependency versions" { npm run check:tauri-versions }
    Invoke-Checked "Type-check the frontend" { npm run typecheck }
    Invoke-Checked "Audit npm dependencies" { npm audit --audit-level=high }
    Invoke-Checked "Check Rust formatting" { cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check }
    Invoke-Checked "Run Rust tests" { cargo test --manifest-path src-tauri/Cargo.toml --locked }
    Invoke-Checked "Run Clippy" { cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings }
    Invoke-Checked "Generate application icons" { npm run icons }

    Remove-Item -Recurse -Force "src-tauri/target/release/bundle" -ErrorAction SilentlyContinue
    Remove-Item -Force "src-tauri/target/release/widget-website.exe" -ErrorAction SilentlyContinue

    # Compile the app without bundling. It must be signed before NSIS embeds it.
    Invoke-Checked "Compile the Windows application without bundling" { npm run tauri build -- --no-bundle }

    $appBinary = Join-Path $repoRoot "src-tauri/target/release/widget-website.exe"
    if (!(Test-Path $appBinary)) {
        throw "The application executable was not generated: $appBinary"
    }

    $pfxPath = Join-Path ([System.IO.Path]::GetTempPath()) ("widget-website-store-" + [guid]::NewGuid().ToString("N") + ".pfx")
    $pfxBytes = [Convert]::FromBase64String(($env:WINDOWS_PFX_BASE64 -replace '\s', ''))
    [System.IO.File]::WriteAllBytes($pfxPath, $pfxBytes)
    $securePassword = ConvertTo-SecureString -String $env:WINDOWS_PFX_PASSWORD -AsPlainText -Force

    $existingThumbprints = @(Get-ChildItem Cert:\CurrentUser\My | ForEach-Object { $_.Thumbprint })
    $certificates = @(Import-PfxCertificate -FilePath $pfxPath -CertStoreLocation Cert:\CurrentUser\My -Password $securePassword)
    $certificate = $certificates | Where-Object { $_.HasPrivateKey } | Select-Object -First 1

    if ($null -eq $certificate) {
        throw "The PFX did not contain a certificate with a private key."
    }
    if ($certificate.NotBefore -gt (Get-Date) -or $certificate.NotAfter -le (Get-Date)) {
        throw "The code-signing certificate is not currently valid."
    }

    $ekuExtension = $certificate.Extensions |
        Where-Object { $_.Oid.Value -eq "2.5.29.37" } |
        Select-Object -First 1
    $ekuOids = @()
    if ($null -ne $ekuExtension) {
        $ekuOids = @($ekuExtension.EnhancedKeyUsages | ForEach-Object { $_.Value })
    }
    if ($ekuOids -notcontains "1.3.6.1.5.5.7.3.3") {
        throw "The selected certificate does not explicitly include the Code Signing EKU."
    }

    $importedThumbprints = @(
        $certificates |
        Where-Object { $existingThumbprints -notcontains $_.Thumbprint } |
        ForEach-Object { $_.Thumbprint }
    )

    $sdkBin = Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"
    $signtool = Get-ChildItem $sdkBin -Recurse -Filter "signtool.exe" -File -ErrorAction SilentlyContinue |
        Where-Object { $_.Directory.Name -eq "x64" } |
        Sort-Object {
            try { [version]$_.Directory.Parent.Name }
            catch { [version]"0.0" }
        } -Descending |
        Select-Object -First 1

    if ($null -eq $signtool) {
        throw "Windows SDK signtool.exe was not found. Install the Windows 10/11 SDK and retry."
    }

    function Sign-And-Verify {
        param([Parameter(Mandatory = $true)][string]$Path)

        Write-Host "Signing: $Path"
        & $signtool.FullName sign /s My /sha1 $certificate.Thumbprint /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 $Path
        if ($LASTEXITCODE -ne 0) {
            throw "signtool failed to sign $Path."
        }

        & $signtool.FullName verify /pa /v $Path
        if ($LASTEXITCODE -ne 0) {
            throw "The Authenticode signature did not verify for $Path. Check the certificate chain and timestamp."
        }
    }

    # Sign the application binary before NSIS packages it.
    Sign-And-Verify -Path $appBinary

    # Tauri merges the Store-only config here, embedding the offline WebView2 installer.
    Invoke-Checked "Create the offline NSIS installer for Microsoft Store" {
        npm run tauri bundle -- --config src-tauri/tauri.microsoftstore.conf.json
    }

    $installerDirectory = Join-Path $repoRoot "src-tauri/target/release/bundle/nsis"
    $installers = @(Get-ChildItem $installerDirectory -Filter "*.exe" -File -ErrorAction SilentlyContinue)
    if ($installers.Count -ne 1) {
        throw "Expected exactly one new NSIS installer; found $($installers.Count)."
    }

    # Sign the outer installer after its contents have already been signed.
    Sign-And-Verify -Path $installers[0].FullName

    foreach ($signedFile in @($appBinary, $installers[0].FullName)) {
        $signature = Get-AuthenticodeSignature -FilePath $signedFile
        if ($signature.Status -ne "Valid") {
            throw "Windows reports a non-valid signature for $signedFile: $($signature.StatusMessage)"
        }
    }

    Write-Host ""
    Write-Host "Microsoft Store installer created and signed:" -ForegroundColor Green
    Write-Host $installers[0].FullName
    Write-Host "Silent install parameter for Partner Center: /S"
    Write-Warning "This script does not upload the installer or submit the product. Test silent install, Add/Remove Programs, updates and Store package validation before submission."
}
finally {
    if ($null -ne $pfxPath) {
        Remove-Item $pfxPath -Force -ErrorAction SilentlyContinue
    }
    foreach ($thumbprint in $importedThumbprints) {
        Remove-Item ("Cert:\CurrentUser\My\" + $thumbprint) -Force -ErrorAction SilentlyContinue
    }
    Remove-Item Env:WINDOWS_PFX_BASE64 -ErrorAction SilentlyContinue
    Remove-Item Env:WINDOWS_PFX_PASSWORD -ErrorAction SilentlyContinue
    Pop-Location
}
